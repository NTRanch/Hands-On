//! audio — the game's sound: synthesized SFX + ambient beds.
//!
//! Assets are small WAVs rendered offline from the game's own
//! composition tables (MIDI lane; the generator + metrics live
//! with the build record) and embedded with `include_bytes!`.
//! Playback runs through rodio/cpal — the estate's first audio
//! backend dependency, flagged as such: cpal reaches the OS
//! audio stack through platform crates (ALSA on Linux, Oboe on
//! Android), which are FFI by nature. Everything above the
//! backend — cue mapping, settings, bed selection, the WAV
//! inventory — is pure and unit-tested without a device.
//!
//! Failure posture: no audio device (headless CI, a muted VM)
//! is a normal state, not an error. The engine degrades to a
//! silent no-op and says so via `backend_live()`.
//!
//! U5 note: sound settings are presentation only. Nothing in
//! this module can gate, delay, or condition stop / safeword /
//! safety behaviour — it owns no game state at all.

use std::io::Cursor;

use milking_core::sim::Outcome;

/// One sound effect, named by the game event it voices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cue {
    Squeeze,
    SqueezeHard,
    BeatPerfect,
    BeatMiss,
    Spill,
    Caress,
    Success,
    Failure,
    Unlock,
    Climax,
    GatePass,
    GateFail,
    UiClick,
    UiBack,
}

impl Cue {
    /// Every cue (the inventory test walks this list).
    pub const ALL: [Cue; 14] = [
        Cue::Squeeze,
        Cue::SqueezeHard,
        Cue::BeatPerfect,
        Cue::BeatMiss,
        Cue::Spill,
        Cue::Caress,
        Cue::Success,
        Cue::Failure,
        Cue::Unlock,
        Cue::Climax,
        Cue::GatePass,
        Cue::GateFail,
        Cue::UiClick,
        Cue::UiBack,
    ];

    /// The embedded WAV for this cue.
    #[must_use]
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Cue::Squeeze => include_bytes!("../assets/audio/squeeze.wav"),
            Cue::SqueezeHard => include_bytes!("../assets/audio/squeeze_hard.wav"),
            Cue::BeatPerfect => include_bytes!("../assets/audio/beat_perfect.wav"),
            Cue::BeatMiss => include_bytes!("../assets/audio/beat_miss.wav"),
            Cue::Spill => include_bytes!("../assets/audio/spill.wav"),
            Cue::Caress => include_bytes!("../assets/audio/caress.wav"),
            Cue::Success => include_bytes!("../assets/audio/success.wav"),
            Cue::Failure => include_bytes!("../assets/audio/failure.wav"),
            Cue::Unlock => include_bytes!("../assets/audio/unlock.wav"),
            Cue::Climax => include_bytes!("../assets/audio/climax.wav"),
            Cue::GatePass => include_bytes!("../assets/audio/gate_pass.wav"),
            Cue::GateFail => include_bytes!("../assets/audio/gate_fail.wav"),
            Cue::UiClick => include_bytes!("../assets/audio/ui_click.wav"),
            Cue::UiBack => include_bytes!("../assets/audio/ui_back.wav"),
        }
    }
}

/// The cue a sim outcome earns (P1's feedback grammar).
#[must_use]
pub fn cue_for_outcome(outcome: &Outcome, hard: bool) -> Cue {
    match outcome {
        Outcome::Perfect => Cue::BeatPerfect,
        Outcome::Good => {
            if hard {
                Cue::SqueezeHard
            } else {
                Cue::Squeeze
            }
        }
        Outcome::Spill => Cue::Spill,
        Outcome::SideApplied => Cue::Caress,
        Outcome::Climax { .. } => Cue::Climax,
    }
}

/// The ambient bed for a room/screen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Bed {
    Menu,
    Playroom,
    Studio,
}

impl Bed {
    /// Every bed (the inventory test walks this list).
    pub const ALL: [Bed; 3] = [Bed::Menu, Bed::Playroom, Bed::Studio];

