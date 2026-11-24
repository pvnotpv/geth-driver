use tokio::time::{interval, Duration};

pub async fn check_online() -> Result<u8, Box<dyn std::error::Error>> {
    let mut timer = interval(Duration::from_secs(3));

    let mut retries = 0;

    loop {

        timer.tick().await;


        let response = crate::rpc_call("web3_clientVersion".to_string(), vec![], false).await;

        match response {
            Ok(res) => {

                let result = res.json::<crate::RpcResponse>().await?;

                if result.result.to_string().contains("Geth") {
                    println!("Geth Online, Starting the beacon...\n");
                    return Ok(1);
                }
            }
            Err(e) => {
                retries += 1;
                eprintln!("Unexpected error, try restarting GETH! {}, Retrying connection...", e);
                if retries >= 3 {
                    break
                }
            }
            
        }
    }

    return Ok(0);
}
