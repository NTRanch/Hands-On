//! settings — the player's persisted preferences: how
//! strongly each host responds to stimulation, plus sound.
//!
//! The store front-ends share (the egui app's Settings
//! screen, the TUI) is a [`GameSettings`]: for each host, a
//! sensitivity gain and a response threshold — the same two
//! axes the P1 live dial exposes, because they are the same
//! values: the dial reads and writes this store, and a host's
//! live sim profile is derived from it
//! ([`GameSettings::effective_profile`]). Defaults are the
//! sim's tuned per-host profiles from the level registry;
//! ranges are the bounds both front-ends' dials already
//! clamp to. Everything else about a host's profile (her
//! reservoir, her refractory) stays hers — the settings tune
//! stimulation response only.
//!
//! U5 note: nothing in this module touches STOP, aftercare,
//! the limiter, or any safety surface. It carries two f64s
//! per host and the sound toggle; it cannot gate anything.
//!
//! The file format is a line-oriented text document with a
//! version header — human-readable, diffable, and parsed
//! strictly: a malformed file is a typed refusal, never a
//! panic and never a silent half-load. Out-of-range numbers
//! are clamped on load (a hand edit should not brick the
//! game), non-finite numbers are corruption.

use std::fmt;

use milking_core::sim::SensitivityProfile;

use crate::level::{self, HostGirl};

/// Version of the settings file format.
pub const SETTINGS_VERSION: u32 = 1;

/// Sane bounds for the sensitivity gain — the range both
/// front-ends' dials offer (the sim default is 1.0).
pub const GAIN_MIN: f64 = 0.1;
/// See [`GAIN_MIN`].
pub const GAIN_MAX: f64 = 4.0;
/// Sane bounds for the response threshold, in meter points
/// (the sim default is 96 of 100).
pub const THRESHOLD_MIN: f64 = 20.0;
/// See [`THRESHOLD_MIN`].
pub const THRESHOLD_MAX: f64 = 100.0;

/// One host's tuned stimulation response: how much each
/// touch counts, and how full her meter must be to tip over.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostTuning {
    /// Sensitivity gain multiplier.
    pub gain: f64,
    /// Climax threshold, in meter points.
    pub threshold: f64,
}

impl HostTuning {
    /// The tuning a profile ships with.
    #[must_use]
    pub fn from_profile(profile: SensitivityProfile) -> Self {
        Self {
            gain: profile.gain,
            threshold: profile.threshold,
        }
    }

    /// This tuning held inside the sane bounds.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            gain: clamp(self.gain, GAIN_MIN, GAIN_MAX),
            threshold: clamp(self.threshold, THRESHOLD_MIN, THRESHOLD_MAX),
        }
    }
}

fn clamp(v: f64, lo: f64, hi: f64) -> f64 {
    if v.is_finite() { v.clamp(lo, hi) } else { lo }
}

/// The game's persisted settings: per-host stimulation
/// response + sound. One store, every front-end.
#[derive(Clone, Debug, PartialEq)]
pub struct GameSettings {
    /// Per-host tunings, keyed by host name. May carry hosts
    /// the current build doesn't know (an older save from a
    /// bigger cast); they ride along untouched.
    pub tunings: Vec<(String, HostTuning)>,
    /// Sound on/off (presentation only).
    pub sound_enabled: bool,
    /// Sound volume, 0.0..=1.0.
    pub sound_volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self::defaults()
    }
}

impl GameSettings {
    /// The tuned defaults: every registered host at the
    /// profile the sim ships for her, sound on at 0.8 — the
    /// audio defaults both front-ends already used.
    #[must_use]
    pub fn defaults() -> Self {
        let mut tunings: Vec<(String, HostTuning)> = Vec::new();
        for desc in level::registry() {
            if !tunings.iter().any(|(n, _)| n == desc.host.name) {
                tunings.push((
                    desc.host.name.to_string(),
                    HostTuning::from_profile(desc.host.profile),
                ));
            }
        }
        Self {
            tunings,
            sound_enabled: true,
            sound_volume: 0.8,
        }
    }

    /// The tuning in force for a host: her stored value
    /// (clamped) when present, else the profile the registry
    /// ships for her, else the sim's global default.
    #[must_use]
    pub fn tuning_for(&self, host_name: &str) -> HostTuning {
        if let Some((_, t)) = self.tunings.iter().find(|(n, _)| n == host_name) {
            return t.clamped();
        }
        registry_default_tuning(host_name)
            .unwrap_or_else(|| HostTuning::from_profile(SensitivityProfile::default()))
    }

    /// The sim profile a host plays with under these
    /// settings: her own profile, with gain + threshold
    /// replaced by her tuning. Her reservoir capacity/rate
    /// and refractory stay exactly as the sim tuned them.
    #[must_use]
    pub fn effective_profile(&self, host: &HostGirl) -> SensitivityProfile {
        let t = self.tuning_for(host.name);
        SensitivityProfile {
            gain: t.gain,
            threshold: t.threshold,
            ..host.profile
        }
    }

