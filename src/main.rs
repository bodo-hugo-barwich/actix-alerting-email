/*
* @author Bodo (Hugo) Barwich
* @version 2022-03-10
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

mod email;
mod ping;

use actix_web::{error, web, App, Error, HttpResponse, HttpServer};
use actix::Actor;
use actix::sync::SyncArbiter;
//use std::future::Future;
//use futures_core::stream::Stream;
use futures_util::stream::StreamExt;
use std::sync::Arc;
//use mime;

//use futures::StreamExt;
use json::JsonValue;
use serde::{Deserialize, Serialize};

use actix_web::middleware::Logger;

use email::{EmailSender, EmailLink, EmailData};


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
struct ResponseData {
    title: String,
    statuscode: u16,
    page: String,
    description: String,
}

/// Handler to build the Home Page
async fn dispatch_home_page() -> HttpResponse {
    //------------------------
    //Project Description

    HttpResponse::Ok().json(ResponseData {
        title: String::from("Grafana Alerting Email"),
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
async fn send_email(link: web::Data<Arc<EmailLink>>, mut payload: web::Payload) -> Result<HttpResponse, Error> {
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
            match email::send_mail(link, email).await {
              Ok(rs) => {
                println!("email res: '{:?}'", rs);
                Ok(HttpResponse::Ok().json(rs)) // <- send response
              }
              Err(e) => {
                println!("email error: '{:?}'", e);
                Err(error::ErrorBadRequest(format!("Sending failed: '{:?}'\n", e)))
              }
            }
          //}
          //, Err(_e) => {Err(error::InternalError{cause: "email link lock failed", status: 500})}
        //}
      }
      Err(e) => {
        println!("json error: '{:?}'", e);
        Err(error::ErrorBadRequest(format!("Request invalid: '{}'\n", e.to_string())))
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

async fn ping() -> Result<HttpResponse, Error> {
  println!("Request 'Ping': processing ...");
    ping::ping().await;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body("{\"ping\":\"ok\"}"))

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    println!("Email App: launching at 127.0.0.1:3100 ...");

      //Create 2 Email Sender Instances
      let sender = SyncArbiter::start(2, || EmailSender);
      //Create 1 Email Link Instance
      let link = Arc::new(EmailLink::new(sender).start());

//    let link_mutex = web::Data::new(Mutex::new(link));


    HttpServer::new(move || {
        let link = link.clone();

        App::new()
            .wrap(Logger::default())
            .app_data(link)
            .app_data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
            .service(web::resource("/").route(web::get().to(dispatch_home_page)))
            .service(web::resource("/send").route(web::post().to(send_email)))
            .service(web::resource("/mjsonrust").route(web::post().to(index_mjsonrust)))
            .service(web::resource("/ping").route(web::get().to(ping)))
    })
    .bind("127.0.0.1:3100")?
    .run()
    .await?;

    println!("Email App: finished.");

    Ok(())
}
