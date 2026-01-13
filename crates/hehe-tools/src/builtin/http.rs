use crate::error::Result;
use crate::traits::{Tool, ToolOutput};
use async_trait::async_trait;
use hehe_core::{Context, ToolDefinition, ToolParameter};
use reqwest::{header::HeaderMap, Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

pub struct HttpRequestTool {
    def: ToolDefinition,
    client: Client,
}

impl HttpRequestTool {
    pub fn new() -> Self {
        let def = ToolDefinition::new("http_request", "Make an HTTP request")
            .with_required_param(
                "url",
                ToolParameter::string().with_description("The URL to request"),
            )
            .with_param(
                "method",
                ToolParameter::string()
                    .with_description("HTTP method (GET, POST, PUT, DELETE, PATCH)")
                    .with_default(Value::String("GET".into())),
            )
            .with_param(
                "headers",
                ToolParameter::object().with_description("HTTP headers as key-value pairs"),
            )
            .with_param(
                "body",
                ToolParameter::string().with_description("Request body (for POST, PUT, PATCH)"),
            )
            .with_param(
                "json",
                ToolParameter::object().with_description("JSON body (alternative to body)"),
            )
            .with_param(
                "timeout_ms",
                ToolParameter::integer()
                    .with_description("Request timeout in milliseconds (default: 30000)")
                    .with_default(Value::Number(30000.into())),
            );

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("hehe-agent/0.1")
            .build()
            .unwrap_or_default();

        Self { def, client }
    }
}

impl Default for HttpRequestTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct HttpRequestInput {
    url: String,
    #[serde(default = "default_method")]
    method: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
    json: Option<Value>,
    timeout_ms: Option<u64>,
}

fn default_method() -> String {
    "GET".to_string()
}

#[derive(Serialize)]
struct HttpResponse {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: String,
}

#[async_trait]
impl Tool for HttpRequestTool {
    fn definition(&self) -> &ToolDefinition {
        &self.def
    }

    async fn execute(&self, _ctx: &Context, input: Value) -> Result<ToolOutput> {
        let input: HttpRequestInput = serde_json::from_value(input)?;

        let method = match input.method.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            other => {
                return Ok(ToolOutput::error(format!("Unsupported HTTP method: {}", other)));
            }
        };

        let mut request = self.client.request(method, &input.url);

        if let Some(timeout_ms) = input.timeout_ms {
            request = request.timeout(Duration::from_millis(timeout_ms));
        }

        if let Some(headers) = input.headers {
            let mut header_map = HeaderMap::new();
            for (key, value) in headers {
                if let (Ok(name), Ok(val)) = (
                    key.parse::<reqwest::header::HeaderName>(),
                    value.parse::<reqwest::header::HeaderValue>(),
                ) {
                    header_map.insert(name, val);
                }
            }
            request = request.headers(header_map);
        }

        if let Some(json_body) = input.json {
            request = request.json(&json_body);
        } else if let Some(body) = input.body {
            request = request.body(body);
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let status_text = response.status().canonical_reason().unwrap_or("").to_string();
                let headers: HashMap<String, String> = response
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();

                let body = response.text().await.unwrap_or_default();

                let http_response = HttpResponse {
                    status,
                    status_text,
                    headers,
                    body,
                };

                let output = ToolOutput::json(&http_response)?
                    .with_metadata("url", &input.url)
                    .with_metadata("status", status);

                if status >= 400 {
                    Ok(ToolOutput {
                        is_error: true,
                        ..output
                    })
                } else {
                    Ok(output)
                }
            }
            Err(e) => {
                let message = if e.is_timeout() {
                    "Request timed out".to_string()
                } else if e.is_connect() {
                    format!("Connection failed: {}", e)
                } else {
                    format!("Request failed: {}", e)
                };
                Ok(ToolOutput::error(message))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_request_definition() {
        let tool = HttpRequestTool::new();
        let def = tool.definition();
        assert_eq!(def.name, "http_request");
        assert!(!def.dangerous);
    }
}
