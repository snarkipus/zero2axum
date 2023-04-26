#![allow(unused)]

use serde_json::json;

#[tokio::test]
#[cfg_attr(feature = "ci", ignore)]
async fn quick_test() -> color_eyre::Result<()> {
    let hc = httpc_test::new_client("http://127.0.0.1:3000")?;

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
        .post(&format!("{}/subscribe", "http://127.0.0.1:3000"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    println!("\n=== Response for POST http://127.0.0.1:3000/subscribe");
    println!("=> Status \t : {}", response.status());
    println!("=> Headers \t : {:#?}", response.headers());
    println!("=> Response Body : {:#?}", response.text().await?);

    Ok(())
}
