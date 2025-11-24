use actix::Recipient;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub addr: Recipient<WsMessage>,
}

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct BroadcastLog {
    pub message: String,
    pub is_error: bool,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);
