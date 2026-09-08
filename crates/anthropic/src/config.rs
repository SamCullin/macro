use macro_env_var::env_vars;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

const ANTHROPIC_ROUTER_BASE_URL: &str = "https://api.anthropic.com";

env_vars! {
    struct AnthropicApiKey;
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub api_base: String,
    pub headers: HeaderMap,
}

impl Config {
    pub fn dangrously_try_from_env() -> Self {
        let api_key = AnthropicApiKey::new().expect("api key");
        let api_base = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| ANTHROPIC_ROUTER_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(api_key.as_ref()).expect("good config"),
        );
        if api_base != ANTHROPIC_ROUTER_BASE_URL {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key.as_ref()))
                    .expect("good config"),
            );
        }
        headers.insert(
            "anthropic-version",
            "2023-06-01".parse().expect("good version"),
        );
        Self { api_base, headers }
    }
}
