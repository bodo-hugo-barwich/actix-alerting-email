/*
* @author Bodo (Hugo) Barwich
* @version 2024-02-04
* @package Grafana Alerting
* @subpackage Email Micro Service

* This Module defines the HTTP Interface for accepting new Email Send Requests
*
*---------------------------------
* Requirements:
* - The Rust Crate "actix-web" must be installed
* - The Rust Crate "futures" must be installed
* - The Rust Crate "serde" must be installed
* - The Rust Crate "serde-json" must be installed
* - The Rust Crate "json" must be installed
*/

//#[macro_use]
extern crate json;

pub mod config;
pub mod email;
pub mod ping;

use std::env;

use actix::sync::SyncArbiter;
use actix_web::{error, web, App, Error, HttpResponse, HttpServer};
use futures_util::stream::StreamExt;
//use mime;

use json::JsonValue;
use serde::{Deserialize, Serialize};

use actix_web::middleware::Logger;

use config::AppConfig;
use email::{EmailData, EmailLink, EmailSender};

const MAX_SIZE: usize = 262_144; // max payload size is 256k

/*
/// Structure for Incoming Data
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailData {
    subject: String,
    from: String,
    to: String,
    message: String,
}
*/

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    pub title: String,
    pub statuscode: u16,
    pub page: String,
    pub description: String,
}

/// Handler to build the Home Page
pub async fn dispatch_home_page() -> HttpResponse {
    //------------------------
    //Project Description

    HttpResponse::Ok().json(ResponseData {
        title: String::from("Actix Alerting Email"),
        statuscode: 200,
        page: String::from("Home"),
        description: String::from("Email Sending Micro Service for the Grafana Alerting Project"),
    })

    /*
        let response_data =
         object!{
        "title" => "Grafana Alerting Email",
        "statuscode" => 200,
        "page" => "Home",
        "description" => "Email Sending Micro Service for the Grafana Alerting Project"
        };
        println!("model: {:?}", &response_data);
    */
}

/// This Handler reads the Request and parses it into EmailData object with serde
pub async fn send_email(
    link: web::Data<EmailLink>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    println!("got payload: '{:?}'", &body);

    // body is loaded, now we can deserialize serde-json

    match serde_json::from_slice::<EmailData>(&body) {
        Ok(email) => {
            //Ok(HttpResponse::Ok().json(email)) // <- send response
            //match link_mutex.lock() {
            //Ok(link_lock) => {
            match email::send_mail(&link, email).await {
                Ok(rs) => {
                    println!("email res: '{:?}'", rs);
                    Ok(HttpResponse::Ok().json(rs)) // <- send response
                }
                Err(e) => {
                    println!("email error: '{:?}'", e);
                    Err(error::ErrorBadRequest(format!(
                        "Sending failed: '{:?}'\n",
                        e
                    )))
                }
            }
            //}
            //, Err(_e) => {Err(error::InternalError{cause: "email link lock failed", status: 500})}
            //}
        }
        Err(e) => {
            println!("json error: '{:?}'", e);
            Err(error::ErrorBadRequest(format!(
                "Request invalid: '{}'\n",
                e
            )))
        }
    }
}

/// This handler manually load request payload and parse json-rust
async fn index_mjsonrust(body: web::Bytes) -> Result<HttpResponse, Error> {
    // body is loaded, now we can deserialize json-rust
    let result = json::parse(std::str::from_utf8(&body).unwrap()); // return Result
    let injson: JsonValue = match result {
        Ok(v) => v,
        Err(e) => json::object! {"err" => e.to_string() },
    };
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(injson.dump()))
}

pub async fn dispatch_ping_request() -> Result<HttpResponse, Error> {
    println!("Request 'Ping': processing ...");
    let ping_rs = ping::ping().await;

    match ping_rs {
        Ok(resp) => Ok(HttpResponse::Ok().json(ResponseData {
            title: String::from("Ping Request"),
            statuscode: 200,
            page: String::from("Ping"),
            description: format!("Ping Request [ OK ]; Message: '{}'", resp),
        })),

        Err(e) => {
            println!("ping error: '{:?}'", e);
            Err(error::ErrorBadRequest(format!("Request failed: '{}'\n", e)))
        }
    }
}

//==============================================================================
// Executing Section

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let config = AppConfig::from_file();

    println!("app config: {:?}", config);

    let component_name = match env::var("COMPONENT") {
        Ok(comp) => comp,
        Err(_) => "default".to_owned(),
    };

    let mut app_host = String::from("127.0.0.1");
    let app_port = match env::var("PORT") {
        Ok(p) => p,
        Err(_) => "3100".to_owned(),
    };

    app_host.push(':');
    app_host.push_str(app_port.as_str());

    println!(
        "Email App '{}': launching at {} ...",
        component_name, app_host
    );

    //Clone the SMTP Config for the Email Worker
    let smtp_config = config.smtp.clone();

    //Create 2 Email Sender Instances
    let sender = SyncArbiter::start(config.mail_worker as usize, move || {
        EmailSender::from_config(&smtp_config)
     });
    //Create 1 Email Link Object
    let link = EmailLink::new(sender);

    HttpServer::new(move || {
        let app_config = web::Data::new(config.clone());
        let link_data = web::Data::new(link.clone());

        App::new()
            .app_data(link_data)
            .app_data(web::JsonConfig::default().limit(MAX_SIZE)) // <- limit size of the payload (global configuration)
            .service(web::resource(app_config.web_root.as_str()).route(web::get().to(dispatch_home_page)))
            .service(web::resource(app_config.web_root.as_str().to_owned() + "send").route(web::post().to(send_email)))
            .service(web::resource(app_config.web_root.as_str().to_owned() + "mjsonrust").route(web::post().to(index_mjsonrust)))
            .service(web::resource(app_config.web_root.as_str().to_owned() + "ping").route(web::get().to(dispatch_ping_request)))
            .app_data(app_config)
            .wrap(Logger::default())
    })
    .bind(app_host)?
    .run()
    .await?;

    println!("Email App '{}': finished.", component_name);

    Ok(())
}