    /// The embedded loop WAV for this bed.
    #[must_use]
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Bed::Menu => include_bytes!("../assets/audio/bed_menu.wav"),
            Bed::Playroom => include_bytes!("../assets/audio/bed_playroom.wav"),
            Bed::Studio => include_bytes!("../assets/audio/bed_studio.wav"),
        }
    }
}

/// Player-facing sound settings. Presentation only (see the
/// module header's U5 note).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioSettings {
    pub enabled: bool,
    /// 0.0..=1.0, clamped on every write through `set_volume`.
    pub volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 0.8,
        }
    }
}

impl AudioSettings {
    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.clamp(0.0, 1.0);
    }
}

/// Parsed facts about an embedded WAV (the inventory the tests
/// and the build record share).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WavInfo {
    pub channels: u16,
    pub sample_rate: u32,
    pub frames: u32,
}

impl WavInfo {
    #[must_use]
    pub fn duration_s(&self) -> f32 {
        self.frames as f32 / self.sample_rate as f32
    }
}

/// Read the RIFF/fmt/data headers of a 16-bit PCM WAV.
/// Returns None on anything that is not exactly that.
#[must_use]
pub fn wav_info(bytes: &[u8]) -> Option<WavInfo> {
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }
    let channels = u16::from_le_bytes([bytes[22], bytes[23]]);
    let sample_rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    let bits = u16::from_le_bytes([bytes[34], bytes[35]]);
    if bits != 16 || channels == 0 || sample_rate == 0 {
        return None;
    }
    // Walk sub-chunks to the data chunk (fmt may be padded).
    let mut pos = 12usize;
    while pos + 8 <= bytes.len() {
        let size = u32::from_le_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;
        if &bytes[pos..pos + 4] == b"data" {
            let frames = (size / 2) as u32 / u32::from(channels);
            return Some(WavInfo {
                channels,
                sample_rate,
                frames,
            });
        }
        pos += 8 + size + (size % 2);
    }
    None
}

/// The live backend: stream + one sink per layer. Kept in the
/// engine; dropping it stops all sound.
struct Backend {
    _stream: rodio::OutputStream,
    music: rodio::Sink,
    sfx: rodio::Sink,
}

/// The sound engine: settings + optional live backend + the
/// bed the player should be hearing.
pub struct AudioEngine {
    pub settings: AudioSettings,
    backend: Option<Backend>,
    desired_bed: Option<Bed>,
    playing_bed: Option<Bed>,
}

impl AudioEngine {
    /// Start the engine, opening the default output device if
    /// one exists. Headless is fine (see the module header).
    #[must_use]
    pub fn new(settings: AudioSettings) -> Self {
        let backend = rodio::OutputStream::try_default()
            .ok()
            .and_then(|(stream, handle)| {
                let music = rodio::Sink::try_new(&handle).ok()?;
                let sfx = rodio::Sink::try_new(&handle).ok()?;
                music.set_volume(settings.volume);
                sfx.set_volume(settings.volume);
                Some(Backend {
                    _stream: stream,
                    music,
                    sfx,
                })
            });
        Self {
            settings,
            backend,
            desired_bed: None,
            playing_bed: None,
        }
    }

    /// True when sound is actually reaching a device.
    #[must_use]
    pub fn backend_live(&self) -> bool {
        self.backend.is_some()
    }

    /// Apply the current settings to the sinks (after the UI
    /// edits them): volume everywhere; disabled silences the
    /// bed, re-enabling restores the desired one.
    pub fn apply_settings(&mut self) {
        if let Some(b) = &self.backend {
            b.music.set_volume(self.settings.volume);
            b.sfx.set_volume(self.settings.volume);
        }
        if !self.settings.enabled {
            if let Some(b) = &self.backend {
                b.music.stop();
            }
            self.playing_bed = None;
        } else if self.playing_bed != self.desired_bed {
            self.start_bed();
        }
    }

