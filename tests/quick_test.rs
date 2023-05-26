#![allow(unused)]

use serde_json::json;
use zero2axum::configuration::get_configuration;

#[tokio::test]
#[cfg_attr(not(feature = "ci"), ignore)]
async fn quick_test() -> color_eyre::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!(
        "http://{}:{}",
        configuration.application.host, configuration.application.port
    );
    let hc = httpc_test::new_client(&address)?;

    // hello handler tests
    hc.do_get("/").await?.print().await?;
    hc.do_get("/?name=Harvey").await?.print().await?;

    // health_check handler tests
    hc.do_get("/health_check").await?.print().await?;

    Ok(())
}
