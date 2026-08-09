use anyhow::{anyhow, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::mpsc;
use std::time::Duration;

pub struct AudioRecorder {
    sample_rate: u32,
    channels: u16,
}

impl AudioRecorder {
    pub fn new(sample_rate: u32, channels: u16) -> Result<Self> {
        Ok(Self {
            sample_rate,
            channels,
        })
    }

    pub fn record(&self, duration_seconds: u64) -> Result<Vec<f32>> {
        for attempt in 1..=3 {
            match self.try_record(duration_seconds) {
                Ok(data) => return Ok(data),
                Err(e) => {
                    println!("⚠️ Attempt {} failed: {}", attempt, e);
                    if attempt == 3 {
                        return Err(e);
                    }
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
        }
        Err(anyhow!("All recording attempts failed"))
    }

    fn try_record(&self, duration_seconds: u64) -> Result<Vec<f32>> {
        let host = cpal::default_host();
        
        let devices: Vec<_> = host.input_devices()?.collect();
        if devices.is_empty() {
            return Err(anyhow!("No input devices found"));
        }
        
        let device = devices[0].clone();
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        println!("✅ Using device: {}", name);
        
        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        println!("📊 Config: {} Hz, {} channels", sample_rate, channels);
        
        let (tx, rx) = mpsc::channel();
        
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                let _ = tx.send(data.to_vec());
            },
            move |err| {
                eprintln!("❌ Stream error: {}", err);
            },
            Some(Duration::from_secs(10)),
        )?;
        
        stream.play()?;
        println!("🎙️ Recording for {} seconds...", duration_seconds);
        
        std::thread::sleep(Duration::from_millis(200));
        
        let mut audio_data = Vec::new();
        let start = std::time::Instant::now();
        let duration = Duration::from_secs(duration_seconds);
        
        // Ждем первый чанк
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(data) => audio_data.extend(data),
            Err(_) => {
                stream.pause()?;
                return Err(anyhow!("No audio data received"));
            }
        }
        
        // Собираем остальное
        while start.elapsed() < duration {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(data) => audio_data.extend(data),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        
        stream.pause()?;
        
        if audio_data.is_empty() {
            return Err(anyhow!("No audio data received"));
        }
        
        // Ресемплинг до 16kHz если нужно
        let audio_data = if sample_rate != 16000 {
            resample_audio(&audio_data, sample_rate, 16000)
        } else {
            audio_data
        };
        
        // Конвертация в моно
        let audio_data = if channels > 1 {
            convert_to_mono(&audio_data, channels as usize)
        } else {
            audio_data
        };
        
        Ok(audio_data)
    }
}

fn resample_audio(data: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return data.to_vec();
    }
    
    let ratio = to_rate as f32 / from_rate as f32;
    let new_len = (data.len() as f32 * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);
    
    for i in 0..new_len {
        let pos = i as f32 / ratio;
        let pos_int = pos as usize;
        let pos_frac = pos - pos_int as f32;
        
        if pos_int + 1 < data.len() {
            let sample = data[pos_int] * (1.0 - pos_frac) + data[pos_int + 1] * pos_frac;
            resampled.push(sample);
        } else if pos_int < data.len() {
            resampled.push(data[pos_int]);
        }
    }
    
    resampled
}

fn convert_to_mono(data: &[f32], channels: usize) -> Vec<f32> {
    let mut mono = Vec::with_capacity(data.len() / channels);
    for chunk in data.chunks(channels) {
        let sum: f32 = chunk.iter().sum();
        mono.push(sum / channels as f32);
    }
    mono
}