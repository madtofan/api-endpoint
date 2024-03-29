use axum::{
    extract::{Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    routing::{delete, get},
    Json, Router, TypedHeader,
};
use madtofan_microservice_common::{
    errors::{ServiceError, ServiceResult},
    templating::{AddTemplateRequest, ListTemplateRequest, RemoveTemplateRequest, TemplateInput},
};
use validator::Validate;

use crate::{
    request::{templating::AddTemplateEndpointRequest, Pagination},
    response::templating::{ListTemplateEndpointResponse, TemplateEndpointResponse},
    utilities::{
        constants::PAGINATION_SIZE,
        service_register::ServiceRegister,
        states::{templating_service::StateTemplatingService, token_service::StateTokenService},
    },
};
use tracing::info;

pub struct TemplatingRouter;

impl TemplatingRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route(
                "/",
                get(TemplatingRouter::list_templates_endpoint)
                    .post(TemplatingRouter::add_template_endpoint),
            )
            .route(
                "/:template_name",
                delete(TemplatingRouter::remove_template_endpoint),
            )
            .with_state(service_register)
    }

    pub async fn list_templates_endpoint(
        State(mut templating_service): State<StateTemplatingService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        pagination: Query<Pagination>,
    ) -> ServiceResult<Json<ListTemplateEndpointResponse>> {
        info!("List Templates Endpoint");
        token_service.decode_bearer_token(authorization.token())?;
        let pagination: Pagination = pagination.0;
        let offset = pagination.page.unwrap_or_default() * *PAGINATION_SIZE;

        let list_templates_request: ListTemplateRequest = ListTemplateRequest {
            offset,
            limit: *PAGINATION_SIZE,
        };
        let response = templating_service
            .list_templates(list_templates_request)
            .await?
            .into_inner();

        Ok(Json(
            ListTemplateEndpointResponse::from_list_template_response(response),
        ))
    }

    pub async fn add_template_endpoint(
        State(mut templating_service): State<StateTemplatingService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Json(request): Json<AddTemplateEndpointRequest>,
    ) -> ServiceResult<Json<TemplateEndpointResponse>> {
        info!("Add Template Endpoint");

        token_service.decode_bearer_token(authorization.token())?;
        request.validate()?;
        let add_template_request: AddTemplateRequest =
            if let (Some(name), Some(description), Some(body), Some(template_inputs)) = (
                request.name,
                request.description,
                request.body,
                request.template_inputs,
            ) {
                Ok(AddTemplateRequest {
                    name,
                    description,
                    body,
                    template_inputs: template_inputs
                        .into_iter()
                        .map(|input| input.into())
                        .collect::<Vec<TemplateInput>>(),
                })
            } else {
                Err(ServiceError::BadRequest(
                    "Missing parameters in the request".to_string(),
                ))
            }?;

        let response: TemplateEndpointResponse = templating_service
            .add_template(add_template_request)
            .await?
            .into_inner()
            .into();

        Ok(Json(response))
    }

    pub async fn remove_template_endpoint(
        State(mut templating_service): State<StateTemplatingService>,
        State(token_service): State<StateTokenService>,
        authorization: TypedHeader<Authorization<Bearer>>,
        Path(template_name): Path<String>,
    ) -> ServiceResult<Json<TemplateEndpointResponse>> {
        info!("Remove Template Endpoint");

        token_service.decode_bearer_token(authorization.token())?;
        let remove_template_request = RemoveTemplateRequest {
            name: template_name,
        };

        let response: TemplateEndpointResponse = templating_service
            .remove_template(remove_template_request)
            .await?
            .into_inner()
            .into();

        Ok(Json(response))
    }
}
