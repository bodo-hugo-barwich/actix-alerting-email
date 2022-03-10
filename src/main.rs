/*
* @author Bodo (Hugo) Barwich
* @version 2022-03-10
* @package Grafana Alerting
* @subpackage Email Micro Service

* This Module defines Classes to manage the Access to Files in persistent or instant Mode
*
*---------------------------------
* Requirements:
* - The Rust Crate "actix-web" must be installed
* - The Rust Crate "futures" must be installed
* - The Rust Crate "serde" must be installed
* - The Rust Crate "serde-json" must be installed
* - The Rust Crate "json" must be installed
*/

#[macro_use]
extern crate json;

use actix_web::{
    error, get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer};
//use std::future::Future;
//use futures_core::stream::Stream;
use futures_util::stream::StreamExt;
use mime;

//use futures::StreamExt;
use json::JsonValue;
use serde::{Deserialize, Serialize};

use actix_web::middleware::Logger;

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Debug, Serialize, Deserialize)]
struct EmailData {
    from: String,
    to: String,
    message: String,
}

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

/// This handler manually load request payload and parse json object
async fn index_manual(mut payload: web::Payload) -> Result<HttpResponse, Error> {
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

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<EmailData>(&body)?;
    Ok(HttpResponse::Ok().json(obj)) // <- send response
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    println!("Launching Email App at 127.0.0.1:3100");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .app_data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
            .service(web::resource("/").route(web::get().to(dispatch_home_page)))
            .service(web::resource("/manual").route(web::post().to(index_manual)))
            .service(web::resource("/mjsonrust").route(web::post().to(index_mjsonrust)))
    })
    .bind("127.0.0.1:3100")?
    .run()
    .await?;

    println!("Email App: finished.");

    Ok(())
}
