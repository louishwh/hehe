use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsonSchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolParameter {
    #[serde(rename = "type")]
    pub schema_type: JsonSchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ToolParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, ToolParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl ToolParameter {
    pub fn string() -> Self {
        Self {
            schema_type: JsonSchemaType::String,
            description: None,
            default: None,
            enum_values: None,
            items: None,
            properties: None,
            required: None,
        }
    }

    pub fn number() -> Self {
        Self {
            schema_type: JsonSchemaType::Number,
            ..Self::string()
        }
    }

    pub fn integer() -> Self {
        Self {
            schema_type: JsonSchemaType::Integer,
            ..Self::string()
        }
    }

    pub fn boolean() -> Self {
        Self {
            schema_type: JsonSchemaType::Boolean,
            ..Self::string()
        }
    }

    pub fn array(items: ToolParameter) -> Self {
        Self {
            schema_type: JsonSchemaType::Array,
            items: Some(Box::new(items)),
            ..Self::string()
        }
    }

    pub fn object() -> Self {
        Self {
            schema_type: JsonSchemaType::Object,
            properties: Some(HashMap::new()),
            required: Some(vec![]),
            ..Self::string()
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }

    pub fn with_property(mut self, name: impl Into<String>, param: ToolParameter) -> Self {
        if let Some(props) = &mut self.properties {
            props.insert(name.into(), param);
        }
        self
    }

    pub fn with_required(mut self, name: impl Into<String>) -> Self {
        if let Some(req) = &mut self.required {
            req.push(name.into());
        }
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameter,
    #[serde(default)]
    pub dangerous: bool,
}

impl ToolDefinition {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: ToolParameter::object(),
            dangerous: false,
        }
    }

    pub fn with_parameters(mut self, params: ToolParameter) -> Self {
        self.parameters = params;
        self
    }

    pub fn with_required_param(self, name: impl Into<String>, param: ToolParameter) -> Self {
        let name = name.into();
        let mut def = self.with_param(name.clone(), param);
        if let Some(req) = &mut def.parameters.required {
            req.push(name);
        }
        def
    }

    pub fn with_param(mut self, name: impl Into<String>, param: ToolParameter) -> Self {
        if let Some(props) = &mut self.parameters.properties {
            props.insert(name.into(), param);
        }
        self
    }

    pub fn dangerous(mut self) -> Self {
        self.dangerous = true;
        self
    }
}
