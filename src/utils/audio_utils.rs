use anyhow::{anyhow, Result};
use hound::{WavSpec, WavWriter, WavReader};
use std::io::Cursor;

/// Конвертирует вектор f32 (значения от -1.0 до 1.0) в WAV формат
pub fn f32_to_wav(audio_data: &[f32], sample_rate: u32, channels: u16) -> Result<Vec<u8>> {
    if audio_data.is_empty() {
        return Err(anyhow!("Cannot convert empty audio data to WAV"));
    }

    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut cursor, spec)?;
        
        for &sample in audio_data {
            let clamped = sample.clamp(-1.0, 1.0);
            let sample_i16 = (clamped * 32767.0) as i16;
            writer.write_sample(sample_i16)?;
        }
        
        writer.finalize()?;
    }
    
    Ok(cursor.into_inner())
}

/// Конвертирует WAV данные в вектор f32 (значения от -1.0 до 1.0)
pub fn wav_to_f32(wav_data: &[u8]) -> Result<Vec<f32>> {
    if wav_data.is_empty() {
        return Err(anyhow!("Cannot convert empty WAV data"));
    }

    let cursor = Cursor::new(wav_data);
    let mut reader = WavReader::new(cursor)?;
    
    let spec = reader.spec();
    if spec.bits_per_sample != 16 {
        return Err(anyhow!("Only 16-bit PCM WAV is supported"));
    }
    
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| {
            match s {
                Ok(sample) => Ok(sample as f32 / 32767.0),
                Err(e) => Err(anyhow::anyhow!("WAV read error: {}", e)),
            }
        })
        .collect::<Result<Vec<f32>>>()?;
    
    Ok(samples)
}

/// Сохраняет аудио данные в WAV файл
pub fn save_wav(audio_data: &[f32], file_path: &str, sample_rate: u32, channels: u16) -> Result<()> {
    let wav_data = f32_to_wav(audio_data, sample_rate, channels)?;
    std::fs::write(file_path, wav_data)?;
    Ok(())
}

/// Загружает WAV файл в вектор f32
pub fn load_wav(file_path: &str) -> Result<Vec<f32>> {
    let wav_data = std::fs::read(file_path)?;
    wav_to_f32(&wav_data)
}

/// Приводит аудио к нужной частоте дискретизации (простой ресемплинг)
pub fn resample(audio_data: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return audio_data.to_vec();
    }
    
    let ratio = to_rate as f32 / from_rate as f32;
    let new_len = (audio_data.len() as f32 * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);
    
    for i in 0..new_len {
        let pos = i as f32 / ratio;
        let pos_int = pos as usize;
        let pos_frac = pos - pos_int as f32;
        
        if pos_int + 1 < audio_data.len() {
            let sample = audio_data[pos_int] * (1.0 - pos_frac) 
                       + audio_data[pos_int + 1] * pos_frac;
            resampled.push(sample);
        } else if pos_int < audio_data.len() {
            resampled.push(audio_data[pos_int]);
        }
    }
    
    resampled
}