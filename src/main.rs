#![allow(unused)]

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    
    // 
    let _guard = sentry::init((
        std::env::var("SENTRY_DSN").expect("$SENTRY_DSN must be set"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    // // region: --- Start Server
    // let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // println!("Listening on http://{addr}");
    // axum::Server::bind(&addr)
    //     .serve(routes_all.into_make_service())
    //     .await
    //     .unwrap();
    // // endregion: --- Start Server

    panic!("Everything is on fire!");
    Ok(())
}
