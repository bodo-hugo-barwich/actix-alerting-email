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

use actix::prelude::*;
use actix::Addr;
use lettre::smtp::{SmtpClient, extension::ClientId};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Structure for Incoming Data
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SMTPConfig {
    pub host: String,
    pub port: Option<u16>,
    pub hello: String,
    pub username: String,
    pub password: String,
    pub timeout: Option<u16>
}

/// Structure for Incoming Data
#[derive(Debug, Serialize, Deserialize)]
//#[rtype(result = "Result<EmailResponse, EmailError>")]
pub struct EmailData {
    pub subject: String,
    pub from: String,
    pub to: String,
    pub message: String,
}

#[derive(Debug)]
struct EmailRequest {
  config: SMTPConfig,
  data: EmailData,
}

/// Structure for Email Sending Results
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailResponse {
    pub status: String,
    pub report: String,
}

/// Structure for Email Sending Errors
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailError {
    status: String,
    report: String,
}

impl Message for EmailRequest {
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

impl EmailSender {
    fn connect(config: &SMTPConfig) -> Result<SmtpClient, EmailError> {
        let client_name = if ! config.hello.is_empty() {
          ClientId::new(String::from(config.hello.as_str()))
        } else {
          ClientId::new(String::from("localhost"))
        };
        let timeout_secs = Duration::from_secs(config.timeout.unwrap_or(15) as u64);

            match SmtpClient::new_simple(&config.host) {
              Ok(cln) => {
                Ok(cln.hello_name(client_name).timeout(Some(timeout_secs)))
              }
              , Err(e) => Err(
                  EmailError {status: String::from("failed"), report: format!("SMTP Connection failed: '{}'", e) }
                )
            } //match SmtpClient::new_simple(&cfg.host)
    }
}

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

/// Define handler for `EmailRequest` structure
impl Handler<EmailRequest> for EmailSender {
    //type Result = MessageResult<EmailData>;
    //type Result = ResponseActFuture<Self, Result<EmailResponse, EmailError>>;
    type Result = Result<EmailResponse, EmailError>;

    fn handle(&mut self, mail_request: EmailRequest, _ctx: &mut Self::Context) -> Self::Result {
        println!("Email Data: '{:?}'", &mail_request);

        let conn_rs = EmailSender::connect(&mail_request.config);

        match conn_rs {
          Ok(conn) => {
            Ok(EmailResponse {
              status: String::from("sent"),
              report: String::from("Email was sent"),
          })
          }
          , Err(e) => {
            Err(e)
          }
        } //match conn_future
    }
}

#[derive(Clone)]
pub struct EmailLink {
    addr: Addr<EmailSender>,
    config: SMTPConfig
}

impl EmailLink {
    pub fn new(addr: Addr<EmailSender>, config: SMTPConfig) -> Self {
        Self { addr, config }
    }

    pub fn send_email(
        &self,
        email: EmailData,
    ) -> impl Future<Output = Result<EmailResponse, EmailError>> + 'static {
        let sender = self.addr.clone();
        let request = EmailRequest {config: self.config.clone(), data: email };
        async move {
            match sender.send(request).await {
                Ok(rs) => rs,
                Err(e) => Err(EmailError {
                    status: String::from("failed"),
                    report: format!("Sending Error: '{:?}'", e),
                }),
            }
        }
    }
}

pub async fn send_mail(link: &EmailLink, email: EmailData) -> Result<EmailResponse, EmailError> {
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
