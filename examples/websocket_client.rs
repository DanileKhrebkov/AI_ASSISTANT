use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio::io::{AsyncBufReadExt, BufReader};
use std::io::Cursor;
use rodio::{Decoder, OutputStream, Sink};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Connecting to WebSocket server...");
    
    let url = "ws://127.0.0.1:8080";
    let (ws_stream, _) = connect_async(url).await?;
    
    println!("✅ Connected!");
    println!("💡 Press '+' and speak to ask a question");
    println!("💡 Or type text directly");
    println!("💡 Type 'exit' to quit");
    
    let (mut sender, mut receiver) = ws_stream.split();
    
    // Создаем аудио систему
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("📨 Server: {}", text);
                }
                Ok(Message::Binary(data)) => {
                    println!("🎵 Received audio: {} bytes", data.len());
                    
                    // Воспроизводим аудио
                    let cursor = Cursor::new(data);
                    match Decoder::new(cursor) {
                        Ok(source) => {
                            sink.append(source);
                            println!("🔊 Playing audio...");
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to decode audio: {}", e);
                        }
                    }
                }
                Ok(Message::Ping(_)) => {}
                Ok(Message::Pong(_)) => {}
                Ok(Message::Close(_)) => {
                    println!("Server closed connection");
                    break;
                }
                Ok(Message::Frame(_)) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
    });
    
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    
    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;
        }
        
        let trimmed = line.trim();
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }
        
        if !trimmed.is_empty() {
            sender.send(Message::Text(trimmed.to_string())).await?;
        }
    }
    
    sender.close().await?;
    receive_task.await?;
    
    Ok(())
}