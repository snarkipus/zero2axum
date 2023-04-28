#![allow(unused)]

use serde_json::json;
use zero2axum::configuration::get_configuration;

#[tokio::test]
#[cfg_attr(feature = "ci", ignore)]
async fn quick_test() -> color_eyre::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("http://127.0.0.1:{}", configuration.application_port);
    let hc = httpc_test::new_client(&address)?;

    // hello handler tests
    hc.do_get("/").await?.print().await?;
    hc.do_get("/?name=Harvey").await?.print().await?;

    // health_check handler tests
    hc.do_get("/health_check").await?.print().await?;

    // subscribe handler tests -- application/x-www-form-urlencoded
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // let body = "";
    // let body = "name=le%20guin";
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/subscribe", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    println!("\n=== Response for POST http://{}/subscribe", &address);
    println!("=> Status \t : {}", response.status());
    println!("=> Headers \t : {:#?}", response.headers());
    println!("=> Response Body : {:#?}", response.text().await?);
    println!("===");

    Ok(())
}
