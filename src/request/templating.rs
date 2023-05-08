use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::templating::TemplateInput;

#[derive(Clone, Serialize, Deserialize, Debug, Validate, Default)]
pub struct AddTemplateEndpointRequest {
    #[validate(required, length(min = 6, max = 30))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(required, length(min = 1))]
    pub body: Option<String>,
    pub template_inputs: Option<Vec<TemplateInputsEndpoint>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate, Default)]
pub struct TemplateInputsEndpoint {
    #[validate(required, length(min = 6, max = 30))]
    pub name: Option<String>,
    pub default_value: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate, Default)]
pub struct RemoveTemplateEndpointRequest {
    #[validate(required, length(min = 6, max = 30))]
    pub name: Option<String>,
}

impl From<TemplateInputsEndpoint> for TemplateInput {
    fn from(template_inputs_endpoint: TemplateInputsEndpoint) -> Self {
        Self {
            name: template_inputs_endpoint.name.unwrap_or_default(),
            default_value: template_inputs_endpoint.default_value.unwrap_or_default(),
        }
    }
}
