use crate::routes::notification::NotificationRouter;
use crate::routes::templating::TemplatingRouter;
use crate::routes::user::UserRouter;
use anyhow::Ok;
use axum::Router;
use clap::Parser;
use dotenv::dotenv;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use utilities::config::AppConfig;
use utilities::service_register::ServiceRegister;

mod request;
mod response;
mod routes;
mod utilities;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let config = Arc::new(AppConfig::parse());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Environment loaded and configuration parsed, initializing connection to services...");
    let app_host = &config.service_url;
    let app_port = &config.service_port;
    // let cors_origin = config.cors_origin.as_str();
    let app_url = format!("{}:{}", app_host, app_port);
    let service_register = ServiceRegister::new(config.clone()).await?;

    info!("migrations successfully ran, initializing axum server...");
    let app = Router::new()
        .nest(
            "/api/user",
            UserRouter::new_router(service_register.clone()),
        )
        .nest(
            "/api/templating",
            TemplatingRouter::new_router(service_register.clone()),
        )
        .nest(
            "/api/notification",
            NotificationRouter::new_router(service_register.clone()),
        )
        .layer(CorsLayer::permissive());

    axum::Server::bind(&app_url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
