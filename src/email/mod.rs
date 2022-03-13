/*
* @author Bodo (Hugo) Barwich
* @version 2022-03-10
* @package Grafana Alerting
* @subpackage Email Sending Actor

* This Module defines the Actor that spawns dedicated Email Sending Threads
*
*---------------------------------
* Requirements:
*/

//use actix::dev::MessageResponse;
use actix::prelude::*;
//use actix::fut;
use actix_web::web;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Structure for Incoming Data
#[derive(Debug, Serialize, Deserialize)]
//#[rtype(result = "Result<EmailResponse, EmailError>")]
pub struct EmailData {
    subject: String,
    from: String,
    to: String,
    message: String,
}

/// Structure for Email Sending Results
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailResponse {
    status: String,
    report: String,
}

/// Structure for Email Sending Errors
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailError {
    status: String,
    report: String,
}

impl Message for EmailData {
    type Result = Result<EmailResponse, EmailError>;
}

/*
impl<A, M> MessageResponse<A, M> for EmailResponse
where
    A: Actor,
    M: Message<Result = EmailResponse>,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}
*/

// Define actor
pub struct EmailSender;

/* -> EmailSender::Result
impl Message for EmailSender {
    type Result = Result<EmailResponse, EmailError>;
}
*/

// Provide Actor implementation for EmailSender
impl Actor for EmailSender {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Email Sender Actor is alive");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Email Sender Actor is stopped");
    }
}

/// Define handler for `Messages` enum
impl Handler<EmailData> for EmailSender {
    //type Result = MessageResult<EmailData>;
    //type Result = ResponseActFuture<Self, Result<EmailResponse, EmailError>>;
    type Result = Result<EmailResponse, EmailError>;

    fn handle(&mut self, mail: EmailData, _ctx: &mut Self::Context) -> Self::Result {
        println!("Email Result: '{:?}'", &mail);

        //MessageResult(EmailResponse {status: String::from("sent"), report: String::from("")})
        Ok(EmailResponse {
            status: String::from("sent"),
            report: String::from(""),
        })
    }
}

#[derive(Clone)]
pub struct EmailLink {
    addr: Addr<EmailSender>,
}

// Provide Actor implementation for EmailSender
impl Actor for EmailLink {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Email Link Actor is alive");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Email Link Actor is stopped");
    }
}

impl EmailLink {
    pub fn new(addr: Addr<EmailSender>) -> Self {
        Self { addr }
    }

    pub fn send_email(
        &self,
        email: EmailData,
    ) -> impl Future<Output = Result<EmailResponse, EmailError>> + 'static {
        let sender = self.addr.clone();
        async move {
            match sender.send(email).await {
                Ok(rs) => rs,
                Err(e) => Err(EmailError {
                    status: String::from("failed"),
                    report: String::from(format!("Sending Error: '{:?}'", e)),
                }),
            }
        }
    }
}

pub async fn send_mail(
    link: web::Data<Arc<EmailLink>>,
    email: EmailData,
) -> Result<EmailResponse, EmailError> {
    // Send Email Data message.
    // send() message returns Future object, that resolves to message result
    let email_future = link.send_email(email).await;

    match email_future {
        Ok(rs) => {
            println!("Email Result: '{:?}'", &rs);
            Ok(rs)
        }
        Err(e) => {
            println!("Email Error: '{:?}'", &e);
            Err(e)
        }
    }
}
