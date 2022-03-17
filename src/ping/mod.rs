use actix::dev::{MessageResponse, ResponseChannel};
use actix::prelude::*;
use std::fmt;

#[derive(Debug, Message)]
#[rtype(result = "PingResponse")]
pub enum PingMessage {
    Ping,
    Pong,
}

impl fmt::Display for PingMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PingMessage::Ping => write!(f, "Ping"),
            PingMessage::Pong => write!(f, "Pong"),
        }
    }
}

#[derive(Debug)]
pub enum PingResponse {
    GotPing,
    GotPong,
}

impl fmt::Display for PingResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PingResponse::GotPing => write!(f, "Ping received"),
            PingResponse::GotPong => write!(f, "Pong received"),
        }
    }
}

#[derive(Debug)]
pub struct PingError {
    pub request: PingMessage,
    pub response: Option<PingResponse>,
    pub report: String,
}

impl fmt::Display for PingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Request '{}' failed: '{}'", self.request, self.report)
    }
}

impl<A, M> MessageResponse<A, M> for PingResponse
where
    A: Actor,
    M: Message<Result = PingResponse>,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}

// Define actor
struct PingActor;

// Provide Actor implementation for our actor
impl Actor for PingActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("Actor is alive");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("Actor is stopped");
    }
}

/// Define handler for `PingMessage` enum
impl Handler<PingMessage> for PingActor {
    type Result = PingResponse;

    fn handle(&mut self, msg: PingMessage, _ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            PingMessage::Ping => PingResponse::GotPing,
            PingMessage::Pong => PingResponse::GotPong,
        }
    }
}

//#[actix_rt::main]
pub async fn ping() -> Result<PingResponse, PingError> {
    // Start PingActor in current thread
    let addr = PingActor.start();

    // Send Ping message.
    // send() message returns Future object, that resolves to PingResponse result
    let ping_future = addr.send(PingMessage::Ping).await;

    println!("ping res: {:?}", ping_future);

    match ping_future {
        Ok(res) => match res {
            PingResponse::GotPing => {
                println!("Ping received");
                Ok(res)
            }
            _ => {
                println!("Wrong Response received");
                Err(PingError {
                    request: PingMessage::Ping,
                    response: Some(res),
                    report: String::from("wrong response received"),
                })
            }
        },
        Err(e) => {
            println!("Actor is probably dead: {}", e);
            Err(PingError {
                request: PingMessage::Ping,
                response: None,
                report: String::from(format!("Ping Request failed: '{}'", e)),
            })
        }
    }
}

//#[actix_rt::main]
pub async fn pong() -> Result<PingResponse, PingError> {
    // Start PingActor in current thread
    let addr = PingActor.start();

    // Send Ping message.
    // send() message returns Future object, that resolves to PingResponse result
    let pong_future = addr.send(PingMessage::Pong).await;

    println!("pong res: {:?}", pong_future);

    match pong_future {
        Ok(res) => match res {
            PingResponse::GotPong => {
                println!("Pong received");
                Ok(res)
            }
            _ => {
                println!("Wrong Response received");
                Err(PingError {
                    request: PingMessage::Pong,
                    response: Some(res),
                    report: String::from("wrong response received"),
                })
            }
        },
        Err(e) => {
            println!("Actor is probably dead: {}", e);
            Err(PingError {
                request: PingMessage::Pong,
                response: None,
                report: String::from(format!("Ping Request failed: '{}'", e)),
            })
        }
    }
}
