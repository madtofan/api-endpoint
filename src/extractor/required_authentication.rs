use crate::domain::errors::ServiceError;
use crate::service::token::DynTokenServiceTrait;
use async_trait::async_trait;
use axum::extract::{FromRequest, RequestParts};
use axum::http::header::AUTHORIZATION;
use axum::Extension;
use tracing::error;

pub struct RequiredAuthentication(pub i64);

#[async_trait]
impl<B> FromRequest<B> for RequiredAuthentication
where
    B: Send + Sync,
{
    type Rejection = ServiceError;

    async fn from_request(request: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(token_service): Extension<DynTokenServiceTrait> =
            Extension::from_request(request)
                .await
                .map_err(|err| ServiceError::InternalServerErrorWithContext(err.to_string()))?;

        if let Some(authorization_header) = request.headers().get(AUTHORIZATION) {
            let header_value = authorization_header
                .to_str()
                .map_err(|_| ServiceError::Unauthorized)?;

            if !header_value.contains("Bearer") {
                error!("request does not contain valid 'Bearer' prefix for authorization");
                return Err(ServiceError::Unauthorized);
            }

            let tokenized_value: Vec<_> = header_value.split(' ').collect();

            if tokenized_value.len() != 2 || tokenized_value.get(1).is_none() {
                error!("request does not contain valid 'Bearer' prefix for authorization");
                return Err(ServiceError::Unauthorized);
            }

            let token_value = tokenized_value.into_iter().nth(1).unwrap();
            let user_id = token_service
                .get_user_id_from_token(String::from(token_value))
                .map_err(|err| {
                    error!("could not validate user ID from token: {:?}", err);
                    ServiceError::Unauthorized
                })?;

            Ok(RequiredAuthentication(user_id))
        } else {
            Err(ServiceError::Unauthorized)
        }
    }
}
