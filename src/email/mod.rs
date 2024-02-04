/*
* @author Bodo (Hugo) Barwich
* @version 2024-02-04
* @package Grafana Alerting
* @subpackage Email Sending Actor

* This Module defines the Actor that spawns dedicated Email Sending Threads
*
*---------------------------------
* Requirements:
*/

use actix::prelude::*;
use actix::Addr;
use serde::{Deserialize, Serialize};

use lettre::smtp::client::net::ClientTlsParameters;
use lettre::smtp::{
    authentication::Credentials, authentication::Mechanism, extension::ClientId, SmtpClient,
    SmtpTransport,
};
use lettre::{ClientSecurity, Transport};
use lettre_email::{EmailBuilder, Header};
use native_tls::TlsConnector;

use core::time::Duration;

use super::config::SMTPConfig;

//==============================================================================
// Structure EmailData Declaration

/// Structure for Incoming Data
#[derive(Debug, Serialize, Deserialize)]
//#[rtype(result = "Result<EmailResponse, EmailError>")]
pub struct EmailData {
    pub subject: String,
    pub from: String,
    pub to: String,
    pub message: String,
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

//==============================================================================
// Structure EmailData Implementation

impl Message for EmailData {
    type Result = Result<EmailResponse, EmailError>;
}

//==============================================================================
// Structure EmailSender Declaration

/// Structure for the Email Sending
// Define actor
pub struct EmailSender {
  config: SMTPConfig
}


//==============================================================================
// Structure EmailSender Implementation

impl Default for EmailSender {
    /*----------------------------------------------------------------------------
     * Default Constructor
     */

    fn default() -> Self {
        Self::new()
    }
}

impl EmailSender {
    /*----------------------------------------------------------------------------
     * Constructors
     */

    pub fn new() -> Self {
        Self {
            config: SMTPConfig::new(),
        }
    }

    pub fn from_config(config: &SMTPConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /*----------------------------------------------------------------------------
     * Administration Methods
     */

    pub fn set_config(&mut self, config: &SMTPConfig) {
        self.config = config.clone();
    }
}

// Provide Actor implementation for EmailSender
impl Actor for EmailSender {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Email Sender Actor is alive");
        println!("smtp config: {:?}", self.config);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Email Sender Actor is stopped");
    }
}

/// Define handler for `EmailData` structure
impl Handler<EmailData> for EmailSender {
    type Result = Result<EmailResponse, EmailError>;

    fn handle(&mut self, mail: EmailData, _ctx: &mut Self::Context) -> Self::Result {
        println!("Email Data: '{:?}'", &mail);

        let security = ClientSecurity::Required(ClientTlsParameters::new(
            self.config.host.clone(),
            TlsConnector::new().unwrap(),
        ));
        let smtp_url = self.config.host.clone() + ":" + self.config.port.as_str();

        match SmtpClient::new(smtp_url, security) {
            Ok(smtp) => {
                let mut mailer = SmtpTransport::new(
                    smtp.hello_name(ClientId::hostname())
                        .credentials(Credentials::new(
                            self.config.login.clone(),
                            self.config.password.clone(),
                        ))
                        .authentication_mechanism(Mechanism::Login)
                        .timeout(Some(Duration::new(15, 0))),
                );

                let message = mail.message.as_bytes().to_owned();

                println!("send message: '{:?}'", &message);

                let email = EmailBuilder::new()
                    // Addresses can be specified by the tuple (email, alias)
                    // ... or by an address only
                    .from((self.config.email_address.as_str(), self.config.full_name.as_str()))
                    .header(Header::new("X-Forward-From".to_owned(), mail.from.clone()))
                    .to((self.config.email_address.as_str(), self.config.full_name.as_str()))
                    .subject(mail.subject.as_str())
                    .text(mail.message.as_str())
                    .build()
                    .unwrap();

                // Send the email via remote relay
                match mailer.send(email.into()) {
                    Ok(res) => {
                        mailer.close();

                        Ok(EmailResponse {
                            status: String::from("sent"),
                            report: format!(
                                "Email was sent with [{:?}]: {:?}",
                                res.code, res.message
                            ),
                        })
                    }
                    Err(e) => {
                        mailer.close();

                        Err(EmailError {
                            status: String::from("failed"),
                            report: format!("Sending Error - SmtpTransport: '{:?}'", e),
                        })
                    }
                }
            }
            Err(e) => Err(EmailError {
                status: String::from("failed"),
                report: format!("Sending Error - SmtpClient: '{:?}'", e),
            }),
        }
    }
}

#[derive(Clone)]
pub struct EmailLink {
    addr: Addr<EmailSender>,
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
