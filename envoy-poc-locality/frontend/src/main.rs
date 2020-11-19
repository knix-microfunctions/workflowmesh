use actix_web::http::StatusCode;
use actix_web::{
    error::ResponseError, get, middleware::Logger, post, web, App, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use chrono::prelude::*;
use log::*;
use rand::prelude::*;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::Write;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use tokio::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::time::delay_for;

static PORT: u32 = 12340;

type ResponseSendChannel = oneshot::Sender<String>;
type ResponseRecvChannel = oneshot::Receiver<String>;

pub fn get_unique_id() -> String {
    let r: f64 = rand::thread_rng().gen();
    let u: u32 = (r * 100000000.0 as f64) as u32;
    format!("{}", u)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComputeDataRef {
    pub request_id: String,
    pub pass_data: bool,
    pub data: String,
    pub filename: String,
    pub size: u32,
    pub loops: u32,
    pub terminal_service_number: u32,
    pub source_service_number: u32,
    pub source_service_ip: String,
    pub source_service_port: u32,
    pub frontend_ip: String,
    pub service_ips: String,
}

async fn dostart(req: HttpRequest, body: web::Bytes) -> HttpResponse {
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

    let mut request_id = get_unique_id();
    if compute_data_ref.request_id.len() > 0 {
        request_id = compute_data_ref.request_id.clone();
    }
    info!("[TRACE] [{}] [REQUEST_START_FRONTEND]", &request_id);

    let mut response_rx_chan: ResponseRecvChannel;
    {
        if let Ok(mut response_tx_map) = req
            .app_data::<web::Data<Mutex<HashMap<String, ResponseSendChannel>>>>()
            .unwrap()
            .lock()
        {
            // got the map
            if response_tx_map.contains_key(&request_id) {
                return HttpResponse::build(StatusCode::BAD_REQUEST)
                    .content_type("text/html; charset=utf-8")
                    .body(format!("Request id {}, already exists", &request_id));
            } else {
                let (resp_tx, resp_rx) = oneshot::channel::<String>();
                response_rx_chan = resp_rx;
                response_tx_map.insert(request_id.clone(), resp_tx);
                compute_data_ref.request_id = request_id.clone();
            }
        } else {
            // could not get the map
            error!("could not lock request map");
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("text/html; charset=utf-8")
                .body("could not lock request map");
        }
    }

    let self_ip = std::env::var("HOST_IP").unwrap();
    compute_data_ref.source_service_ip = self_ip.clone();
    compute_data_ref.source_service_number = 0;
    compute_data_ref.source_service_port = PORT;
    compute_data_ref.frontend_ip = self_ip.clone();
    info!("input data: {:?}", &compute_data_ref);

    // emulate callback
    // tokio::spawn(async move {
    //     delay_for(Duration::from_millis(20000)).await;
    //     send_post_json_message(
    //         "http://127.0.0.1:12340/service/0/stop".into(),
    //         serde_json::to_string(&compute_data_ref.clone()).unwrap(),
    //     )
    //     .await
    // });

    let mut custom_headers: HeaderMap = HeaderMap::new();
    let service_to_call = 1;
    let header_map = req.headers();
    if header_map.contains_key("x-force-local") {
        let local_jumps = header_map.get("x-force-local").unwrap();
        let v: Value = serde_json::from_str(local_jumps.clone().to_str().unwrap()).unwrap();
        let local_jumps_vec = v.as_array().unwrap();
        for elem in local_jumps_vec {
            if elem.as_u64().unwrap() == service_to_call {
                custom_headers.insert(
                    HeaderName::from_static("x-stay-local"),
                    HeaderValue::from_str(format!("{}", service_to_call).as_str()).unwrap(),
                );
            }
        }
        custom_headers.insert(
            HeaderName::from_static("x-force-local"),
            local_jumps.clone(),
        );
    } else {
        custom_headers.insert(
            HeaderName::from_static("x-force-local"),
            HeaderValue::from_static("[]"),
        );
    }
    // async call the first service and wait
    let envoy_port = std::env::var("ENVOY_PORT").unwrap();
    let next_url = format!(
        "http://127.0.0.1:{}/service/{}/docompute",
        envoy_port, service_to_call
    );
    info!("[TRACE] [{}] [CALL_SERVICE] {}", &request_id, &next_url);
    tokio::spawn(send_post_json_message(
        next_url,
        serde_json::to_string(&compute_data_ref.clone()).unwrap(),
        custom_headers,
    ));

    let response = response_rx_chan.await;
    match response {
        // No error on response channel
        Ok(message) => {
            info!("[TRACE] [{}] [REQUEST_END_FRONTEND]", &request_id);
            return HttpResponse::build(StatusCode::OK)
                .content_type("application/json")
                .body(message);
        }
        Err(e) => {
            // Some error on response channel
            let err_msg = format!(
                "Error on response channel for request {}, Error: {:?}",
                &request_id, e
            );
            warn!("{}", &err_msg);
            return HttpResponse::build(StatusCode::BAD_REQUEST)
                .content_type("text/html; charset=utf-8")
                .body(err_msg);
        }
    }
}

async fn send_post_json_message(url: String, json_body: String, custom_headers: HeaderMap) {
    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .header("Content-Type", "application/json")
        .headers(custom_headers)
        .body(json_body)
        .send()
        .await;
    if res.is_ok() {
        let ret_body = res.unwrap().text().await;
        if ret_body.is_ok() {
            debug!("Response: {}", ret_body.unwrap());
        } else {
            warn!(
                "Unable to get reponse body for workflow invocation, {}",
                url
            );
        }
    } else {
        warn!("Error response from workflow invocation, {}", url);
    }
}

async fn dostop(req: HttpRequest, body: web::Bytes) -> impl Responder {
    // body is loaded, now we can deserialize json-rust
    let mut decoded_body = std::str::from_utf8(&body);
    if let Err(_e) = decoded_body {
        return HttpResponse::BadRequest().body("body decode failure");
    }

    let mut compute_data_ref: ComputeDataRef = match serde_json::from_str(&decoded_body.unwrap()) {
        Ok(decoded) => decoded,
        Err(e) => {
            let err_msg = format!("{:?}", e);
            warn!("{}", &err_msg);
            return HttpResponse::BadRequest().body(err_msg);
        }
    };

    let mut request_id: String;
    if compute_data_ref.request_id.len() > 0 {
        request_id = compute_data_ref.request_id.clone();
    } else {
        let err_msg = format!("Could not find a request id");
        warn!("{}", &err_msg);
        return HttpResponse::BadRequest().body(err_msg);
    }

    info!("[TRACE] [{}] [REQUEST_STOP_FRONTEND]", &request_id);
    let mut response_tx_chan: ResponseSendChannel;
    {
        if let Ok(mut response_tx_map) = req
            .app_data::<web::Data<Mutex<HashMap<String, ResponseSendChannel>>>>()
            .unwrap()
            .lock()
        {
            // got the map
            let chan = response_tx_map.remove(&request_id);
            match chan {
                Some(tx_chan) => {
                    response_tx_chan = tx_chan;
                }
                None => {
                    return HttpResponse::build(StatusCode::BAD_REQUEST)
                        .content_type("text/html; charset=utf-8")
                        .body(format!("No channel found for Request id {}", &request_id));
                }
            }
        } else {
            // could not get the map
            error!("could not lock request map");
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("text/html; charset=utf-8")
                .body("could not lock request map");
        }
    }
    let response_data = serde_json::to_string(&compute_data_ref.clone()).unwrap();
    response_tx_chan.send(response_data);
    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let start = std::time::Instant::now();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::Builder::from_default_env()
        .format(move |buf, rec| {
            let t = start.elapsed().as_secs_f32();
            //let t2 = Local::now().to_rfc2822();
            let t2 = Local::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            //let tid = thread::current().id();
            writeln!(
                buf,
                "[S0] [{}] [{:.06}] [{}] [{}:{}] {}",
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
    if std::env::var("ENVOY_PORT").is_err() {
        std::env::set_var("ENVOY_PORT", "80");
    }

    let host_port: String = format!("0.0.0.0:{}", PORT);
    info!(
        "Service name {} starting on {}, host_ip: {}, envoy_port: {}",
        0,
        &host_port,
        self_ip,
        std::env::var("ENVOY_PORT").unwrap()
    );

    let state: HashMap<String, ResponseSendChannel> = HashMap::new();
    let app_state = web::Data::new(Mutex::new(state));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            //.app_data(args.clone())
            .app_data(app_state.clone())
            .route("service/{id}/start", web::post().to(dostart))
            .route("service/{id}/stop", web::post().to(dostop))
    })
    .bind(host_port.as_str())?
    .workers(2)
    .run()
    .await
}
