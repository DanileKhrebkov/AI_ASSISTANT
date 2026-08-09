use std::env;
use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub whisper_model_path: String,
    pub piper_model_path: String,
    pub ollama_url: String,
    pub ollama_model: String,
    pub ollama_timeout: u64,
    pub tts_voice: String,
    pub tts_language: String,
    pub audio_sample_rate: u32,
    pub audio_channels: u16,
    pub record_duration_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| anyhow!("Invalid SERVER_PORT"))?,
            whisper_model_path: env::var("WHISPER_MODEL_PATH")
                .unwrap_or_else(|_| "./models/ggml-base.bin".to_string()),
            piper_model_path: env::var("PIPER_MODEL_PATH")
                .unwrap_or_else(|_| "./models/ru_RU-irina-medium.onnx".to_string()), // <-- добавь эту строку
            ollama_url: env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            ollama_model: env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama2".to_string()),
            ollama_timeout: env::var("OLLAMA_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            tts_voice: env::var("TTS_VOICE")
                .unwrap_or_else(|_| "ru-RU-DmitryNeural".to_string()),
            tts_language: env::var("TTS_LANGUAGE")
                .unwrap_or_else(|_| "ru-RU".to_string()),
            audio_sample_rate: env::var("AUDIO_SAMPLE_RATE")
                .unwrap_or_else(|_| "16000".to_string())
                .parse()
                .unwrap_or(16000),
            audio_channels: env::var("AUDIO_CHANNELS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            record_duration_seconds: env::var("RECORD_DURATION_SECONDS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        })
    }
}