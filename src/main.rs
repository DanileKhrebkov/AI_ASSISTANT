mod config;
mod audio;
mod ai;
mod server;
mod utils;

use log::{info, error};
use std::sync::Arc;
use dotenv::dotenv;

use config::Config;
use audio::{AudioRecorder, WhisperSTT, PiperTTS};
use ai::ollama::OllamaClient;
use server::websocket::WebSocketServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    env_logger::init();
    
    info!("🚀 Starting Voice Assistant WebSocket Server");
    info!("📋 Loading configuration...");
    
    let config = Config::from_env()?;
    info!("✅ Configuration loaded");
    
    info!("🎤 Initializing Whisper STT...");
    let stt = Arc::new(WhisperSTT::new(&config.whisper_model_path)?);
    info!("✅ Whisper STT initialized");
    
    info!("🎧 Initializing Audio Recorder...");
    let recorder = Arc::new(AudioRecorder::new(
        config.audio_sample_rate,
        config.audio_channels,
    )?);
    info!("✅ Audio Recorder initialized");
    
    info!("🔊 Initializing Piper TTS...");
    let tts = Arc::new(PiperTTS::new(&config.piper_model_path));
    info!("✅ Piper TTS initialized");
    
    info!("🧠 Initializing Ollama AI...");
    let ai = Arc::new(OllamaClient::new(
        &config.ollama_url,
        &config.ollama_model,
        config.ollama_timeout,
    ));
    info!("✅ Ollama AI initialized");
    
    let server = WebSocketServer::new(
        recorder,
        stt,
        tts,
        ai,
        config.record_duration_seconds,
    );
    
    let addr = format!("{}:{}", config.server_host, config.server_port);
    info!("🌐 WebSocket server listening on ws://{}", addr);
    
    if let Err(e) = server.run(&addr).await {
        error!("Server error: {}", e);
        return Err(e.into());
    }
    
    Ok(())
}