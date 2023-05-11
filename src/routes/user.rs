use axum::{
    extract::{Path, State},
    headers::{authorization::Bearer, Authorization},
    routing::{get, post},
    Json, Router, TypedHeader,
};
use common::errors::{ServiceError, ServiceResult};
use urlencoding::{decode, encode};
use validator::Validate;

use crate::{
    email::SendEmailRequest,
    request::user::{
        LoginEndpointRequest, RefreshtokenEndpointRequest, RegisterEndpointRequest,
        UpdateEndpointRequest,
    },
    response::user::{RegisterUserEndpointResponse, UserEndpointResponse},
    templating::{compose_request::InputValue, ComposeRequest},
    user::{
        update_request::UpdateFields, GetUserRequest, LoginRequest, RefreshTokenRequest,
        RegisterRequest, UpdateRequest, VerifyRegistrationRequest,
    },
    utilities::{
        service_register::ServiceRegister,
        states::{
            email_service::StateEmailService, templating_service::StateTemplatingService,
            token_service::StateTokenService, user_service::StateUserService,
        },
    },
};
use tracing::info;

pub struct UserRouter;

impl UserRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route(
                "/",
                get(UserRouter::get_current_user_endpoint)
                    .post(UserRouter::register_user_endpoint)
                    .put(UserRouter::update_user_endpoint),
            )
            .route("/login", post(UserRouter::login_user_endpoint))
            .route("/refresh", post(UserRouter::refresh_token_endpoint))
            .route(
                "/verify/:token",
                get(UserRouter::verify_registration_endpoint),
            )
            .with_state(service_register)
    }

    pub async fn register_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(mut email_service): State<StateEmailService>,
        State(mut templating_service): State<StateTemplatingService>,
        State(token_service): State<StateTokenService>,
        Json(request): Json<RegisterEndpointRequest>,
    ) -> ServiceResult<Json<RegisterUserEndpointResponse>> {
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

        let verify_token =
            encode(&token_service.create_verify_registration_token(user.id)?).into_owned();

        let compose_request: ComposeRequest = ComposeRequest {
            name: "registration".to_string(),
            input_values: vec![
                InputValue {
                    name: "username".to_string(),
                    value: user.username.clone(),
                },
                InputValue {
                    name: "verification_token".to_string(),
                    value: verify_token.clone(),
                },
            ],
        };

        let email_template = templating_service
            .compose(compose_request)
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext(
                    "Unable to compose email for verification".to_string(),
                )
            })?
            .into_inner();

        let send_email_request: SendEmailRequest = SendEmailRequest {
            email: user.email.clone(),
            title: "Thank you for registering".to_string(),
            body: email_template.result,
        };

        email_service
            .send_email(send_email_request)
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext(
                    "Failed to send email to verify user".to_string(),
                )
            })?;

        Ok(Json(RegisterUserEndpointResponse {
            username: user.username,
            email: user.email,
            verify_token,
        }))
    }

    pub async fn verify_registration_endpoint(
        State(mut user_service): State<StateUserService>,
        State(mut email_service): State<StateEmailService>,
        State(mut templating_service): State<StateTemplatingService>,
        State(token_service): State<StateTokenService>,
        Path(token): Path<String>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        info!("Verify Registration Endpoint");
        let verification_token = decode(&token)
            .map_err(|_| {
                ServiceError::BadRequest(
                    "Unable to decode registration verification token".to_string(),
                )
            })?
            .into_owned();
        let user_id = token_service.extract_verify_registration_token(&verification_token)?;

        let verify_registration_request: VerifyRegistrationRequest =
            VerifyRegistrationRequest { id: user_id };

        let user = user_service
            .verify_registration(verify_registration_request)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        let compose_request: ComposeRequest = ComposeRequest {
            name: "verified".to_string(),
            input_values: vec![InputValue {
                name: "username".to_string(),
                value: user.username.clone(),
            }],
        };

        let email_template = templating_service
            .compose(compose_request)
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext(
                    "Unable to compose email confirming verification".to_string(),
                )
            })?
            .into_inner();

        let send_email_request: SendEmailRequest = SendEmailRequest {
            email: user.email.clone(),
            title: "You are now verified".to_string(),
            body: email_template.result,
        };

        email_service
            .send_email(send_email_request)
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext(
                    "Failed to send email to confirm verification".to_string(),
                )
            })?;

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
        let id = token_service.get_user_id_from_bearer_token(authorization.token())?;
        request.validate()?;
        let refresh_request: RefreshTokenRequest = if let Some(token) = request.token {
            Ok(RefreshTokenRequest { id, token })
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
        let id = token_service.get_user_id_from_bearer_token(authorization.token())?;

        info!("Obtained authorization, obtaining response from User service...");
        let user = user_service
            .get(GetUserRequest { id })
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        info!("Returning response!");
        Ok(Json(UserEndpointResponse::from_user_response(user, None)))
    }

    pub async fn update_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Json(request): Json<UpdateEndpointRequest>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        info!("Update User Endpoint, obtaining authorization...");
        let id = token_service.get_user_id_from_bearer_token(authorization.token())?;

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

        info!("Returning response!");
        Ok(Json(UserEndpointResponse::from_user_response(user, None)))
    }
}
