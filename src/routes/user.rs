use axum::{
    extract::{Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    routing::{delete, get, post},
    Json, Router, TypedHeader,
};
use madtofan_microservice_common::{
    email::SendEmailRequest,
    errors::{ServiceError, ServiceResult},
    templating::{compose_request::InputValue, ComposeRequest},
    user::{
        update_request::UpdateFields, GetListRequest, GetUserRequest, LoginRequest,
        RefreshTokenRequest, RegisterRequest, Role, RolesPermissionsRequest, UpdateRequest,
        VerifyRegistrationRequest, VerifyTokenRequest,
    },
};
use urlencoding::{decode, encode};
use validator::Validate;

use crate::{
    request::{
        user::{
            AddRolePermissionRequest, AuthorizeRevokeRolePermissionRequest, LoginEndpointRequest,
            RefreshtokenEndpointRequest, RegisterEndpointRequest, UpdateEndpointRequest,
        },
        Pagination,
    },
    response::{
        user::{
            ObtainTokenResponse, PermissionsListResponse, RegisterUserEndpointResponse,
            RolesListResponse, UserEndpointResponse,
        },
        StatusMessageResponse,
    },
    utilities::{
        constants::PAGINATION_SIZE,
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
            .route(
                "/roles",
                get(UserRouter::get_roles).post(UserRouter::add_role),
            )
            .route("/roles/:role_name", delete(UserRouter::delete_role))
            .route(
                "/permissions",
                get(UserRouter::get_permissions).post(UserRouter::add_permission),
            )
            .route(
                "/permissions/:permission_name",
                delete(UserRouter::delete_permission),
            )
            .route("/authorize/:role_name", post(UserRouter::authorize_role))
            .route("/revoke/:role_name", post(UserRouter::revoke_role))
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
            if let (Some(email), Some(password), Some(first_name), Some(last_name)) = (
                request.email,
                request.password,
                request.first_name,
                request.last_name,
            ) {
                Ok(RegisterRequest {
                    email,
                    password,
                    first_name,
                    last_name,
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
                    name: "name".to_string(),
                    value: format!(
                        "{:#?} {:#?}",
                        user.first_name.clone(),
                        user.last_name.clone()
                    ),
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
        let user_id = token_service.decode_verify_registration_token(&verification_token)?;

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
                name: "name".to_string(),
                value: format!(
                    "{:#?} {:#?}",
                    user.first_name.clone(),
                    user.last_name.clone()
                ),
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

        Ok(Json(UserEndpointResponse::from_user_response(user)))
    }

    pub async fn login_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        Json(request): Json<LoginEndpointRequest>,
    ) -> ServiceResult<Json<ObtainTokenResponse>> {
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

        info!("Obtained response from service, creating bearer token...");
        let tokens = token_service.create_token(user.id, &user.email)?;

        info!("Token created, updating user token!");
        user_service
            .refresh_token(RefreshTokenRequest {
                id: user.id,
                token: tokens.clone().refresh,
            })
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        info!("User token updated, returning response!");
        Ok(Json(ObtainTokenResponse::from_tokens(tokens)))
    }

    pub async fn refresh_token_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        Json(request): Json<RefreshtokenEndpointRequest>,
    ) -> ServiceResult<Json<ObtainTokenResponse>> {
        info!("Refresh token Endpoint, creating service request...");
        request.validate()?;
        let refresh_token = request.token.unwrap();
        let claims = token_service.decode_refresh_token(&refresh_token.clone())?;
        let email = claims.user_email;
        let user_id = claims.user_id;
        info!("Token decoded, checking if token match user...");
        let is_valid_token = user_service
            .verify_token(VerifyTokenRequest {
                id: claims.user_id,
                token: refresh_token,
            })
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner()
            .valid;

        match is_valid_token {
            true => {
                info!("Validated token, creating token...");
                let tokens = token_service.create_token(user_id, &email)?;

                info!("Token created, updating user token!");
                user_service
                    .refresh_token(RefreshTokenRequest {
                        id: claims.user_id,
                        token: tokens.clone().refresh,
                    })
                    .await
                    .map_err(|_| ServiceError::InternalServerError)?;

                info!("User token updated, returning response!");
                Ok(Json(ObtainTokenResponse::from_tokens(tokens)))
            }
            false => Err(ServiceError::Unauthorized),
        }
    }

    pub async fn get_current_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        info!("Get User Endpoint, obtaining authorization...");
        let bearer_claims = token_service.decode_bearer_token(authorization.token())?;

        info!("Obtained authorization, obtaining response from User service...");
        let user = user_service
            .get_user(GetUserRequest {
                id: bearer_claims.user_id,
            })
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        info!("Returning response!");
        Ok(Json(UserEndpointResponse::from_user_response(user)))
    }

    pub async fn update_user_endpoint(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Json(request): Json<UpdateEndpointRequest>,
    ) -> ServiceResult<Json<UserEndpointResponse>> {
        info!("Update User Endpoint, obtaining authorization...");
        let bearer_claims = token_service.decode_bearer_token(authorization.token())?;

        info!("Obtained authorization, obtaining response from User service...");
        let user = user_service
            .update(UpdateRequest {
                id: bearer_claims.user_id,
                fields: Some(UpdateFields {
                    password: request.password,
                    first_name: request.first_name,
                    last_name: request.last_name,
                    bio: request.bio,
                    image: request.image,
                }),
            })
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        info!("Returning response!");
        Ok(Json(UserEndpointResponse::from_user_response(user)))
    }

    pub async fn get_roles(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        pagination: Query<Pagination>,
    ) -> ServiceResult<Json<RolesListResponse>> {
        info!("Get Roles Endpoint, obtaining authorization...");
        token_service.decode_bearer_token(authorization.token())?;
        let pagination: Pagination = pagination.0;
        let offset = pagination.page * *PAGINATION_SIZE;

        info!("Obtained authorization, obtaining response from User service...");
        let role_response = user_service
            .list_roles(GetListRequest {
                offset,
                limit: *PAGINATION_SIZE,
            })
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        info!("Returning roles list response!");
        Ok(Json(RolesListResponse::from_list_response(role_response)))
    }

    pub async fn add_role(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Json(request): Json<AddRolePermissionRequest>,
    ) -> ServiceResult<Json<StatusMessageResponse>> {
        info!("Add Role Endpoint, obtaining authorization...");
        token_service.decode_bearer_token(authorization.token())?;

        info!("Obtained authorization, adding role...");
        match request.name {
            Some(request_name) => {
                info!("Adding role {:?}...", &request_name);
                let add_role_request = RolesPermissionsRequest { name: request_name };

                let status = user_service
                    .add_role(add_role_request)
                    .await
                    .map_err(|_| ServiceError::InternalServerError)?
                    .into_inner();
                info!("Added Role!");

                Ok(Json(StatusMessageResponse {
                    status: status.message,
                }))
            }
            None => Err(ServiceError::BadRequest("Missing role name".to_string())),
        }
    }

    pub async fn delete_role(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Path(role_name): Path<String>,
    ) -> ServiceResult<Json<StatusMessageResponse>> {
        info!("Delete Role Endpoint, obtaining authorization...");
        token_service.decode_bearer_token(authorization.token())?;

        info!("Obtained authorization, deleting role {:?}...", &role_name);
        let delete_role_request = RolesPermissionsRequest { name: role_name };

        let status = user_service
            .delete_role(delete_role_request)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        info!("Role deleted!");
        Ok(Json(StatusMessageResponse {
            status: status.message,
        }))
    }

    pub async fn get_permissions(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        pagination: Query<Pagination>,
    ) -> ServiceResult<Json<PermissionsListResponse>> {
        info!("Get Permissions Endpoint, obtaining authorization...");
        token_service.decode_bearer_token(authorization.token())?;
        let pagination: Pagination = pagination.0;
        let offset = pagination.page * *PAGINATION_SIZE;

        info!("Obtained authorization, obtaining response from User service...");
        let permission_response = user_service
            .list_roles(GetListRequest {
                offset,
                limit: *PAGINATION_SIZE,
            })
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        info!("Returning permissions list response!");
        Ok(Json(PermissionsListResponse::from_list_response(
            permission_response,
        )))
    }

    pub async fn add_permission(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Json(request): Json<AddRolePermissionRequest>,
    ) -> ServiceResult<Json<StatusMessageResponse>> {
        info!("Add Permission Endpoint, obtaining authorization...");
        token_service.decode_bearer_token(authorization.token())?;

        info!("Obtained authorization, adding permission...");
        match request.name {
            Some(permission_name) => {
                info!("Adding permission {:?}...", &permission_name);
                let add_permission_request = RolesPermissionsRequest {
                    name: permission_name,
                };

                let status = user_service
                    .add_permission(add_permission_request)
                    .await
                    .map_err(|_| ServiceError::InternalServerError)?
                    .into_inner();

                info!("Role added!");
                Ok(Json(StatusMessageResponse {
                    status: status.message,
                }))
            }
            None => Err(ServiceError::BadRequest("Missing role name".to_string())),
        }
    }

    pub async fn delete_permission(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Path(permission_name): Path<String>,
    ) -> ServiceResult<Json<StatusMessageResponse>> {
        info!("Delete Permission Endpoint, obtaining authorization...");
        token_service.decode_bearer_token(authorization.token())?;

        info!(
            "Obtained authorization, deleting permission {:?}...",
            &permission_name
        );
        let delete_permission_request = RolesPermissionsRequest {
            name: permission_name,
        };

        let status = user_service
            .delete_permission(delete_permission_request)
            .await
            .map_err(|_| ServiceError::InternalServerError)?
            .into_inner();

        info!("Permission deleted!");

        Ok(Json(StatusMessageResponse {
            status: status.message,
        }))
    }

    pub async fn authorize_role(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Path(role_name): Path<String>,
        Json(request): Json<AuthorizeRevokeRolePermissionRequest>,
    ) -> ServiceResult<Json<StatusMessageResponse>> {
        info!("Authorize Role Endpoint, obtaining authorization...");
        token_service.decode_bearer_token(authorization.token())?;

        info!("Obtained authorization, adding permission...");
        match request.permissions {
            Some(permissions) => {
                let permissions_string = &permissions.join(",");
                info!(
                    "Authorizing Role {:?} with {:?}",
                    &role_name, &permissions_string
                );
                let authorize_request = Role {
                    name: role_name,
                    permissions,
                };

                let status = user_service
                    .authorize_role(authorize_request)
                    .await
                    .map_err(|_| ServiceError::InternalServerError)?
                    .into_inner();

                info!("Role authorized!");

                Ok(Json(StatusMessageResponse {
                    status: status.message,
                }))
            }
            None => Err(ServiceError::BadRequest(
                "Missing permissions to authorize".to_string(),
            )),
        }
    }
    pub async fn revoke_role(
        State(mut user_service): State<StateUserService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Path(role_name): Path<String>,
        Json(request): Json<AuthorizeRevokeRolePermissionRequest>,
    ) -> ServiceResult<Json<StatusMessageResponse>> {
        info!("Revoking Role Endpoint, obtaining authorization...");
        token_service.decode_bearer_token(authorization.token())?;

        info!("Obtained authorization, removing permission...");
        match request.permissions {
            Some(permissions) => {
                let permissions_string = &permissions.join(",");
                info!(
                    "Revoking Role {:?} from {:?}",
                    &role_name, &permissions_string
                );
                let authorize_request = Role {
                    name: role_name,
                    permissions,
                };

                let status = user_service
                    .authorize_role(authorize_request)
                    .await
                    .map_err(|_| ServiceError::InternalServerError)?
                    .into_inner();

                info!("Role revoked!");

                Ok(Json(StatusMessageResponse {
                    status: status.message,
                }))
            }
            None => Err(ServiceError::BadRequest(
                "Missing permissions to revoke".to_string(),
            )),
        }
    }
}
