use crate::Cli;
use crate::StringError;
use log::*;
use log::*;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::{thread, time};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::task;
use tokio::task::JoinHandle;

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

pub type ResponseChannelSender = oneshot::Sender<(bool, String)>;
pub type ResponseChannelReceiver = oneshot::Receiver<(bool, String)>;

#[derive(Debug)]
pub struct ComputeActor {
    pub response_channel_rx: ResponseChannelReceiver,
    pub join_handle: JoinHandle<Result<String, Box<dyn std::error::Error + Send + Sync>>>,
}

impl ComputeActor {
    pub fn spawn(compute_data_ref: ComputeDataRef, args: Cli, local_jumps: String) -> Self {
        // these channels are unused
        let (mut response_channel_tx, mut response_channel_rx) =
            oneshot::channel::<(bool, String)>();

        let join_handle = task::spawn_blocking(move || {
            compute_actor_loop(
                response_channel_tx,
                compute_data_ref,
                args.clone(),
                local_jumps.clone(),
            )
        });

        ComputeActor {
            response_channel_rx,
            join_handle,
        }
    }
}

fn compute_actor_loop(
    mut response_channel_tx: ResponseChannelSender,
    mut compute_data_ref: ComputeDataRef,
    cli_args: Cli,
    local_jumps: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "[TRACE] [{}] [COMPUTE_START] [{}]",
        &compute_data_ref.request_id, &cli_args.servicetype
    );
    let result = match cli_args.servicetype.as_str() {
        "produce" => produce_data(&compute_data_ref),
        "process" => process_data(&compute_data_ref),
        _ => Err("unknown service type".into()),
        // _ => Err(Box::new(StringError {
        //     err_msg: format!("Unknown action"),
        // })),
    };
    info!(
        "[TRACE] [{}] [COMPUTE_STOP] [{}]",
        &compute_data_ref.request_id, &cli_args.servicetype
    );

    if result.is_err() {
        warn!("Error: {:?}", result.as_ref().unwrap_err());
        return result;
    } else {
        // call the next function, if any
        let mut custom_headers: HeaderMap = HeaderMap::new();
        compute_data_ref.source_service_ip = std::env::var("HOST_IP").unwrap().clone();
        compute_data_ref.source_service_number = cli_args.servicenumber;
        compute_data_ref.source_service_port = cli_args.port;
        compute_data_ref
            .service_ips
            .push_str(format!("{};", std::env::var("HOST_IP").unwrap()).as_str());

        let next_service_data: ComputeDataRef;
        if cli_args.servicenumber == compute_data_ref.terminal_service_number {
            // call the frontend to return response
            let envoy_port = std::env::var("ENVOY_PORT").unwrap();
            let next_service_url = format!(
                "http://{}:{}/service/0/stop",
                &compute_data_ref.frontend_ip, envoy_port
            );
            info!(
                "[TRACE] [{}] [CALL_FRONTEND] {}",
                &compute_data_ref.request_id, next_service_url
            );
            next_service_data = ComputeDataRef {
                request_id: compute_data_ref.request_id.clone(),
                filename: result.unwrap(),
                data: "".to_string(),
                ..compute_data_ref
            };
            send_post_json_message(
                next_service_url,
                serde_json::to_string(&next_service_data)?,
                custom_headers,
            );
        } else {
            // call the next function
            // The next service needs details of the previous service
            //      the service number of the previous service
            //      ip address of the previous service
            //  the ip address of the frontend where to return a response, if this is the last servie
            if cli_args.servicetype.as_str() == "produce" && compute_data_ref.pass_data == true {
                next_service_data = ComputeDataRef {
                    request_id: compute_data_ref.request_id.clone(),
                    data: result.unwrap(),
                    ..compute_data_ref.clone()
                };
            } else {
                next_service_data = ComputeDataRef {
                    request_id: compute_data_ref.request_id.clone(),
                    filename: result.unwrap(),
                    data: "".to_string(),
                    ..compute_data_ref
                };
            }

            let service_to_call = compute_data_ref.source_service_number + 1;
            let v: Value = serde_json::from_str(local_jumps.clone().as_str()).unwrap();
            let local_jumps_vec = v.as_array().unwrap();
            for elem in local_jumps_vec {
                if elem.as_u64().unwrap() == service_to_call as u64 {
                    custom_headers.insert(
                        HeaderName::from_static("x-stay-local"),
                        HeaderValue::from_str(format!("{}", service_to_call).as_str()).unwrap(),
                    );
                }
            }
            custom_headers.insert(
                HeaderName::from_static("x-force-local"),
                HeaderValue::from_str(&local_jumps).unwrap(),
            );

            let envoy_port = std::env::var("ENVOY_PORT").unwrap();
            let next_service_url = format!(
                "http://127.0.0.1:{}/service/{}/docompute",
                envoy_port, service_to_call
            );

            // HACK!!
            //let next_service_url = format!("http://127.0.0.1:12342/service/2/docompute");

            info!(
                "[TRACE] [{}] [CALL_SERVICE_START] {}",
                &compute_data_ref.request_id, &next_service_url
            );
            send_post_json_message(
                next_service_url.clone(),
                serde_json::to_string(&next_service_data)?,
                custom_headers,
            );
            info!(
                "[TRACE] [{}] [CALL_SERVICE_END] {}",
                &compute_data_ref.request_id, &next_service_url
            );
        }
    }
    Ok("ok".into())
}

