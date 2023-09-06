use std::convert::Infallible;

use async_stream::stream;
use axum::{
    extract::{Path, State},
    headers::{authorization::Bearer, Authorization},
    response::sse::{Event as SseEvent, Sse},
    routing::{delete, get, post},
    Json, Router, TypedHeader,
};
use futures::Stream;
use madtofan_microservice_common::{
    errors::{ServiceError, ServiceResult},
    notification::{
        AddGroupRequest, AddSubscriberRequest, GetGroupsRequest, RemoveGroupRequest,
        RemoveSubscriberRequest,
    },
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    request::notification::{AddGroupEndpointRequest, SendNotificationEndpointRequest},
    response::notification::NotificationEndpointResponse,
    utilities::{
        events::{ChannelTag, EventMessage, NotificationMessage},
        service_register::ServiceRegister,
        states::{
            channels::StateChannelsService, notification_service::StateNotificationService,
            token_service::StateTokenService,
        },
    },
};
use tracing::info;

pub struct NotificationRouter;

impl NotificationRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route(
                "/",
                get(NotificationRouter::event_notification)
                    .post(NotificationRouter::send_notification),
            )
            .route(
                "/subscribe/:group",
                get(NotificationRouter::subscribe_to_group)
                    .delete(NotificationRouter::unsubscribe_from_group),
            )
            .route("/group", post(NotificationRouter::add_group))
            .route(
                "/group/:group_name/:admin_email",
                delete(NotificationRouter::remove_group),
            )
            .with_state(service_register)
    }

    pub async fn send_notification(
        State(channels_service): State<StateChannelsService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Json(request): Json<SendNotificationEndpointRequest>,
    ) -> ServiceResult<Json<NotificationEndpointResponse>> {
        info!("Send Notification Endpoint");
        request.validate()?;
        token_service.decode_notification_sender_token(authorization.token())?;
        let tag: ChannelTag = request.address.unwrap().parse()?;
        let message = format!(
            r#"
            # {}
            {}
            "#,
            request.subject.unwrap_or_default(),
            request.message.unwrap_or_default()
        );
        let notification_message = NotificationMessage {
            id: Uuid::new_v4(),
            channel: tag.to_string(),
            message,
        };

        match tag {
            ChannelTag::Broadcast => {
                let event_message = EventMessage::Broadcast(notification_message);
                channels_service.broadcast(event_message).await;
            }
            ChannelTag::UserId(_) => {
                let event_message = EventMessage::User(notification_message);
                channels_service.send_by_tag(&tag, event_message).await;
            }
            ChannelTag::ChannelId(_) => {
                let event_message = EventMessage::Channel(notification_message);
                channels_service.send_by_tag(&tag, event_message).await;
            }
        }

        Ok(Json(NotificationEndpointResponse {
            message: "successfully sent notification".to_string(),
        }))
    }

    pub async fn event_notification(
        State(mut channels_service): State<StateChannelsService>,
        State(mut notification_service): State<StateNotificationService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
    ) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
        info!("Subscribe Notification Endpoint");
        let stream = stream! {
            let bearer_claims = token_service
                .decode_bearer_token(authorization.token());
            let tags = match bearer_claims {
                Ok(claims) => {
                    let mut new_tags = Vec::new();
                    new_tags.push(ChannelTag::UserId(claims.user_id));
                    let get_groups_request = GetGroupsRequest {
                        user_id: claims.user_id
                    };
                    let groups_response = notification_service.get_groups(get_groups_request).await;
                    if let Ok(result) = groups_response {
                        for group in result.into_inner().groups {
                            new_tags.push(ChannelTag::ChannelId(group.name));
                        }
                    }
                    new_tags
                }
                Err(_) => {
                    vec![]
                }
            };
            let mut rx = channels_service.create_channel(tags);

            while let Some(msg) = rx.recv().await {
                let Ok(json) = serde_json::to_string(&msg) else { continue };
                yield Ok(SseEvent::default().data(json));
            }
        };
        Sse::new(stream)
    }

    pub async fn subscribe_to_group(
        State(mut notification_service): State<StateNotificationService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Path(group): Path<String>,
    ) -> ServiceResult<Json<NotificationEndpointResponse>> {
        info!("Subscribe To Group Endpoint");
        let bearer_claims = token_service.decode_bearer_token(authorization.token())?;

        let add_subscriber_request = AddSubscriberRequest {
            user_id: bearer_claims.user_id,
            group,
        };
        notification_service
            .add_subscriber(add_subscriber_request)
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext(
                    "failed to subscribe to group".to_string(),
                )
            })?;

        Ok(Json(NotificationEndpointResponse {
            message: "successfully subscribed".to_string(),
        }))
    }

    pub async fn unsubscribe_from_group(
        State(mut notification_service): State<StateNotificationService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Path(group): Path<String>,
    ) -> ServiceResult<Json<NotificationEndpointResponse>> {
        info!("Unsubscribe From Group Endpoint");
        let bearer_claims = token_service.decode_bearer_token(authorization.token())?;

        let clear_subscription_request = RemoveSubscriberRequest {
            user_id: bearer_claims.user_id,
            group,
        };

        notification_service
            .remove_subscriber(clear_subscription_request)
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext(
                    "failed to clear subscription".to_string(),
                )
            })?;

        Ok(Json(NotificationEndpointResponse {
            message: "successfully unsubscribed".to_string(),
        }))
    }

    pub async fn add_group(
        State(mut notification_service): State<StateNotificationService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Json(request): Json<AddGroupEndpointRequest>,
    ) -> ServiceResult<Json<NotificationEndpointResponse>> {
        info!("Add Group Endpoint");
        request.validate()?;
        token_service.decode_bearer_token(authorization.token())?;
        let group_name = request.group_name.unwrap_or_default();
        let admin_email = request.admin_email.unwrap_or_default();

        let token = token_service.create_notification_sender_token(&group_name, &admin_email)?;
        let add_group_request = AddGroupRequest {
            name: group_name.clone(),
            admin_email: admin_email.clone(),
            token: token.clone(),
        };

        notification_service
            .add_group(add_group_request)
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext("failed to add group".to_string())
            })?;

        let message = format!(
            "successfully subscribed to group {}, group token is: {}",
            &group_name, token
        );
        Ok(Json(NotificationEndpointResponse { message }))
    }

    pub async fn remove_group(
        State(mut notification_service): State<StateNotificationService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Path((group_name, admin_email)): Path<(String, String)>,
    ) -> ServiceResult<Json<NotificationEndpointResponse>> {
        info!("Remove Group Endpoint");
        token_service.decode_bearer_token(authorization.token())?;

        let remove_group_request = RemoveGroupRequest {
            name: group_name,
            admin_email,
        };

        notification_service
            .remove_group(remove_group_request)
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext("failed to remove group".to_string())
            })?;

        Ok(Json(NotificationEndpointResponse {
            message: "successfully removed group".to_string(),
        }))
    }
}