    /// Ask for a room's bed; starts it if it is not already the
    /// one playing. Silent when disabled or headless.
    pub fn set_bed(&mut self, bed: Bed) {
        self.desired_bed = Some(bed);
        if self.settings.enabled && self.playing_bed != Some(bed) {
            self.start_bed();
        }
    }

    fn start_bed(&mut self) {
        let (Some(bed), Some(b)) = (self.desired_bed, &self.backend) else {
            return;
        };
        b.music.stop();
        if let Ok(dec) = rodio::Decoder::new(Cursor::new(bed.bytes())) {
            use rodio::Source;
            b.music.append(dec.repeat_infinite());
            b.music.set_volume(self.settings.volume);
            self.playing_bed = Some(bed);
        }
    }

    /// Fire one cue. No-op when disabled or headless.
    pub fn play(&mut self, cue: Cue) {
        if !self.settings.enabled {
            return;
        }
        let Some(b) = &self.backend else { return };
        if let Ok(dec) = rodio::Decoder::new(Cursor::new(cue.bytes())) {
            b.sfx.append(dec);
        }
    }

    /// The bed the player should hear now (test observability).
    #[must_use]
    pub fn desired_bed(&self) -> Option<Bed> {
        self.desired_bed
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new(AudioSettings::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cue_asset_is_a_valid_wav_in_range() {
        for cue in Cue::ALL {
            let info = wav_info(cue.bytes()).expect("cue asset must parse");
            assert_eq!(info.sample_rate, 22050, "{cue:?}");
            assert_eq!(info.channels, 1, "{cue:?}");
            let d = info.duration_s();
            assert!((0.03..=2.0).contains(&d), "{cue:?} duration {d}");
        }
    }

    #[test]
    fn every_bed_asset_is_a_stereo_loop_in_range() {
        for bed in Bed::ALL {
            let info = wav_info(bed.bytes()).expect("bed asset must parse");
            assert_eq!(info.sample_rate, 22050, "{bed:?}");
            assert_eq!(info.channels, 2, "{bed:?}");
            let d = info.duration_s();
            assert!((10.0..=14.0).contains(&d), "{bed:?} duration {d}");
        }
    }

    #[test]
    fn outcome_mapping_covers_the_grammar() {
        assert_eq!(cue_for_outcome(&Outcome::Perfect, false), Cue::BeatPerfect);
        assert_eq!(cue_for_outcome(&Outcome::Good, false), Cue::Squeeze);
        assert_eq!(cue_for_outcome(&Outcome::Good, true), Cue::SqueezeHard);
        assert_eq!(cue_for_outcome(&Outcome::Spill, false), Cue::Spill);
        assert_eq!(cue_for_outcome(&Outcome::SideApplied, false), Cue::Caress);
        assert_eq!(
            cue_for_outcome(
                &Outcome::Climax {
                    intensity: 0.5,
                    burst: 3.0
                },
                false
            ),
            Cue::Climax
        );
    }

    #[test]
    fn volume_clamps_and_engine_survives_headless() {
        let mut s = AudioSettings::default();
        s.set_volume(1.7);
        assert_eq!(s.volume, 1.0);
        s.set_volume(-0.2);
        assert_eq!(s.volume, 0.0);
        // No device in CI is a legal state: every call is a no-op,
        // never a panic.
        let mut e = AudioEngine::new(s);
        e.play(Cue::Squeeze);
        e.set_bed(Bed::Menu);
        assert_eq!(e.desired_bed(), Some(Bed::Menu));
        e.settings.enabled = false;
        e.apply_settings();
        e.play(Cue::Climax);
    }

    #[test]
    fn wav_info_rejects_non_wavs() {
        assert!(wav_info(b"not a wav at all, not even close........").is_none());
        assert!(wav_info(&[]).is_none());
    }
}