    /// Set a host's tuning (clamped to the sane bounds).
    pub fn set_tuning(&mut self, host_name: &str, tuning: HostTuning) {
        let tuning = tuning.clamped();
        if let Some((_, t)) = self.tunings.iter_mut().find(|(n, _)| n == host_name) {
            *t = tuning;
        } else {
            self.tunings.push((host_name.to_string(), tuning));
        }
    }

    /// Return one host to the profile the sim ships for her.
    pub fn reset_host(&mut self, host_name: &str) {
        let d = registry_default_tuning(host_name)
            .unwrap_or_else(|| HostTuning::from_profile(SensitivityProfile::default()));
        self.set_tuning(host_name, d);
    }

    /// Return every host to her shipped profile. Sound is
    /// not touched.
    pub fn reset_all_tunings(&mut self) {
        self.tunings = Self::defaults().tunings;
    }

    /// Serialize to the settings file format.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = format!("HANDSON-SETTINGS {SETTINGS_VERSION}\n");
        out.push_str(&format!("sound-enabled: {}\n", self.sound_enabled));
        out.push_str(&format!("sound-volume: {}\n", self.sound_volume));
        for (name, t) in &self.tunings {
            out.push_str(&format!("tuning: {name} {} {}\n", t.gain, t.threshold));
        }
        out
    }

    /// Parse a settings file. Malformed content is a typed
    /// refusal; out-of-range numbers are clamped, not
    /// refused (see the module header).
    pub fn from_text(text: &str) -> Result<GameSettings, SettingsError> {
        let mut lines = text.lines();
        let header = lines.next().ok_or_else(|| corrupt("empty settings file"))?;
        let Some(version) = header.strip_prefix("HANDSON-SETTINGS ") else {
            return Err(corrupt("missing settings header"));
        };
        let version: u32 = version
            .trim()
            .parse()
            .map_err(|_| corrupt("unreadable settings version"))?;
        if version != SETTINGS_VERSION {
            return Err(SettingsError::UnsupportedVersion(version));
        }
        let mut settings = GameSettings::defaults();
        settings.tunings.clear();
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once(':') else {
                return Err(corrupt("line without a key"));
            };
            let value = value.trim();
            match key.trim() {
                "sound-enabled" => {
                    settings.sound_enabled = value
                        .parse()
                        .map_err(|_| corrupt("sound-enabled is not a boolean"))?;
                }
                "sound-volume" => {
                    let v: f32 = value
                        .parse()
                        .map_err(|_| corrupt("sound-volume is not a number"))?;
                    if !v.is_finite() {
                        return Err(corrupt("sound-volume is not finite"));
                    }
                    settings.sound_volume = v.clamp(0.0, 1.0);
                }
                "tuning" => {
                    let mut parts = value.split_whitespace();
                    let name = parts
                        .next()
                        .ok_or_else(|| corrupt("tuning without a host"))?;
                    let gain: f64 = parts
                        .next()
                        .and_then(|p| p.parse().ok())
                        .ok_or_else(|| corrupt("tuning gain is not a number"))?;
                    let threshold: f64 = parts
                        .next()
                        .and_then(|p| p.parse().ok())
                        .ok_or_else(|| corrupt("tuning threshold is not a number"))?;
                    if !gain.is_finite() || !threshold.is_finite() {
                        return Err(corrupt("tuning is not finite"));
                    }
                    settings.set_tuning(
                        level::canonical_host_name(name),
                        HostTuning { gain, threshold },
                    );
                }
                // Unknown keys are ignored: a file written by
                // a newer build stays loadable here.
                _ => {}
            }
        }
        Ok(settings)
    }
}

/// The tuning the registry ships for a host name, if the
/// registry knows her.
fn registry_default_tuning(host_name: &str) -> Option<HostTuning> {
    level::registry()
        .iter()
        .find(|d| d.host.name == host_name)
        .map(|d| HostTuning::from_profile(d.host.profile))
}

/// Why a settings file was refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettingsError {
    /// The file is not a well-formed settings document.
    Corrupt(String),
    /// The file was written under another format version.
    UnsupportedVersion(u32),
}

fn corrupt(why: &str) -> SettingsError {
    SettingsError::Corrupt(why.to_string())
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::Corrupt(why) => write!(f, "settings file is corrupt: {why}"),
            SettingsError::UnsupportedVersion(v) => {
                write!(
                    f,
                    "settings version {v} is not supported (this build reads {SETTINGS_VERSION})"
                )
            }
        }
    }
}

