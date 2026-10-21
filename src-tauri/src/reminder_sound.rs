use rodio::DeviceSinkBuilder;
use std::{
    io::Cursor,
    time::{Duration, Instant},
};

const CHIME: &[u8] = include_bytes!("../sounds/reminder.wav");

pub fn play() {
    if let Err(error) = std::thread::Builder::new()
        .name("reminder-sound".into())
        .spawn(|| {
            if let Err(error) = play_chime() {
                log::warn!("Could not play reminder sound: {error}");
            }
        })
    {
        log::warn!("Could not start reminder sound: {error}");
    }
}

fn play_chime() -> Result<(), Box<dyn std::error::Error>> {
    // Use only the user's default output; falling back can expose sound on another device.
    let mut output = DeviceSinkBuilder::from_default_device()?
        .with_error_callback(|error| log::warn!("Reminder audio output failed: {error}"))
        .open_stream()?;
    output.log_on_drop(false);
    let player = rodio::play(output.mixer(), Cursor::new(CHIME))?;
    let deadline = Instant::now() + Duration::from_secs(3);
    while !player.empty() {
        if Instant::now() >= deadline {
            return Err("Reminder audio output did not finish playback".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::{Decoder, Source};

    #[test]
    fn bundled_chime_decodes_to_a_brief_finite_sound() {
        let decoder = Decoder::new(Cursor::new(CHIME)).unwrap();
        let sample_rate = decoder.sample_rate().get();
        let channels = u32::from(decoder.channels().get());
        let duration = decoder.total_duration().unwrap();
        assert!(duration > Duration::from_millis(500));
        assert!(duration < Duration::from_secs(1));

        let samples: Vec<_> = decoder
            .take((sample_rate * channels * 2) as usize)
            .collect();
        assert_eq!(samples.len(), (sample_rate * channels * 72 / 100) as usize);
        assert!(samples.iter().any(|sample| sample.abs() > 0.1));
        assert!(
            samples
                .iter()
                .all(|sample| sample.is_finite() && sample.abs() <= 1.0)
        );
    }
}