fn process_data(
    compute_data_ref: &ComputeDataRef,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if compute_data_ref.pass_data == true && compute_data_ref.data.len() > 0 {
        info!(
            "[TRACE] [{}] [DATA_PROCESS_START] [process]",
            &compute_data_ref.request_id
        );
        info!("processing data of length: {}", compute_data_ref.data.len());
        for j in 0..compute_data_ref.loops {
            let mut new_string: String = String::new();
            for (i, c) in compute_data_ref.data.chars().enumerate() {
                // do something with character 'c' and index 'i'
                if c == 'a' {
                    new_string.push('b');
                } else {
                    new_string.push(c);
                }
            }
        }
        info!(
            "[TRACE] [{}] [DATA_PROCESS_END] [process] data len {}",
            &compute_data_ref.request_id,
            compute_data_ref.data.len()
        );
        return Ok(compute_data_ref.filename.clone());
    }
    info!(
        "[TRACE] [{}] [FILE_READ_START] [process] file: {}",
        &compute_data_ref.request_id, &compute_data_ref.filename
    );
    let contents: String = match fs::read_to_string(&compute_data_ref.filename) {
        Ok(contents) => contents,
        Err(e) => {
            let envoy_port = std::env::var("ENVOY_PORT").unwrap();
            let service_url = format!(
                "http://{}:{}/service/{}/getdata",
                &compute_data_ref.source_service_ip,
                envoy_port,
                compute_data_ref.source_service_number,
            );
            info!(
                "[TRACE] [{}] [FILE_FETCH_START] [process] {}",
                &compute_data_ref.request_id, &service_url
            );
            match get_file_from_remote_host(service_url.clone(), compute_data_ref.filename.clone())
            {
                Ok(contents) => {
                    info!(
                        "[TRACE] [{}] [FILE_FETCH_END] [process] {}",
                        &compute_data_ref.request_id, &service_url
                    );
                    let path = Path::new(&compute_data_ref.filename);
                    let display = path.display();
                    let mut file = match File::create(&path) {
                        Err(why) => {
                            return Err(Box::new(StringError {
                                err_msg: format!("couldn't create {}: {}", display, why),
                            }))
                        }
                        Ok(file) => file,
                    };
                    match file.write_all(contents.as_bytes()) {
                        Err(why) => {
                            return Err(Box::new(StringError {
                                err_msg: format!("couldn't write to {}: {}", display, why),
                            }))
                        }
                        Ok(_) => {
                            info!(
                                "Finish fetching file {}, from {}",
                                &compute_data_ref.filename, &service_url
                            );
                            fs::read_to_string(&path)?
                        }
                    }
                }
                Err(()) => {
                    return Err(Box::new(StringError {
                        err_msg: format!(
                            "couldn't create fetch file {}, from {}",
                            compute_data_ref.filename.clone(),
                            service_url
                        ),
                    }))
                }
            }
        }
    };
    info!(
        "[TRACE] [{}] [DATA_PROCESS_START] [process] data len {}",
        &compute_data_ref.request_id,
        contents.len()
    );
    info!("processing contents of length: {}", contents.len());
    for j in 0..compute_data_ref.loops {
        let mut new_string: String = String::new();
        for (i, c) in contents.chars().enumerate() {
            // do something with character 'c' and index 'i'
            if c == 'a' {
                new_string.push('b');
            } else {
                new_string.push(c);
            }
        }
    }
    info!(
        "[TRACE] [{}] [DATA_PROCESS_END] [process] data len {}",
        &compute_data_ref.request_id,
        contents.len()
    );
    info!("finished processing contents of length: {}", contents.len());
    Ok(compute_data_ref.filename.clone())
}