impl std::error::Error for SettingsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_sims_tuned_profiles() {
        let s = GameSettings::defaults();
        // The five hosts, at the registry's tuned values.
        assert_eq!(
            s.tuning_for("Opal"),
            HostTuning {
                gain: 1.0,
                threshold: 96.0
            }
        );
        assert_eq!(
            s.tuning_for("Sable"),
            HostTuning {
                gain: 2.6,
                threshold: 42.0
            }
        );
        assert_eq!(
            s.tuning_for("Vega"),
            HostTuning {
                gain: 0.45,
                threshold: 98.0
            }
        );
        assert_eq!(
            s.tuning_for("Mireille"),
            HostTuning {
                gain: 1.4,
                threshold: 65.0
            }
        );
        assert_eq!(
            s.tuning_for("Vigil"),
            HostTuning {
                gain: 0.9,
                threshold: 85.0
            }
        );
        assert!(s.sound_enabled);
        assert_eq!(s.sound_volume, 0.8);
    }

    #[test]
    fn effective_profile_replaces_only_the_two_tuned_axes() {
        let mut s = GameSettings::defaults();
        s.set_tuning(
            "Opal",
            HostTuning {
                gain: 3.0,
                threshold: 30.0,
            },
        );
        let opal = level::registry()[0].host;
        let eff = s.effective_profile(&opal);
        assert_eq!(eff.gain, 3.0);
        assert_eq!(eff.threshold, 30.0);
        // Reservoir behaviour and refractory stay hers.
        assert_eq!(eff.reservoir_capacity, opal.profile.reservoir_capacity);
        assert_eq!(eff.reservoir_rate, opal.profile.reservoir_rate);
        assert_eq!(eff.refractory_ticks, opal.profile.refractory_ticks);
    }

    #[test]
    fn tuning_writes_clamp_to_the_sane_bounds() {
        let mut s = GameSettings::defaults();
        s.set_tuning(
            "Opal",
            HostTuning {
                gain: 99.0,
                threshold: -5.0,
            },
        );
        assert_eq!(
            s.tuning_for("Opal"),
            HostTuning {
                gain: GAIN_MAX,
                threshold: THRESHOLD_MIN
            }
        );
        s.set_tuning(
            "Opal",
            HostTuning {
                gain: f64::NAN,
                threshold: 55.0,
            },
        );
        assert!(s.tuning_for("Opal").gain.is_finite());
    }

    #[test]
    fn legacy_settings_with_the_old_host_name_migrate() {
        // D-195 migration: a settings file written before the
        // rename keys her tuning by her old name. It loads
        // onto Opal; the next write persists the new name.
        let s = GameSettings::from_text("HANDSON-SETTINGS 1\ntuning: Betsy 2.0 50\n")
            .expect("legacy settings must parse");
        assert_eq!(
            s.tuning_for("Opal"),
            HostTuning {
                gain: 2.0,
                threshold: 50.0
            }
        );
        assert!(s.to_text().contains("tuning: Opal 2 50\n"));
    }

    #[test]
    fn text_round_trip_preserves_everything() {
        let mut s = GameSettings::defaults();
        s.set_tuning(
            "Sable",
            HostTuning {
                gain: 1.25,
                threshold: 71.5,
            },
        );
        s.sound_enabled = false;
        s.sound_volume = 0.35;
        let back = GameSettings::from_text(&s.to_text()).expect("own format must parse");
        assert_eq!(back, s);
    }

    #[test]
    fn reset_restores_shipped_tunings_but_not_sound() {
        let mut s = GameSettings::defaults();
        s.set_tuning(
            "Vega",
            HostTuning {
                gain: 4.0,
                threshold: 20.0,
            },
        );
        s.sound_volume = 0.1;
        s.reset_host("Vega");
        assert_eq!(
            s.tuning_for("Vega"),
            HostTuning {
                gain: 0.45,
                threshold: 98.0
            }
        );
        s.set_tuning(
            "Vega",
            HostTuning {
                gain: 4.0,
                threshold: 20.0,
            },
        );
        s.reset_all_tunings();
        assert_eq!(s.tunings, GameSettings::defaults().tunings);
        assert_eq!(s.sound_volume, 0.1, "sound survives a tuning reset");
    }

    #[test]
    fn corrupt_settings_are_refused_not_half_loaded() {
        for bad in [
            "",
            "SETTINGS 1\n",
            "HANDSON-SETTINGS x\n",
            "HANDSON-SETTINGS 1\nsound-volume: loud\n",
            "HANDSON-SETTINGS 1\nsound-enabled: maybe\n",
            "HANDSON-SETTINGS 1\ntuning: Opal NaN 50\n",
            "HANDSON-SETTINGS 1\ntuning: Opal inf 50\n",
            "HANDSON-SETTINGS 1\ntuning: Opal\n",
            "HANDSON-SETTINGS 1\na line with no key\n",
        ] {
            assert!(
                matches!(GameSettings::from_text(bad), Err(SettingsError::Corrupt(_))),
                "input: {bad:?}"
            );
        }
        assert_eq!(
            GameSettings::from_text("HANDSON-SETTINGS 99\n"),
            Err(SettingsError::UnsupportedVersion(99))
        );
    }

    #[test]
    fn out_of_range_file_values_clamp_instead_of_refusing() {
        let s = GameSettings::from_text(
            "HANDSON-SETTINGS 1\nsound-volume: 7.5\ntuning: Opal 400 0.5\n",
        )
        .expect("extreme but well-formed values load, clamped");
        assert_eq!(s.sound_volume, 1.0);
        assert_eq!(
            s.tuning_for("Opal"),
            HostTuning {
                gain: GAIN_MAX,
                threshold: THRESHOLD_MIN
            }
        );
    }
}
