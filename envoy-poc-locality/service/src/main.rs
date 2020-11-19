use actix_web::http::StatusCode;
use actix_web::{
    error::ResponseError, get, middleware::Logger, post, web, App, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use chrono::prelude::*;
use log::*;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::thread;
use std::time::SystemTime;
use structopt::StructOpt;
use tokio::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;

mod compute_actor;
use compute_actor::ComputeActor;
use compute_actor::ComputeDataRef;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct StringError {
    pub err_msg: String,
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.err_msg)
    }
}

impl Error for StringError {}

#[derive(StructOpt, Debug, Clone)]
pub struct Cli {
    /// The name of the service, provided as a number
    #[structopt(short = "n", long = "servicenumber")]
    pub servicenumber: u32,
    /// The port on which the service should listen
    #[structopt(short = "p", long = "port")]
    pub port: u32,
    /// The type of service - produce data or process data
    #[structopt(short = "t", long = "servicetype")]
    pub servicetype: String,
}

async fn docompute(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    // body is loaded, now we can deserialize json-rust
    let mut decoded_body = std::str::from_utf8(&body);
    if let Err(e) = decoded_body {
        return HttpResponse::BadRequest().body("body decode failure");
        //return HttpResponse::Ok().body("body decode failure");
    }

    let mut compute_data_ref: ComputeDataRef = match serde_json::from_str(&decoded_body.unwrap()) {
        Ok(decoded) => decoded,
        Err(e) => {
            let err_msg = format!("{:?}", e);
            warn!("{}", &err_msg);
            return HttpResponse::BadRequest().body(err_msg);
        }
    };

    let cli_arguments: Cli = req.app_data::<Cli>().unwrap().clone();
    //info!("input data: {:?}", &compute_data_ref);

    let header_map = req.headers();
    let local_jumps: String;
    if header_map.contains_key("x-force-local") {
        local_jumps = header_map
            .get("x-force-local")
            .unwrap()
            .clone()
            .to_str()
            .unwrap()
            .to_string();
    } else {
        warn!("could not file x-force-local header");
        return HttpResponse::BadRequest().body("could not file x-force-local header");
    }

    let compute = ComputeActor::spawn(compute_data_ref, cli_arguments, local_jumps);
    return HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body("ok");
    // let res = compute.join_handle.await;
    // match res {
    //     Ok(result) => match result {
    //         Ok(data) => {
    //             info!("success: {}", data);
    //             return HttpResponse::Ok().body("ok");
    //         }
    //         Err(error) => {
    //             warn!("got error {}", error);
    //             return HttpResponse::BadRequest().body(format!("{}", error));
    //             //return HttpResponse::Ok().body(format!("{}", error));
    //         }
    //     },
    //     Err(e) => {
    //         warn!("got error {:?}", e);
    //         return HttpResponse::build(StatusCode::BAD_REQUEST)
    //             .content_type("text/html; charset=utf-8")
    //             .body(format!("{:?}", e));
    //     }
    // }
}

async fn getdata(req: HttpRequest, body: web::Bytes) -> impl Responder {
    // body is loaded, now we can deserialize json-rust
    let mut decoded_body = std::str::from_utf8(&body);
    if let Err(e) = decoded_body {
        return HttpResponse::Ok().body("body decode failure");
    }
    let decoded_body = decoded_body.unwrap();
    info!("getdata body: {}", decoded_body);
    match fs::read_to_string(decoded_body.clone()) {
        Ok(contents) => HttpResponse::Ok().body(contents),
        Err(e) => {
            warn!("could not read file: {}. Error: {:?}", decoded_body, e);
            return HttpResponse::BadRequest().body(format!(
                "could not read file: {}. Error: {:?}",
                decoded_body, e
            ));
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let start = std::time::Instant::now();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    let args = Cli::from_args();
    let servicenumber = args.servicenumber;
    env_logger::Builder::from_default_env()
        .format(move |buf, rec| {
            let t = start.elapsed().as_secs_f32();
            //let t2 = Local::now().to_rfc2822();
            let t2 = Local::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            //let tid = thread::current().id();
            writeln!(
                buf,
                "[S{}] [{}] [{:.06}] [{}] [{}:{}] {}",
                servicenumber,
                t2,
                t,
                rec.level(),
                //tid,
                rec.target(),
                rec.line().unwrap(),
                rec.args(),
            )
        })
        .init();

    let self_ip = std::env::var("HOST_IP").expect("HOST_IP env variable not specified");
    let host_port: String = format!("0.0.0.0:{}", args.port);

    if std::env::var("ENVOY_PORT").is_err() {
        std::env::set_var("ENVOY_PORT", "80");
    }
    info!(
        "Service name {} starting on {}, host_ip: {}, service type: {}, envoy_port: {}",
        args.servicenumber,
        &host_port,
        self_ip,
        &args.servicetype,
        std::env::var("ENVOY_PORT").unwrap()
    );
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(args.clone())
            .data(web::PayloadConfig::new(600 * 1024 * 1024))
            //.app_data(app_state.clone())
            .route("service/{id}/docompute", web::post().to(docompute))
            .route("service/{id}/getdata", web::post().to(getdata))
    })
    .bind(host_port.as_str())?
    .workers(2)
    .run()
    .await
}
