use anyhow::Ok;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::routing::{get, post, put};
use axum::{Extension, Json, Router};
use clap::Parser;
use dotenv::dotenv;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use data::config::AppConfig;
use data::connection_pool::ServiceConnectionManager;
use domain::errors::ServiceResult;
use domain::request::{LoginUserRequest, RegisterUserRequest, UpdateUserRequest};
use domain::response::UserAuthenticationResponse;
use domain::service_register::ServiceRegister;
use extractor::required_authentication::RequiredAuthentication;
use extractor::validation::ValidationExtractor;
use service::user::DynUserServiceTrait;

use crate::domain::errors::ServiceError;

mod data;
mod domain;
mod extractor;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().expect("Failed to read .env file, please add a .env file to the project root");

    let config = Arc::new(AppConfig::parse());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Environment loaded and configuration parsed, initializing Postgres connection and running migrations...");
    let pg_pool = ServiceConnectionManager::new_pool(&config.database_url, config.run_migrations)
        .await
        .expect("could not initialize the database connection pool");

    let app_host = &config.service_url;
    let app_port = &config.service_port;
    let cors_origin = config.cors_origin.as_str();
    let app_url = format!("{}:{}", app_host, app_port);
    let service_register = ServiceRegister::new(pg_pool, config.clone());

    if config.seed {
        info!("seeding enabled, creating test data...");
        todo!("create seeding code here");
    }

    info!("migrations successfully ran, initializing axum server...");

    let app = Router::new()
        .route("/user", post(register_user_endpoint))
        .route("/user/login", post(login_user_endpoint))
        .route("/user", get(get_current_user_endpoint))
        .route("/user", put(update_user_endpoint))
        .layer(Extension(service_register.user_service))
        .layer(Extension(service_register.token_service))
        .layer(
            CorsLayer::new()
                // .allow_origin(any())
                .allow_origin(cors_origin.parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(vec![CONTENT_TYPE, AUTHORIZATION]),
        );

    axum::Server::bind(&app_url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

pub async fn register_user_endpoint(
    ValidationExtractor(request): ValidationExtractor<RegisterUserRequest>,
    Extension(user_service): Extension<DynUserServiceTrait>,
) -> ServiceResult<Json<UserAuthenticationResponse>> {
    info!(
        "received request to create user {:?}/{:?}",
        request.user.email.as_ref().unwrap(),
        request.user.username.as_ref().unwrap()
    );

    let created_user = user_service.register_user(request.user).await?;

    Ok(Json(UserAuthenticationResponse { user: created_user }))
        .map_err(|err| ServiceError::AnyhowError(err))
}

pub async fn login_user_endpoint(
    ValidationExtractor(request): ValidationExtractor<LoginUserRequest>,
    Extension(user_service): Extension<DynUserServiceTrait>,
) -> ServiceResult<Json<UserAuthenticationResponse>> {
    info!(
        "received request to login user {:?}",
        request.user.email.as_ref().unwrap()
    );

    let logged_in_user = user_service.login_user(request.user).await?;

    Ok(Json(UserAuthenticationResponse {
        user: logged_in_user,
    }))
    .map_err(|err| ServiceError::AnyhowError(err))
}

pub async fn get_current_user_endpoint(
    RequiredAuthentication(user_id): RequiredAuthentication,
    Extension(user_service): Extension<DynUserServiceTrait>,
) -> ServiceResult<Json<UserAuthenticationResponse>> {
    info!("received request to retrieve current user");

    let current_user = user_service.get_current_user(user_id).await?;

    Ok(Json(UserAuthenticationResponse { user: current_user }))
        .map_err(|err| ServiceError::AnyhowError(err))
}

pub async fn update_user_endpoint(
    RequiredAuthentication(user_id): RequiredAuthentication,
    Extension(user_service): Extension<DynUserServiceTrait>,
    Json(request): Json<UpdateUserRequest>,
) -> ServiceResult<Json<UserAuthenticationResponse>> {
    info!("received request to update user {:?}", user_id);

    let updated_user = user_service.updated_user(user_id, request.user).await?;

    Ok(Json(UserAuthenticationResponse { user: updated_user }))
        .map_err(|err| ServiceError::AnyhowError(err))
}
