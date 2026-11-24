use reqwest::{Client};
use serde::{Deserialize, Serialize};
use clap::{self, Arg};
use tokio::time::{interval, Duration};
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::{Value};
use serde_json::json;
use std::thread;
use std::time::{Duration as dur};

mod jwt;
mod online;

const BASE_HASH: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";

const BANNER: &str = r#"

   ██████╗ ███████╗████████╗██╗  ██╗
  ██╔════╝ ██╔════╝╚══██╔══╝██║  ██║
  ██║  ███╗█████╗     ██║   ███████║
  ██║   ██║██╔══╝     ██║   ██╔══██║
  ╚██████╔╝███████╗   ██║   ██║  ██║
   ╚═════╝ ╚══════╝   ╚═╝   ╚═╝  ╚═╝

             D R I V E R
"#;

#[derive(Serialize)]
struct Request {
    jsonrpc: String,
    method: String,
    params: Vec<serde_json::Value>,
    id: u32,
}

#[derive(Serialize)]
struct UpdateParams {
    timestamp: String, 
    prevRandao: String, 
    suggestedFeeRecipient: String, 
    withdrawals: Vec<serde_json::Value>, 
    parentBeaconBlockRoot: String
}

#[derive(Serialize)]
struct ForkChoiceState {
    headBlockHash: String, 
    safeBlockHash: String, 
    finalizedBlockHash: String
}


#[derive(Deserialize, Debug)]
struct RpcResponse {
    result: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("{}", BANNER);
    let cmd = clap::Command::new("geth-driver")
        .bin_name("geth")
        .about("Ultra-lightweight consensus client to drive geth.")
        .arg_required_else_help(false)
        .subcommand(
            clap::Command::new("hey")
                .about("Say hey")
                .arg(
                    Arg::new("debug")
                        .short('d')
                        .help("Debug message")
                        .id("dd")
                        .required(true),
                ),
        );

    let matches = cmd.get_matches();

    match matches.subcommand() {
        _ => {
            let response = online::check_online().await?;
            if response == 1 {
                start().await?;
            } else if response == 0 {
                println!("nope")
            }
        }
    }

    Ok(())
}



async fn rpc_call(method: String, params: Vec<serde_json::Value>, authen: bool) -> Result<reqwest::Response, Box<dyn std::error::Error>> {

    let request = Request {
            jsonrpc: "2.0".to_string(),
            method: method,
            params: params,
            id: 1,
        };

    let client = Client::new();

    let token = jwt::generate_jwt("jwt.hex")?;

    let port = if authen { 
        8551 
    } else { 
        8545 
    };

    let url = format!("http://localhost:{}", port);

    client.post(url)
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .json(&request)
        .send()
        .await.map_err(Into::into)
}

async fn fork_choice_request(hash: String, update: bool) -> Result<reqwest::Response, Box<dyn std::error::Error>> {

    println!("=>> Change fork head request, New Head: {}", hash);

    let fee_recipient = "0x1234567890abcdef1234567890abcdef12345678";
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let timestamp = format!("0x{:x}", now);

    let secret = format!("0x{}", jwt::generate_secret());

    let update_params = UpdateParams {
        timestamp: timestamp, 
        prevRandao: secret, 
        suggestedFeeRecipient: fee_recipient.to_string(), 
        withdrawals: Vec::new(), 
        parentBeaconBlockRoot: BASE_HASH.to_string().clone()
    };

    let update_json = serde_json::to_value(update_params)?;

    let forkchoice = ForkChoiceState {
        headBlockHash: hash.to_string(), 
        safeBlockHash: hash.to_string(), 
        finalizedBlockHash: BASE_HASH.to_string().clone()
    };

    let forkchoise_json = serde_json::to_value(forkchoice)?;

    let mut params = vec![];

    if update {
        params.push(forkchoise_json);
        params.push(update_json);
    } else {
        params.push(forkchoise_json);
        params.push(serde_json::Value::Null);
    }

    let response = rpc_call("engine_forkchoiceUpdatedV3".to_string(), params, true).await?;

    Ok(response)


}

async fn get_current_block() -> Result<reqwest::Response, Box<dyn std::error::Error>> {

    let response = rpc_call("eth_getBlockByNumber".to_string(), vec![json!("latest"), json!(false)], false).await?;
    Ok(response)

}


async fn start() -> Result<(), Box<dyn std::error::Error>> {

    let mut timer = interval(Duration::from_secs(3));

    loop {

        timer.tick().await;

        // Get current head
        let response = get_current_block().await?;
        let result: RpcResponse = response.json::<RpcResponse>().await?;

        let hex_str = result.result["number"].as_str().unwrap().trim_start_matches("0x");
        let decimal = u32::from_str_radix(hex_str, 16).unwrap();
        println!("=>> Current Block: {:?}", decimal);

        let hash = result.result["hash"].as_str().unwrap();

        // Trigger block building
        let result = fork_choice_request(hash.to_string(), true).await?;
        let body: Value = result.json().await?;

        // Get payload id
        let payload_id = body["result"]["payloadId"].as_str().ok_or("Missing payloadId")?;
        println!("=>> New Payload Received: {}", payload_id);

        let params = vec![json!(payload_id)];

        tokio::time::sleep(Duration::from_secs(1)).await;

        let response = rpc_call("engine_getPayloadV3".to_string(), params, true).await?;
        let body: Value = response.json().await?;

        let execution_payload = body["result"]["executionPayload"].clone();

        let new_hash = execution_payload["blockHash"].as_str().ok_or("add erro")?;

        let params = vec![execution_payload.clone(), json!([]), json!(BASE_HASH.to_string())];

        // Execute the payload
        let response = rpc_call("engine_newPayloadV3".to_string(), params, true).await?;
        let body: Value = response.json().await?;

        println!("=>> Executed Payload, Latest block hash: {:?}",body["result"]["latestValidHash"].as_str().unwrap());

        // Update the head
        fork_choice_request(new_hash.to_string(), false).await?;

        println!("");

    }


}
