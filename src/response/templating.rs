use serde::{Deserialize, Serialize};

use crate::{
    request::templating::TemplateInputsEndpoint,
    templating::{list_template_response::Templates, ListTemplateResponse, TemplatingResponse},
};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TemplatingEndpointResponse {
    pub message: String,
}

impl TemplatingEndpointResponse {
    pub fn from_templating_response(templating_response: TemplatingResponse) -> Self {
        Self {
            message: templating_response.message,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TemplatesEndpointResponse {
    pub name: String,
    pub description: String,
    pub template_inputs: Vec<TemplateInputsEndpoint>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ListTemplateEndpointResponse {
    pub templates: Vec<TemplatesEndpointResponse>,
}

impl From<Templates> for TemplatesEndpointResponse {
    fn from(templates: Templates) -> Self {
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
                .collect::<Vec<TemplatesEndpointResponse>>(),
        }
    }
}
