use actix_web::HttpResponse;
use std::process::{Command, Stdio};

const START_PATH: &str = "../../server/launch";

pub async fn start_server() -> HttpResponse {
    let command = Command::new(START_PATH)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match command {
        Ok(child) => {
            println!("Server started (PID : {})", child.id());
            HttpResponse::Ok().body(format!("Server started (PID: {})", child.id()))
        }
        Err(e) => {
            eprintln!("Error starting server: {e}");
            actix_web::HttpResponse::InternalServerError()
                .body(format!("Error starting server: {e}"))
        }
    }
}

pub async fn stop_server() -> HttpResponse {
    HttpResponse::Ok().body("")
}
