use std::net::TcpListener;

use axum::{
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use hyper::{server::conn::AddrIncoming, Body};
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

use crate::{configuration::*, routes};

pub async fn run(
    listener: TcpListener,
) -> Result<Server<AddrIncoming, IntoMakeService<Router<(), Body>>>, std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = format!(
        "{}:{}",
        configuration.database.host, configuration.database.port
    );

    let db = Surreal::new::<Ws>(connection_string)
        .await
        .expect("Failed to connect to SurrealDB.");

    db.signin(Root {
        username: &configuration.database.username,
        password: &configuration.database.password,
    })
    .await
    .expect("Failed to signin.");

    db.use_ns("default")
        .use_db("newsletter")
        .await
        .expect("Failed to use database.");

    let app = Router::new()
        .route("/", get(routes::handler_hello))
        .route("/health_check", get(routes::handler_health_check))
        .route("/subscribe", post(routes::handler_subscribe))
        .with_state(db);

    let server = Server::from_tcp(listener)
        .unwrap_or_else(|e| {
            panic!("Failed to bind random port: {}", e);
        })
        .serve(app.into_make_service());
    Ok(server)
}
