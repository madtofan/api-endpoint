use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    routing::{get, post},
    Json, Router, TypedHeader,
};
use common::errors::{ServiceError, ServiceResult};
use validator::Validate;

use crate::{
    request::user::{LoginEndpointRequest, RegisterEndpointRequest, UpdateEndpointRequest},
    response::user::UserEndpointResponse,
    user::{
        update_request::UpdateFields, GetUserRequest, LoginRequest, RegisterRequest, UpdateRequest,
    },
    utilities::{
        service_register::ServiceRegister,
        states::{token_service::StateTokenService, user_service::StateUserService},
    },
};
use tracing::info;

pub struct UserRouter;

impl UserRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route(
                "/user",
                get(UserRouter::get_current_user_endpoint)
                    .post(UserRouter::register_user_endpoint)
                    .put(UserRouter::update_user_endpoint),
            )
            .route("/user/login", post(UserRouter::login_user_endpoint))
            .with_state(service_register)
    }

    pub async fn register_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        Json(request): Json<RegisterEndpointRequest>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        info!("Register User Endpoint");

        request.validate()?;
        let register_request: RegisterRequest =
            if let (Some(username), Some(email), Some(password)) =
                (request.username, request.email, request.password)
            {
                Ok(RegisterRequest {
                    username,
                    email,
                    password,
                })
            } else {
                Err(ServiceError::BadRequest(
                    "Missing parameters in the request".to_string(),
                ))
            }?;

        let user = user_service
            .register(register_request)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        let token = token_service.create_token(user.id, &user.email)?;

        Ok(Json(UserEndpointResponse::from_user_response(user, token)))
    }

    pub async fn login_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        Json(request): Json<LoginEndpointRequest>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        let print = format!(
            "Login User Endpoint, creating service request: {:?}",
            request
        );
        info!(print);

        request.validate()?;
        let login_request: LoginRequest =
            if let (Some(email), Some(password)) = (request.email, request.password) {
                Ok(LoginRequest { email, password })
            } else {
                Err(ServiceError::BadRequest(
                    "Missing parameters in the request".to_string(),
                ))
            }?;

        info!("Created Service Request, obtaining response");
        let user = user_service
            .login(login_request)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        info!("Obtained response from service, creating token");
        let token = token_service.create_token(user.id, &user.email)?;

        Ok(Json(UserEndpointResponse::from_user_response(user, token)))
    }

    pub async fn get_current_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        info!("Get User Endpoint");

        let id = token_service.get_user_id_from_token(authorization.token())?;

        let user = user_service
            .get(GetUserRequest { id })
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();
        let token = token_service.create_token(user.id, &user.email)?;

        Ok(Json(UserEndpointResponse::from_user_response(user, token)))
    }

    pub async fn update_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Json(request): Json<UpdateEndpointRequest>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        info!("Update User Endpoint");

        let id = token_service.get_user_id_from_token(authorization.token())?;

        let user = user_service
            .update(UpdateRequest {
                id,
                fields: Some(UpdateFields {
                    email: request.email,
                    username: request.username,
                    password: request.password,
                    bio: request.bio,
                    image: request.image,
                }),
            })
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        let token = token_service.create_token(user.id, &user.email)?;

        Ok(Json(UserEndpointResponse::from_user_response(user, token)))
    }
}
