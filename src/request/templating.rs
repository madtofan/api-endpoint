use madtofan_microservice_common::templating::TemplateInput;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Debug, Validate, Default, TS)]
#[ts(export, export_to = "bindings/templating/")]
pub struct AddTemplateEndpointRequest {
    #[validate(required, length(min = 6, max = 30))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(required, length(min = 1))]
    pub body: Option<String>,
    pub template_inputs: Option<Vec<TemplateInputsEndpoint>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate, Default, TS)]
#[ts(export, export_to = "bindings/templating/")]
pub struct TemplateInputsEndpoint {
    #[validate(required, length(min = 6, max = 30))]
    pub name: Option<String>,
    pub default_value: Option<String>,
}

impl From<TemplateInputsEndpoint> for TemplateInput {
    fn from(template_inputs_endpoint: TemplateInputsEndpoint) -> Self {
        Self {
            name: template_inputs_endpoint.name.unwrap_or_default(),
            default_value: template_inputs_endpoint.default_value.unwrap_or_default(),
        }
    }
}
