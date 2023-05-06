use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    routing::{get, post},
    Json, Router, TypedHeader,
};
use common::errors::{ServiceError, ServiceResult};
use validator::Validate;

use crate::{
    request::user::{
        LoginEndpointRequest, RefreshtokenEndpointRequest, RegisterEndpointRequest,
        UpdateEndpointRequest,
    },
    response::user::UserEndpointResponse,
    user::{
        update_request::UpdateFields, GetUserRequest, LoginRequest, RegisterRequest, UpdateRequest,
        UpdateTokenRequest,
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
            .route("/user/refresh", post(UserRouter::refresh_token_endpoint))
            .with_state(service_register)
    }

    pub async fn register_user_endpoint(
        State(mut user_service): State<StateUserService>,
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

        Ok(Json(UserEndpointResponse::from_user_response(user, None)))
    }

    pub async fn login_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        Json(request): Json<LoginEndpointRequest>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        info!("Login User Endpoint, creating service request...");
        request.validate()?;
        let login_request: LoginRequest =
            if let (Some(email), Some(password)) = (request.email, request.password) {
                Ok(LoginRequest { email, password })
            } else {
                Err(ServiceError::BadRequest(
                    "Missing parameters in the request".to_string(),
                ))
            }?;

        info!("Created Service Request, obtaining response from User service...");
        let user = user_service
            .login(login_request)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        info!("Obtained response from service, creating token...");
        let token = token_service.create_token(user.id, &user.email)?;

        info!("Token created, returning response!");
        Ok(Json(UserEndpointResponse::from_user_response(
            user,
            Some(token),
        )))
    }

    pub async fn refresh_token_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Json(request): Json<RefreshtokenEndpointRequest>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        info!("Refresh token Endpoint, creating service request...");
        let id = token_service.get_user_id_from_token(authorization.token())?;
        request.validate()?;
        let refresh_request: UpdateTokenRequest = if let Some(token) = request.token {
            Ok(UpdateTokenRequest { id, token })
        } else {
            Err(ServiceError::BadRequest(
                "Missing parameters in the request".to_string(),
            ))
        }?;

        info!("Created Service Request, obtaining response from User service...");
        let user = user_service
            .refresh_token(refresh_request)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        info!("Obtained response from service, creating token...");
        let token = token_service.create_token(user.id, &user.email)?;

        info!("Token created, returning response!");
        Ok(Json(UserEndpointResponse::from_user_response(
            user,
            Some(token),
        )))
    }

    pub async fn get_current_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        info!("Get User Endpoint, obtaining authorization...");
        let id = token_service.get_user_id_from_token(authorization.token())?;

        info!("Obtained authorization, obtaining response from User service...");
        let user = user_service
            .get(GetUserRequest { id })
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        info!("Token created, returning response!");
        Ok(Json(UserEndpointResponse::from_user_response(user, None)))
    }

    pub async fn update_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Json(request): Json<UpdateEndpointRequest>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        info!("Update User Endpoint, obtaining authorization...");
        let id = token_service.get_user_id_from_token(authorization.token())?;

        info!("Obtained authorization, obtaining response from User service...");
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

        info!("Token created, returning response!");
        Ok(Json(UserEndpointResponse::from_user_response(user, None)))
    }
}
