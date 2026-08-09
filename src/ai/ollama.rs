use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

#[derive(Clone)]
pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
    timeout: u64,
}

impl OllamaClient {
    pub fn new(base_url: &str, model: &str, timeout: u64) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            timeout,
        }
    }

    pub async fn chat(&self, prompt: &str) -> Result<String> {
        log::info!("Sending to Ollama: {}", prompt);
        
        let url = format!("{}/api/generate", self.base_url);
        
        let request_body = json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.7,
                "top_p": 0.9,
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .timeout(Duration::from_secs(self.timeout))
            .send()
            .await
            .map_err(|e| anyhow!("Ollama request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama error {}: {}", status, text));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;

        let text = response_json["response"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'response' field in Ollama response"))?
            .to_string();

        log::info!("Ollama response: {}", text);
        Ok(text)
    }
}