fn produce_data(
    compute_data_ref: &ComputeDataRef,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Open a file in write-only mode, returns `io::Result<File>`
    info!(
        "[TRACE] [{}] [DATA_PRODUCE_START] [produce] data len {}",
        &compute_data_ref.request_id, &compute_data_ref.size
    );
    let mut filedata: String = String::new();
    let num_iters = compute_data_ref.size / 10;
    for i in 0..num_iters {
        filedata.push_str("abcdefghij");
    }

    if compute_data_ref.pass_data == true {
        info!(
            "[TRACE] [{}] [DATA_PRODUCE_END] [produce] raw data len {}, file: no-file",
            &compute_data_ref.request_id, &compute_data_ref.size
        );
        return Ok(filedata);
    }
    let filepath = format!("/file_{}.txt", &compute_data_ref.request_id);
    let path = Path::new(&filepath);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => {
            return Err(Box::new(StringError {
                err_msg: format!("couldn't create {}: {}", display, why),
            }))
        }
        Ok(file) => file,
    };

    match file.write_all(filedata.as_bytes()) {
        Err(why) => {
            return Err(Box::new(StringError {
                err_msg: format!("couldn't write to {}: {}", display, why),
            }))
        }
        Ok(_) => {
            info!(
                "[TRACE] [{}] [DATA_PRODUCE_END] [produce] data len {}, file: {}",
                &compute_data_ref.request_id, &compute_data_ref.size, &filepath
            );
            return Ok(filepath);
        }
    }
}

pub fn send_post_json_message(url: String, json_body: String, custom_headers: HeaderMap) {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(&url)
        .header("Content-Type", "application/json")
        .headers(custom_headers)
        .body(json_body)
        .send();
    if res.is_ok() {
        let ret_body = res.unwrap().text();
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

pub fn get_file_from_remote_host(url: String, filepath: String) -> Result<String, ()> {
    info!("Start fetching file {}, from {}", &filepath, &url);
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(&url)
        .header("x-stay-local", "-")
        .body(filepath.clone())
        .send();
    if res.is_ok() {
        let res: reqwest::blocking::Response = res.unwrap();
        if res.status().is_success() {
            let ret_body = res.text();
            if ret_body.is_ok() {
                //debug!("Response: {}", ret_body.unwrap());
                Ok(ret_body.unwrap())
            } else {
                warn!("Unable to get reponse body file get request, {}", url);
                Err(())
            }
        } else {
            warn!(
                "Non 200 status code {:?} returned from {}",
                res.status(),
                url
            );
            Err(())
        }
    } else {
        warn!("Error response from, {}", url);
        Err(())
    }
}
