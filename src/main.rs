use std::sync::Mutex;

use actix::Actor;
use actix_files::Files;
use actix_web::{App, HttpServer, web};

use crate::broadcaster::Broadcaster;

mod broadcaster;
mod messages;
mod server;
mod websocket;

const FRONT_BUILD_DIR: &str = "./front/dist";

#[derive(Debug)]
pub struct AppState {
    pub server_pid: Mutex<Option<u32>>,
    pub broadcaster: actix::Addr<Broadcaster>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let broadcaster_addr = Broadcaster::new().start();

    let app_state = web::Data::new(AppState {
        server_pid: Mutex::new(None),
        broadcaster: broadcaster_addr,
    });

    println!("Ready !");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(
                web::scope("/api")
                    .route("/start", web::post().to(server::start_server))
                    .route("/stop", web::post().to(server::stop_server)),
            )
            .service(web::scope("/ws").route("/logs", web::get().to(websocket::ws_route)))
            .service(Files::new("/", FRONT_BUILD_DIR).index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
