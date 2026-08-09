use anyhow::{anyhow, Result};
use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Read;

pub struct PiperTTS;

impl PiperTTS {
    pub fn new(_model_path: &str) -> Self {
        Self
    }

    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>> {
        log::info!("TTS: synthesizing '{}'", text);
        
        let clean_text: String = text
            .chars()
            .filter(|c| c.is_ascii() || c.is_alphabetic() || c.is_whitespace() || matches!(c, '.' | ',' | '!' | '?' | '-' | ' '))
            .collect::<String>()
            .trim()
            .to_string();
        
        if clean_text.is_empty() {
            return Err(anyhow!("No text to synthesize"));
        }
        
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path().to_str().unwrap();
        
        // Используем espeak-ng для генерации WAV файла
        let output = Command::new("espeak-ng")
            .args(&[
                "-v", "ru",          // русский голос
                "-w", path,          // записать в WAV
                clean_text.as_str(), // текст
            ])
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("espeak-ng failed: {}", stderr));
        }
        
        let metadata = std::fs::metadata(path)?;
        if metadata.len() == 0 {
            return Err(anyhow!("espeak-ng generated empty file"));
        }
        
        let mut file = std::fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        if buffer.is_empty() {
            return Err(anyhow!("No audio generated"));
        }
        
        log::info!("✅ TTS generated {} bytes", buffer.len());
        Ok(buffer)
    }
}