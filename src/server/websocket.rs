use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use log::{info, error, warn};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::audio::{AudioRecorder, WhisperSTT, PiperTTS};
use crate::ai::ollama::OllamaClient;

pub struct WebSocketServer {
    recorder: Arc<AudioRecorder>,
    stt: Arc<WhisperSTT>,
    tts: Arc<PiperTTS>,
    ai: Arc<OllamaClient>,
    record_duration: u64,
}

impl WebSocketServer {
    pub fn new(
        recorder: Arc<AudioRecorder>,
        stt: Arc<WhisperSTT>,
        tts: Arc<PiperTTS>,
        ai: Arc<OllamaClient>,
        record_duration: u64,
    ) -> Self {
        Self {
            recorder,
            stt,
            tts,
            ai,
            record_duration,
        }
    }

    pub async fn run(&self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await
            .map_err(|e| anyhow!("Failed to bind to {}: {}", addr, e))?;
        
        info!("✅ WebSocket server listening on ws://{}", addr);
        info!("💡 Send '+' to start voice recording");

        while let Ok((stream, addr)) = listener.accept().await {
            let handler = self.clone();
            tokio::spawn(async move {
                if let Err(e) = handler.handle_connection(stream, addr).await {
                    error!("Client {} error: {}", addr, e);
                }
            });
        }

        Ok(())
    }

    async fn handle_connection(&self, stream: TcpStream, addr: std::net::SocketAddr) -> Result<()> {
        info!("New WebSocket connection from: {}", addr);

        let ws_stream = accept_async(stream)
            .await
            .map_err(|e| anyhow!("WebSocket handshake failed: {}", e))?;

        let (mut sender, mut receiver) = ws_stream.split();
        
        // Приветственное сообщение
        let welcome = "🎤 Connected! Press '+' to start voice input".to_string();
        sender.send(Message::Text(welcome)).await?;

        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    info!("Received text from {}: {}", addr, text);
                    
                    if text == "+" {
                        // Запускаем запись голоса
                        info!("🎙️ Recording started from {}", addr);
                        sender.send(Message::Text("🎙️ Listening...".into())).await?;
                        
                        // 1. Записываем аудио
                        let audio_data = match self.recorder.record(self.record_duration) {
                            Ok(data) => data,
                            Err(e) => {
                                error!("Recording failed: {}", e);
                                sender.send(Message::Text(format!("❌ Recording failed: {}", e))).await?;
                                continue;
                            }
                        };

                        info!("📝 Recording complete ({} samples)", audio_data.len());

                        if audio_data.is_empty() {
                            sender.send(Message::Text("❌ No audio recorded".into())).await?;
                            continue;
                        }

                        // 2. Распознаем речь через Whisper
                        sender.send(Message::Text("🧠 Transcribing...".into())).await?;
                        let text = match self.stt.transcribe(&audio_data).await {
                            Ok(t) => t,
                            Err(e) => {
                                error!("STT failed: {}", e);
                                sender.send(Message::Text(format!("❌ STT failed: {}", e))).await?;
                                continue;
                            }
                        };

                        if text.is_empty() {
                            sender.send(Message::Text("❌ No speech detected".into())).await?;
                            continue;
                        }

                        info!("🎤 Transcribed: {}", text);
                        sender.send(Message::Text(format!("📝 You said: {}", text))).await?;

                        // 3. Отправляем в Ollama
                        sender.send(Message::Text("🤔 Thinking...".into())).await?;
                        let ai_response = match self.ai.chat(&text).await {
                            Ok(r) => r,
                            Err(e) => {
                                error!("AI failed: {}", e);
                                sender.send(Message::Text(format!("❌ AI error: {}", e))).await?;
                                continue;
                            }
                        };

                        info!("🤖 AI response: {}", ai_response);
                        sender.send(Message::Text(format!("🤖 {}", ai_response))).await?;

                        // 4. Синтезируем речь через TTS
                        sender.send(Message::Text("🔊 Speaking...".into())).await?;
                        let audio_bytes = match self.tts.synthesize(&ai_response).await {
                            Ok(a) => a,
                            Err(e) => {
                                error!("TTS failed: {}", e);
                                sender.send(Message::Text(format!("❌ TTS error: {}", e))).await?;
                                continue;
                            }
                        };

                        // Отправляем аудио как бинарные данные
                        if !audio_bytes.is_empty() {
                            sender.send(Message::Binary(audio_bytes)).await?;
                            info!("✅ Audio sent to {}", addr);
                        }
                        
                        sender.send(Message::Text("✅ Done! Press '+' to ask another question".into())).await?;
                    } else {
                        // Обычный текстовый чат без голоса
                        sender.send(Message::Text("🤔 Processing...".into())).await?;
                        let response = match self.ai.chat(&text).await {
                            Ok(r) => r,
                            Err(e) => {
                                error!("AI failed: {}", e);
                                format!("❌ AI error: {}", e)
                            }
                        };
                        sender.send(Message::Text(format!("🤖 {}", response))).await?;
                    }
                }
                Ok(Message::Binary(_)) => {
                    warn!("Received binary data from {}, ignoring", addr);
                }
                Ok(Message::Ping(_)) => {
                    // Автоматический ответ на ping
                }
                Ok(Message::Pong(_)) => {
                    // Игнорируем pong
                }
                Ok(Message::Close(_)) => {
                    info!("Client {} disconnected", addr);
                    break;
                }
                Ok(Message::Frame(_)) => {
                    // Игнорируем фреймы
                }
                Err(e) => {
                    error!("WebSocket error from {}: {}", addr, e);
                    break;
                }
            }
        }

        Ok(())
    }
}

// Реализуем Clone для WebSocketServer
impl Clone for WebSocketServer {
    fn clone(&self) -> Self {
        Self {
            recorder: self.recorder.clone(),
            stt: self.stt.clone(),
            tts: self.tts.clone(),
            ai: self.ai.clone(),
            record_duration: self.record_duration,
        }
    }
}