use madtofan_microservice_common::templating::{ListTemplateResponse, TemplateResponse};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::request::templating::TemplateInputsEndpoint;

#[derive(Serialize, Deserialize, Default, Debug, TS)]
#[ts(export, export_to = "bindings/templating/")]
pub struct TemplateEndpointResponse {
    pub name: String,
    pub description: String,
    pub template_inputs: Vec<TemplateInputsEndpoint>,
}

#[derive(Serialize, Deserialize, Default, Debug, TS)]
#[ts(export, export_to = "bindings/templating/")]
pub struct ListTemplateEndpointResponse {
    pub templates: Vec<TemplateEndpointResponse>,
}

impl From<TemplateResponse> for TemplateEndpointResponse {
    fn from(templates: TemplateResponse) -> Self {
        Self {
            name: templates.name,
            description: templates.description,
            template_inputs: templates
                .template_inputs
                .into_iter()
                .map(|input| TemplateInputsEndpoint {
                    name: Some(input.name),
                    default_value: Some(input.default_value),
                })
                .collect::<Vec<TemplateInputsEndpoint>>(),
        }
    }
}

impl ListTemplateEndpointResponse {
    pub fn from_list_template_response(list_template_response: ListTemplateResponse) -> Self {
        Self {
            templates: list_template_response
                .templates
                .into_iter()
                .map(|template| template.into())
                .collect::<Vec<TemplateEndpointResponse>>(),
        }
    }
}
