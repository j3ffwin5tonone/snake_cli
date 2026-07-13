use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound};

pub struct Sounds {
    eat: Sound,
    game_over: Sound,
    level_up: Sound,
}

impl Sounds {
    pub async fn load() -> Self {
        Sounds {
            eat: load_sound_from_bytes(&generate_beep_wav(880.0, 0.08, 22050))
                .await
                .expect("eat sound"),
            game_over: load_sound_from_bytes(&generate_beep_wav(220.0, 0.35, 22050))
                .await
                .expect("game over sound"),
            level_up: load_sound_from_bytes(&generate_beep_wav(1320.0, 0.15, 22050))
                .await
                .expect("level up sound"),
        }
    }

    pub fn play_eat(&self, enabled: bool) {
        if !enabled {
            return;
        }
        play_sound(
            &self.eat,
            PlaySoundParams {
                looped: false,
                volume: 0.5,
            },
        );
    }

    pub fn play_game_over(&self, enabled: bool) {
        if !enabled {
            return;
        }
        play_sound(
            &self.game_over,
            PlaySoundParams {
                looped: false,
                volume: 0.6,
            },
        );
    }

    pub fn play_level_up(&self, enabled: bool) {
        if !enabled {
            return;
        }
        play_sound(
            &self.level_up,
            PlaySoundParams {
                looped: false,
                volume: 0.55,
            },
        );
    }
}

/// Generate a minimal mono 16-bit PCM WAV beep.
fn generate_beep_wav(frequency: f32, duration_secs: f32, sample_rate: u32) -> Vec<u8> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let data_size = num_samples * 2;
    let mut wav = Vec::with_capacity(44 + data_size);

    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_size as u32).to_le_bytes());

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let envelope = (1.0 - t / duration_secs).max(0.0);
        let sample =
            (t * frequency * 2.0 * std::f32::consts::PI).sin() * envelope * i16::MAX as f32;
        wav.extend_from_slice(&(sample as i16).to_le_bytes());
    }

    wav
}
