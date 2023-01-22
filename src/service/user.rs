use async_trait::async_trait;
use mockall::automock;
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    data::repository::DynUserRepositoryTrait,
    domain::{
        errors::{ServiceError, ServiceResult},
        request::{LoginUserDto, RegisterUserDto, UpdateUserDto},
        response::UserDto,
    },
};

use super::{security::DynSecurityServiceTrait, token::DynTokenServiceTrait};

#[automock]
#[async_trait]
pub trait UserServiceTrait {
    async fn register_user(&self, request: RegisterUserDto) -> ServiceResult<UserDto>;

    async fn login_user(&self, request: LoginUserDto) -> ServiceResult<UserDto>;

    async fn get_current_user(&self, user_id: i64) -> ServiceResult<UserDto>;

    async fn updated_user(&self, user_id: i64, request: UpdateUserDto) -> ServiceResult<UserDto>;
}

pub type DynUserServiceTrait = Arc<dyn UserServiceTrait + Send + Sync>;

pub struct UserService {
    repository: DynUserRepositoryTrait,
    security_service: DynSecurityServiceTrait,
    token_service: DynTokenServiceTrait,
}

impl UserService {
    pub fn new(
        repository: DynUserRepositoryTrait,
        security_service: DynSecurityServiceTrait,
        token_service: DynTokenServiceTrait,
    ) -> Self {
        Self {
            repository,
            security_service,
            token_service,
        }
    }
}

#[async_trait]
impl UserServiceTrait for UserService {
    async fn register_user(&self, request: RegisterUserDto) -> ServiceResult<UserDto> {
        let email = request.email.unwrap();
        let username = request.username.unwrap();
        let password = request.password.unwrap();

        let existing_user = self
            .repository
            .search_user_by_email_or_username(&email, &username)
            .await?;

        if existing_user.is_some() {
            error!("user {:?}/{:?} already exists", email, username);
            return Err(ServiceError::ObjectConflict(String::from(
                "username or email is taken",
            )));
        }

        info!("creating password hash for user {:?}", email);
        let hashed_password = self.security_service.hash_password(&password)?;

        info!("password hashed successfully, creating user {:?}", email);
        let created_user = self
            .repository
            .create_user(&email, &username, &hashed_password)
            .await?;

        info!("user successfully created, generating token");
        let token = self
            .token_service
            .new_token(created_user.id, &created_user.email)?;

        Ok(created_user.into_dto(token))
    }

    async fn login_user(&self, request: LoginUserDto) -> ServiceResult<UserDto> {
        let email = request.email.unwrap();
        let attempted_password = request.password.unwrap();

        info!("searching for existing user {:?}", email);
        let existing_user = self.repository.get_user_by_email(&email).await?;

        if existing_user.is_none() {
            return Err(ServiceError::NotFound(String::from(
                "user email does not exist",
            )));
        }

        let user = existing_user.unwrap();

        info!("user found, verifying password hash for user {:?}", email);
        let is_valid_login_attempt = self
            .security_service
            .verify_password(&user.password, attempted_password)?;

        if !is_valid_login_attempt {
            error!("invalid login attempt for user {:?}", email);
            return Err(ServiceError::InvalidLoginAttempt);
        }

        info!("user login successful, generating token");
        let token = self.token_service.new_token(user.id, &user.email)?;

        Ok(user.into_dto(token))
    }

    async fn get_current_user(&self, user_id: i64) -> ServiceResult<UserDto> {
        info!("retrieving user {:?}", user_id);
        let user = self.repository.get_user_by_id(user_id).await?;

        info!(
            "user found with email {:?}, generating new token",
            user.email
        );
        let token = self.token_service.new_token(user.id, user.email.as_str())?;

        Ok(user.into_dto(token))
    }

    async fn updated_user(&self, user_id: i64, request: UpdateUserDto) -> ServiceResult<UserDto> {
        info!("retrieving user {:?}", user_id);
        let user = self.repository.get_user_by_id(user_id).await?;

        let updated_email = request.email.unwrap_or(user.email);
        let updated_username = request.username.unwrap_or(user.username);
        let updated_bio = request.bio.unwrap_or(user.bio);
        let updated_image = request.image.unwrap_or(user.image);
        let mut updated_hashed_password = user.password;

        if request.password.is_some() && !request.password.as_ref().unwrap().is_empty() {
            updated_hashed_password = self
                .security_service
                .hash_password(request.password.unwrap().as_str())?;
        }

        info!("updating user {:?}", user_id);
        let updated_user = self
            .repository
            .update_user(
                user_id,
                updated_email.clone(),
                updated_username,
                updated_hashed_password,
                updated_bio,
                updated_image,
            )
            .await?;

        info!("user {:?} updated, generating a new token", user_id);
        let token = self
            .token_service
            .new_token(user_id, updated_email.as_str())?;

        Ok(updated_user.into_dto(token))
    }
}
