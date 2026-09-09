use macro_env_var::env_vars;
use reqwest::header::HeaderMap;

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
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", api_key.parse().expect("good config"));
        headers.insert(
            "anthropic-version",
            "2023-06-01".parse().expect("good version"),
        );
        let api_base = std::env::var("ANTHROPIC_BASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| ANTHROPIC_ROUTER_BASE_URL.to_owned())
            .trim_end_matches('/')
            .to_owned();
        Self {
            api_base,
            headers,
        }
    }
}
