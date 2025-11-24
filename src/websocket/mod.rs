use actix::prelude::*;
use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_ws::Message;
use tokio::sync::mpsc;

use crate::AppState;
use crate::broadcaster::Broadcaster;
use crate::messages::{Connect, Disconnect, WsMessage};

pub struct WsForwarder {
    tx: mpsc::Sender<WsMessage>,
}

impl Actor for WsForwarder {
    type Context = Context<Self>;
}

impl Handler<WsMessage> for WsForwarder {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.tx.try_send(msg).ok();
    }
}

async fn handle_websocket_stream(
    mut session: actix_ws::Session,
    mut rx_ws_actor: mpsc::Receiver<WsMessage>,
    broadcaster_addr: Addr<Broadcaster>,
    recipient: Recipient<WsMessage>,
) {
    while let Some(msg) = rx_ws_actor.recv().await {
        if session.text(msg.0).await.is_err() {
            break;
        }
    }

    broadcaster_addr.do_send(Disconnect { addr: recipient });

    session.close(None).await.ok();
    println!("WS Client disconnected (broadcaster stream closed).");
}

async fn handle_client_messages(
    mut msg_stream: actix_ws::MessageStream,
    broadcaster_addr: Addr<Broadcaster>,
    recipient: Recipient<WsMessage>,
) {
    while let Some(msg_result) = futures::StreamExt::next(&mut msg_stream).await {
        match msg_result {
            Ok(msg) => match msg {
                Message::Text(text) => {
                    println!("WS Client sent: {text}");
                }
                Message::Ping(_) => {}
                Message::Pong(_) => {}
                Message::Close(reason) => {
                    println!("WS Client initiated close: {reason:?}");
                    break;
                }
                Message::Continuation(_) | Message::Binary(_) | Message::Nop => {}
            },
            Err(e) => {
                eprintln!("WS Stream Error: {e:?}");
                break;
            }
        }
    }

    broadcaster_addr.do_send(Disconnect { addr: recipient });

    println!("WS Client disconnected (client stream closed).");
}

pub async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    let (tx_ws_actor, rx_ws_actor) = mpsc::channel(16);

    let forwarder_addr = WsForwarder { tx: tx_ws_actor }.start();
    let recipient: Recipient<WsMessage> = forwarder_addr.recipient();

    app_state.broadcaster.do_send(Connect {
        addr: recipient.clone(),
    });

    tokio::spawn(handle_websocket_stream(
        session,
        rx_ws_actor,
        app_state.broadcaster.clone(),
        recipient.clone(),
    ));

    let client_handler_future =
        handle_client_messages(msg_stream, app_state.broadcaster.clone(), recipient);

    actix_web::rt::spawn(async move {
        client_handler_future.await;
    });

    Ok(response)
}
