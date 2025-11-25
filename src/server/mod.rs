use actix_web::{HttpResponse, web};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader as TokioBufReader};
use tokio::process::Command;

use crate::AppState;
use crate::messages::BroadcastLog;

async fn handle_output_stream<T>(stream: T, app_state: web::Data<AppState>, is_error: bool)
where
    T: AsyncRead + Unpin + Send + 'static,
{
    let mut reader = TokioBufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let log_message = line.trim_end().to_string();

                let broadcaster = &app_state.broadcaster;
                let log_msg = BroadcastLog {
                    message: log_message,
                    is_error,
                };

                broadcaster.do_send(log_msg);
            }
            Err(e) => {
                eprintln!("Error reading stream: {e}");
                break;
            }
        }
    }
    println!("Stream closed.")
}

pub async fn start_server(app_state: web::Data<AppState>) -> HttpResponse {
    let mut command = Command::new(app_state.config.command.clone());

    command.current_dir(app_state.config.server_path.clone());
    command.args(app_state.config.args.clone());

    match command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            let stdout = child.stdout.take().expect("Failed to open stdout");
            let stderr = child.stderr.take().expect("Failed to open stderr");

            tokio::spawn(handle_output_stream(stdout, app_state.clone(), false));
            tokio::spawn(handle_output_stream(stderr, app_state.clone(), true));

            *app_state.server_pid.lock().unwrap() = child.id();

            println!(
                "Server started (PID : {})",
                if let Some(pid) = child.id() {
                    pid.to_string()
                } else {
                    "no pid".to_string()
                }
            );
            HttpResponse::Ok().body(format!(
                "Server started (PID: {})",
                if let Some(pid) = child.id() {
                    pid.to_string()
                } else {
                    "no pid".to_string()
                }
            ))
        }
        Err(e) => {
            eprintln!("Error starting server: {e}");
            actix_web::HttpResponse::InternalServerError()
                .body(format!("Error starting server: {e}"))
        }
    }
}

pub async fn stop_server(app_state: web::Data<AppState>) -> HttpResponse {
    let pid_option = {
        let mut server_pid = app_state.server_pid.lock().unwrap();
        server_pid.take()
    };

    match pid_option {
        Some(pid) => {
            let kill_command = format!("kill -9 {pid}");

            match Command::new("/bin/sh")
                .arg("-c")
                .arg(&kill_command)
                .output()
                .await
            {
                Ok(output) => {
                    if output.status.success() {
                        let msg = format!("Server stopped successfully. (PID: {pid})");
                        println!("{msg}");

                        app_state.broadcaster.do_send(BroadcastLog {
                            message: format!("[INFO]: {msg}"),
                            is_error: false,
                        });
                        HttpResponse::Ok().body(msg)
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let msg = format!("Failed to stop server (PID: {pid}. Error: {stderr}");
                        eprintln!("{msg}");

                        app_state.broadcaster.do_send(BroadcastLog {
                            message: format!("[ERR]: {msg}"),
                            is_error: true,
                        });

                        *app_state.server_pid.lock().unwrap() = Some(pid);
                        HttpResponse::InternalServerError().body(msg)
                    }
                }
                Err(e) => {
                    let msg = format!("Failed to execute kill command: {e}");
                    eprintln!("{msg}");

                    app_state.broadcaster.do_send(BroadcastLog {
                        message: format!("[ERR]: {msg}"),
                        is_error: true,
                    });

                    *app_state.server_pid.lock().unwrap() = Some(pid);
                    HttpResponse::InternalServerError().body(msg)
                }
            }
        }
        None => {
            println!("Server is already stopped or was never started.");
            HttpResponse::Ok().body("Server is already stopped or was never started.")
        }
    }
}
