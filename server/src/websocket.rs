use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use actix::{Actor, Addr, AsyncContext, StreamHandler};
use actix_web::{
    web::{Data, Payload},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{self, Message, ProtocolError, WebsocketContext};

pub type ClientList = Arc<Mutex<HashSet<Addr<MyWebSocket>>>>;

pub fn init_clients() -> ClientList {
    Arc::new(Mutex::new(HashSet::new()))
}

pub struct MyWebSocket {
    clients: ClientList,
}

impl Actor for MyWebSocket {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.clients.lock().unwrap().insert(addr);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.clients.lock().unwrap().remove(&addr);
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(Message::Text(text)) = msg {
            ctx.text(format!("Echo: {}", text));
        }
    }
}

pub async fn ws_handler(
    req: HttpRequest,
    stream: Payload,
    clients: Data<ClientList>,
) -> Result<HttpResponse, Error> {
    ws::start(
        MyWebSocket {
            clients: clients.get_ref().clone(),
        },
        &req,
        stream,
    )
}
