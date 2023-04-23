#![allow(unused)]

use std::sync::Arc;

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

    Ok(())
}
