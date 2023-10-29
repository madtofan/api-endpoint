use std::convert::Infallible;

use async_stream::stream;
use axum::{
    extract::{Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    response::sse::{Event as SseEvent, Sse},
    routing::{delete, get, post},
    Json, Router, TypedHeader,
};
use futures::Stream;
use madtofan_microservice_common::{
    errors::{ServiceError, ServiceResult},
    notification::{
        AddGroupRequest, AddMessageRequest, AddSubscriberRequest, GetGroupsRequest,
        GetMessagesRequest, RemoveGroupRequest, RemoveSubscriberRequest,
    },
};
use validator::Validate;

use crate::{
    request::{
        notification::{AddGroupEndpointRequest, SendNotificationEndpointRequest},
        Pagination,
    },
    response::notification::{NotificationEndpointResponse, NotificationLogsEndpointResponse},
    utilities::{
        constants::PAGINATION_SIZE,
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
            .route("/", post(NotificationRouter::send_notification))
            .route(
                "/:bearer_token",
                get(NotificationRouter::event_notification),
            )
            .route("/log", get(NotificationRouter::get_notification_logs))
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

    pub async fn event_notification(
        State(mut channels_service): State<StateChannelsService>,
        State(mut notification_service): State<StateNotificationService>,
        State(token_service): State<StateTokenService>,
        Path(bearer_token): Path<String>,
    ) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
        info!("Subscribe Notification Endpoint");
        let stream = stream! {
            let bearer_claims = token_service
                .decode_bearer_token(&bearer_token);
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

    pub async fn send_notification(
        State(channels_service): State<StateChannelsService>,
        State(token_service): State<StateTokenService>,
        State(mut notification_service): State<StateNotificationService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Json(request): Json<SendNotificationEndpointRequest>,
    ) -> ServiceResult<Json<NotificationEndpointResponse>> {
        info!("Send Notification Endpoint");
        request.validate()?;
        token_service.decode_notification_sender_token(authorization.token())?;
        let tag: ChannelTag = request.address.unwrap().parse()?;
        let subject = request.subject.unwrap_or_default();
        let message = request.message.unwrap_or_default();

        let notification_response = notification_service
            .add_message(AddMessageRequest {
                channel: tag.to_string(),
                subject: subject.clone(),
                message: message.clone(),
            })
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext(
                    "failed to generate message ID".to_string(),
                )
            })?;
        let notification = notification_response.into_inner();

        let notification_message = NotificationMessage {
            id: notification.id,
            channel: tag.to_string(),
            subject,
            message,
            datetime: notification.date,
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

    pub async fn get_notification_logs(
        State(mut notification_service): State<StateNotificationService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        pagination: Query<Pagination>,
    ) -> ServiceResult<Json<NotificationLogsEndpointResponse>> {
        let bearer_claims = token_service.decode_bearer_token(authorization.token());
        let pagination: Pagination = pagination.0;
        let offset = pagination.page * *PAGINATION_SIZE;

        let channels = match bearer_claims {
            Ok(claims) => {
                let mut new_tags = Vec::new();
                new_tags.push(ChannelTag::UserId(claims.user_id));
                let get_groups_request = GetGroupsRequest {
                    user_id: claims.user_id,
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
        }
        .into_iter()
        .map(|tag| tag.to_string())
        .collect::<Vec<String>>();

        let notification_response = notification_service
            .get_messages(GetMessagesRequest {
                channels,
                offset,
                limit: *PAGINATION_SIZE,
            })
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext("failed to get logs".to_string())
            })?
            .into_inner();

        Ok(Json(NotificationLogsEndpointResponse {
            notifications: notification_response
                .messages
                .into_iter()
                .map(NotificationMessage::from_message_response)
                .collect(),
            count: notification_response.count,
        }))
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
            "successfully created group: {}, group token is: {}",
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
