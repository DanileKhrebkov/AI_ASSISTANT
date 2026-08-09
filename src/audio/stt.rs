use anyhow::{anyhow, Result};
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

pub struct WhisperSTT {
    context: WhisperContext,
}

impl WhisperSTT {
    pub fn new(model_path: &str) -> Result<Self> {
        log::info!("Loading Whisper model from: {}", model_path);
        
        let params = WhisperContextParameters::default();
        let context = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| anyhow!("Failed to load Whisper model: {}", e))?;
        
        Ok(Self { context })
    }

    pub async fn transcribe(&self, audio_data: &[f32]) -> Result<String> {
        log::info!("Transcribing audio ({} samples)...", audio_data.len());
        
        if audio_data.is_empty() {
            return Err(anyhow!("Empty audio data"));
        }

        let mut state = self.context.create_state()
            .map_err(|e| anyhow!("Failed to create whisper state: {}", e))?;

        let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        state.full(params, audio_data)
            .map_err(|e| anyhow!("Whisper transcription failed: {}", e))?;

        let mut result = String::new();

        // Используем итератор для получения сегментов
        for segment in state.as_iter() {
            // to_str() возвращает Result, нужно обработать ошибку
            match segment.to_str() {
                Ok(text) => {
                    result.push_str(text);
                    result.push(' ');
                }
                Err(e) => {
                    log::warn!("Failed to get text from segment: {}", e);
                }
            }
        }

        if result.is_empty() {
            return Err(anyhow!("No text extracted from segments"));
        }

        log::info!("Transcription: {}", result.trim());
        Ok(result.trim().to_string())
    }
}