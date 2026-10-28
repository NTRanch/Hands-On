//! save — save slots, the campaign file, and level
//! progression: the persistence layer both front-ends
//! share.
//!
//! Three named slots plus one campaign file (the Continue
//! state the shells maintain as play happens). A save is a
//! line-oriented text document with a version header:
//! metadata (level, host, a progress summary, a timestamp),
//! the player's progression, and — for the level whose host
//! runs on the live sim — the sim's state as an opaque,
//! separately-versioned payload from `milking-core`
//! ([`SimPayload`]). Shells carry the payload bytes without
//! interpreting them; only `milking-core` reads them back.
//!
//! Discipline: writes are atomic (write a temp file, rename
//! over the target — a crashed write leaves the old save or
//! the new one, never a torn one); a corrupt or
//! foreign-version file is a typed refusal
//! ([`SaveError`]), never a panic and never a partial load.
//! The core stays clock-free — front-ends supply the
//! timestamp — and the only I/O in the crate lives in this
//! module and in `settings`, on purpose: the store is the
//! documented exception to the library's no-I/O rule.
//!
//! U5 note: a save records progression and sim state. It
//! cannot carry consent, safety, or limiter state, because
//! none exists to carry — those surfaces are always live.

use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use milking_core::sim::{Config, STATE_VERSION, Sim, StateError};

use crate::level::{LevelDescriptor, Track, canonical_host_name};
use crate::settings::{GameSettings, SettingsError};

/// Version of the save file format.
pub const SAVE_FORMAT_VERSION: u32 = 1;

/// How many named save slots the game offers.
pub const SLOT_COUNT: u8 = 3;

/// Identity of one level: its track and number (P1, S3, …).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LevelKey {
    /// Track the level sits on.
    pub track: Track,
    /// 1-based level number within the track.
    pub id: u8,
}

impl LevelKey {
    /// The key of a registered level.
    #[must_use]
    pub fn of(desc: &LevelDescriptor) -> Self {
        Self {
            track: desc.track,
            id: desc.id,
        }
    }

    /// Display code: `P1`, `S3`.
    #[must_use]
    pub fn code(self) -> String {
        let letter = match self.track {
            Track::Playroom => 'P',
            Track::Study => 'S',
        };
        format!("{letter}{}", self.id)
    }

    /// Parse a display code (`P1`, `S3`); anything else is
    /// not a level key.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut chars = text.chars();
        let track = match chars.next() {
            Some('P') => Track::Playroom,
            Some('S') => Track::Study,
            _ => return None,
        };
        let id: u8 = chars.as_str().parse().ok()?;
        if id == 0 {
            return None;
        }
        Some(Self { track, id })
    }

    fn sort_rank(self) -> (u8, u8) {
        let track = match self.track {
            Track::Playroom => 0,
            Track::Study => 1,
        };
        (track, self.id)
    }
}

/// Whether a level is locked, open, or done, derived from
/// real progression (see [`Progression`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LevelState {
    /// Its prerequisite is not complete yet.
    Locked,
    /// It can be entered now.
    Available,
    /// It has been completed.
    Completed,
}

/// The player's progression: which levels are complete.
/// Unlock rule: within each track the levels form a chain —
/// the first is always open, and each later level opens
/// when its predecessor completes. The two tracks chain
/// independently (the Study is not gated on the Playroom).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Progression {
    completed: Vec<LevelKey>,
}

impl Progression {
    /// No levels complete.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from raw keys (load path): sorted, de-duplicated.
    #[must_use]
    pub fn from_keys(mut keys: Vec<LevelKey>) -> Self {
        keys.sort_by_key(|k| k.sort_rank());
        keys.dedup();
        Self { completed: keys }
    }

    /// Mark a level complete. Returns true when this is new.
    pub fn mark_completed(&mut self, key: LevelKey) -> bool {
        if self.is_completed(key) {
            return false;
        }
        self.completed.push(key);
        self.completed.sort_by_key(|k| k.sort_rank());
        true
    }

    /// Whether a level is complete.
    #[must_use]
    pub fn is_completed(&self, key: LevelKey) -> bool {
        self.completed.contains(&key)
    }

    /// How many levels are complete.
    #[must_use]
    pub fn count(&self) -> usize {
        self.completed.len()
    }

    /// The completed keys, in track order.
    #[must_use]
    pub fn completed(&self) -> &[LevelKey] {
        &self.completed
    }

    /// The level that must complete before `desc` opens, if
    /// it is currently locked.
    #[must_use]
    pub fn prerequisite(&self, desc: &LevelDescriptor) -> Option<LevelKey> {
        if desc.id <= 1 {
            return None;
        }
        let prev = LevelKey {
            track: desc.track,
            id: desc.id - 1,
        };
        if self.is_completed(prev) {
            None
        } else {
            Some(prev)
        }
    }

    /// The picker state of one level under this progression.
    #[must_use]
    pub fn state_of(&self, desc: &LevelDescriptor) -> LevelState {
        if self.is_completed(LevelKey::of(desc)) {
            LevelState::Completed
        } else if self.prerequisite(desc).is_none() {
            LevelState::Available
        } else {
            LevelState::Locked
        }
    }
}

/// The sim's state, carried opaquely: a version tag plus
/// the payload bytes from `milking-core`. Shells store and
/// hand back these bytes; only the sim interprets them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimPayload {
    /// State-format version (milking-core's `STATE_VERSION`
    /// at capture time).
    pub version: u32,
    /// The opaque payload.
    pub bytes: Vec<u8>,
}

impl SimPayload {
    /// Capture a sim's state for saving.
    #[must_use]
    pub fn capture(sim: &Sim) -> Self {
        Self {
            version: STATE_VERSION,
            bytes: sim.state_bytes(),
        }
    }

    /// Restore the sim this payload captured, under the
    /// caller's config. A payload from another state
    /// version is a typed refusal from `milking-core`.
    pub fn restore(&self, cfg: Config) -> Result<Sim, StateError> {
        if self.version != STATE_VERSION {
            return Err(StateError::UnsupportedVersion(self.version));
        }
        Sim::from_state_bytes(cfg, &self.bytes)
    }
}

/// One save: a slot file, or the campaign file (same shape).
#[derive(Clone, Debug, PartialEq)]
pub struct SaveGame {
    /// Player-facing slot name.
    pub name: String,
    /// When it was written, seconds since the Unix epoch
    /// (supplied by the front-end — the core reads no clock).
    pub saved_at_unix: u64,
    /// The level the player was in.
    pub level: LevelKey,
    /// That level's host.
    pub host: String,
    /// One line of progress summary, for the slot list.
    pub summary: String,
    /// Progression at save time.
    pub progression: Progression,
    /// The host sim's state, when the level runs one live
    /// (P1 today). Absent for session-layer levels, whose
    /// sittings are not resumable mid-chair.
    pub sim: Option<SimPayload>,
}

impl SaveGame {
    /// A one-line description for slot lists and the
    /// Continue button: level, host, completion, when.
    #[must_use]
    pub fn describe(&self) -> String {
        format!(
            "{} · {} · {}/17 complete · {}",
            self.level.code(),
            self.host,
            self.progression.count(),
            format_unix_utc(self.saved_at_unix),
        )
    }

    /// Serialize to the save file format.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = format!("HANDSON-SAVE {SAVE_FORMAT_VERSION}\n");
        out.push_str(&format!("name: {}\n", one_line(&self.name)));
        out.push_str(&format!("saved-at: {}\n", self.saved_at_unix));
        out.push_str(&format!("level: {}\n", self.level.code()));
        out.push_str(&format!("host: {}\n", one_line(&self.host)));
        out.push_str(&format!("summary: {}\n", one_line(&self.summary)));
        let codes: Vec<String> = self
            .progression
            .completed()
            .iter()
            .map(|k| k.code())
            .collect();
        out.push_str(&format!("completed: {}\n", codes.join(",")));
        if let Some(sim) = &self.sim {
            out.push_str(&format!("sim-version: {}\n", sim.version));
            out.push_str(&format!("sim-hex: {}\n", hex_encode(&sim.bytes)));
        }
        out
    }

    /// Parse a save file. Anything malformed, incomplete, or
    /// from another format version is a typed refusal.
    pub fn from_text(text: &str) -> Result<SaveGame, SaveError> {
        let mut lines = text.lines();
        let header = lines.next().ok_or_else(|| corrupt("empty save file"))?;
        let Some(version) = header.strip_prefix("HANDSON-SAVE ") else {
            return Err(corrupt("missing save header"));
        };
        let version: u32 = version
            .trim()
            .parse()
            .map_err(|_| corrupt("unreadable save version"))?;
        if version != SAVE_FORMAT_VERSION {
            return Err(SaveError::UnsupportedVersion(version));
        }
        let mut name = String::new();
        let mut saved_at_unix: Option<u64> = None;
        let mut level: Option<LevelKey> = None;
        let mut host: Option<String> = None;
        let mut summary = String::new();
        let mut completed: Vec<LevelKey> = Vec::new();
        let mut sim_version: Option<u32> = None;
        let mut sim_hex: Option<String> = None;
        for line in lines {
            let line = line.trim_end();
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once(':') else {
                return Err(corrupt("line without a key"));
            };
            let value = value.trim();
            match key.trim() {
                "name" => name = value.to_string(),
                "saved-at" => {
                    saved_at_unix = Some(
                        value
                            .parse()
                            .map_err(|_| corrupt("saved-at is not a number"))?,
                    );
                }
                "level" => {
                    level = Some(LevelKey::parse(value).ok_or_else(|| corrupt("bad level code"))?);
                }
                "host" => host = Some(canonical_host_name(value).to_string()),
                "summary" => summary = value.to_string(),
                "completed" => {
                    if !value.is_empty() {
                        for code in value.split(',') {
                            completed.push(
                                LevelKey::parse(code.trim())
                                    .ok_or_else(|| corrupt("bad completed code"))?,
                            );
                        }
                    }
                }
                "sim-version" => {
                    sim_version = Some(
                        value
                            .parse()
                            .map_err(|_| corrupt("sim-version is not a number"))?,
                    );
                }
                "sim-hex" => sim_hex = Some(value.to_string()),
                _ => return Err(corrupt("unknown field")),
            }
        }
        let sim = match (sim_version, sim_hex) {
            (Some(version), Some(hex)) => Some(SimPayload {
                version,
                bytes: hex_decode(&hex).ok_or_else(|| corrupt("sim payload is not valid hex"))?,
            }),
            (None, None) => None,
            _ => return Err(corrupt("sim version and payload must appear together")),
        };
        Ok(SaveGame {
            name,
            saved_at_unix: saved_at_unix.ok_or_else(|| corrupt("missing saved-at"))?,
            level: level.ok_or_else(|| corrupt("missing level"))?,
            host: host.ok_or_else(|| corrupt("missing host"))?,
            summary,
            progression: Progression::from_keys(completed),
            sim,
        })
    }
}

/// Why a save, a slot, or the store refused.
#[derive(Debug)]
pub enum SaveError {
    /// The underlying file operation failed.
    Io(io::Error),
    /// The file is not a well-formed save document.
    Corrupt(String),
    /// The file was written under another save format
    /// version.
    UnsupportedVersion(u32),
    /// Slots are numbered 1..=SLOT_COUNT.
    InvalidSlot(u8),
}

fn corrupt(why: &str) -> SaveError {
    SaveError::Corrupt(why.to_string())
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "save store I/O failed: {e}"),
            SaveError::Corrupt(why) => write!(f, "save file is corrupt: {why}"),
            SaveError::UnsupportedVersion(v) => {
                write!(
                    f,
                    "save version {v} is not supported (this build reads {SAVE_FORMAT_VERSION})"
                )
            }
            SaveError::InvalidSlot(n) => {
                write!(f, "slot {n} does not exist (slots are 1..={SLOT_COUNT})")
            }
        }
    }
}

impl std::error::Error for SaveError {}

impl From<io::Error> for SaveError {
    fn from(e: io::Error) -> Self {
        SaveError::Io(e)
    }
}

impl From<SettingsError> for SaveError {
    fn from(e: SettingsError) -> Self {
        match e {
            SettingsError::UnsupportedVersion(v) => SaveError::UnsupportedVersion(v),
            SettingsError::Corrupt(why) => SaveError::Corrupt(why),
        }
    }
}

/// Field values in the text formats are single-line by
/// contract; anything else is flattened on write.
fn one_line(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    let nibble = |c: u8| -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    };
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.as_chunks::<2>().0 {
        out.push((nibble(pair[0])? << 4) | nibble(pair[1])?);
    }
    Some(out)
}

/// Format Unix seconds as `YYYY-MM-DD HH:MM UTC` (the slot
/// list's timestamp). Pure arithmetic — no clock, no locale.
#[must_use]
pub fn format_unix_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let tod = secs % 86_400;
    // Civil date from days since the epoch (Hinnant's
    // algorithm, public domain).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    format!(
        "{year:04}-{m:02}-{d:02} {:02}:{:02} UTC",
        tod / 3600,
        (tod % 3600) / 60
    )
}

/// A process-level data-directory override, set by a
/// front-end that knows the platform's app-data path better
/// than the heuristics (the Android activity sets it from
/// its internal data path at startup).
static DATA_DIR_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Set the data-directory override (see [`data_dir`]).
pub fn set_data_dir_override(path: PathBuf) {
    if let Ok(mut slot) = DATA_DIR_OVERRIDE.lock() {
        *slot = Some(path);
    }
}

/// Where saves and settings live on this platform. In
/// order: the `HANDS_ON_DATA_DIR` environment override
/// (tests, portable installs), the front-end's override
/// ([`set_data_dir_override`]), then the platform's
/// app-data convention.
#[must_use]
pub fn data_dir() -> Option<PathBuf> {
    if let Ok(v) = std::env::var("HANDS_ON_DATA_DIR")
        && !v.is_empty()
    {
        return Some(PathBuf::from(v));
    }
    if let Ok(slot) = DATA_DIR_OVERRIDE.lock()
        && let Some(path) = &*slot
    {
        return Some(path.clone());
    }
    platform_data_dir()
}

#[cfg(target_os = "android")]
fn platform_data_dir() -> Option<PathBuf> {
    // Fallback only: android_main sets the override from the
    // activity's internal data path before this can matter.
    // The path is this app's own (ai.ntranch.handson).
    Some(PathBuf::from("/data/data/ai.ntranch.handson/files"))
}

#[cfg(target_os = "windows")]
fn platform_data_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|base| PathBuf::from(base).join("Hands-On"))
}

#[cfg(target_os = "macos")]
fn platform_data_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support/Hands-On"))
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
fn platform_data_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME")
        && !xdg.is_empty()
    {
        return Some(PathBuf::from(xdg).join("hands-on"));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share/hands-on"))
}

/// The store: the directory saves + settings live in, and
/// the typed operations over it. Missing files are a normal
/// state (a fresh install), never an error; corrupt files
/// are typed refusals.
#[derive(Clone, Debug)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// A store rooted at an explicit directory (tests, or
    /// [`Store::default_location`]).
    #[must_use]
    pub fn at(root: PathBuf) -> Self {
        Self { root }
    }

    /// The platform store, when a data directory exists.
    #[must_use]
    pub fn default_location() -> Option<Self> {
        data_dir().map(Self::at)
    }

    /// The store's root directory.
    #[must_use]
    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    fn settings_path(&self) -> PathBuf {
        self.root.join("settings.txt")
    }

    fn campaign_path(&self) -> PathBuf {
        self.root.join("campaign.save")
    }

    fn slot_path(&self, slot: u8) -> Result<PathBuf, SaveError> {
        if (1..=SLOT_COUNT).contains(&slot) {
            Ok(self.root.join("saves").join(format!("slot{slot}.save")))
        } else {
            Err(SaveError::InvalidSlot(slot))
        }
    }

    /// Load the settings; a missing file is the defaults.
    pub fn load_settings(&self) -> Result<GameSettings, SaveError> {
        match fs::read_to_string(self.settings_path()) {
            Ok(text) => Ok(GameSettings::from_text(&text)?),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(GameSettings::defaults()),
            Err(e) => Err(e.into()),
        }
    }

    /// Persist the settings (atomic write).
    pub fn save_settings(&self, settings: &GameSettings) -> Result<(), SaveError> {
        write_atomic(&self.settings_path(), &settings.to_text())?;
        Ok(())
    }

    /// Load the campaign (the Continue state), if any.
    pub fn load_campaign(&self) -> Result<Option<SaveGame>, SaveError> {
        self.read_save(&self.campaign_path())
    }

    /// Persist the campaign (atomic write).
    pub fn save_campaign(&self, game: &SaveGame) -> Result<(), SaveError> {
        write_atomic(&self.campaign_path(), &game.to_text())?;
        Ok(())
    }

    /// Load one slot, if it holds a save.
    pub fn load_slot(&self, slot: u8) -> Result<Option<SaveGame>, SaveError> {
        self.read_save(&self.slot_path(slot)?)
    }

    /// Write one slot (atomic).
    pub fn save_slot(&self, slot: u8, game: &SaveGame) -> Result<(), SaveError> {
        write_atomic(&self.slot_path(slot)?, &game.to_text())?;
        Ok(())
    }

    /// Metadata for every slot, in slot order — the
    /// Save/Load screen's listing. One corrupt slot refuses
    /// the listing (better a refusal than a slot list that
    /// lies about what is loadable).
    pub fn slot_listing(&self) -> Result<Vec<Option<SaveGame>>, SaveError> {
        let mut out = Vec::with_capacity(SLOT_COUNT as usize);
        for slot in 1..=SLOT_COUNT {
            out.push(self.load_slot(slot)?);
        }
        Ok(out)
    }

    fn read_save(&self, path: &std::path::Path) -> Result<Option<SaveGame>, SaveError> {
        match fs::read_to_string(path) {
            Ok(text) => Ok(Some(SaveGame::from_text(&text)?)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

/// Write `text` to `path` atomically: temp file in the same
/// directory, then rename over the target.
fn write_atomic(path: &std::path::Path, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text)?;
    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(first) => {
            // Some platforms (Windows) will not rename over
            // an existing file; drop the target and retry.
            let _ = fs::remove_file(path);
            fs::rename(&tmp, path).map_err(|_| first)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level;
    use milking_core::sim::SensitivityProfile;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_SEQ: AtomicU32 = AtomicU32::new(0);

    fn temp_store(tag: &str) -> Store {
        let n = TEST_SEQ.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "hands-on-store-test-{}-{tag}-{n}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        Store::at(root)
    }

    fn desc(track: Track, id: u8) -> LevelDescriptor {
        level::registry()
            .into_iter()
            .find(|d| d.track == track && d.id == id)
            .expect("registered level")
    }

    fn sample_game() -> SaveGame {
        let mut progression = Progression::new();
        progression.mark_completed(LevelKey {
            track: Track::Playroom,
            id: 1,
        });
        progression.mark_completed(LevelKey {
            track: Track::Study,
            id: 1,
        });
        SaveGame {
            name: "Evening session".to_string(),
            saved_at_unix: 1_767_225_600,
            level: LevelKey {
                track: Track::Playroom,
                id: 4,
            },
            host: "Mireille".to_string(),
            summary: "gate passed · loops 12".to_string(),
            progression,
            sim: None,
        }
    }

    #[test]
    fn level_key_codes_round_trip() {
        for (track, id, code) in [
            (Track::Playroom, 1, "P1"),
            (Track::Playroom, 9, "P9"),
            (Track::Study, 8, "S8"),
        ] {
            let key = LevelKey { track, id };
            assert_eq!(key.code(), code);
            assert_eq!(LevelKey::parse(code), Some(key));
        }
        for bad in ["", "X1", "P0", "P", "P-1", "p1", "P 1"] {
            assert_eq!(LevelKey::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn picker_states_derive_from_real_progression() {
        let mut p = Progression::new();
        let p1 = desc(Track::Playroom, 1);
        let p2 = desc(Track::Playroom, 2);
        let p3 = desc(Track::Playroom, 3);
        let s1 = desc(Track::Study, 1);
        let s2 = desc(Track::Study, 2);
        // Fresh: each track's first level open, the rest locked.
        assert_eq!(p.state_of(&p1), LevelState::Available);
        assert_eq!(p.state_of(&p2), LevelState::Locked);
        assert_eq!(p.prerequisite(&p2), Some(LevelKey::of(&p1)));
        assert_eq!(p.state_of(&s1), LevelState::Available);
        assert_eq!(p.state_of(&s2), LevelState::Locked);
        // Completing P1 opens P2 (not P3) and completes P1.
        assert!(p.mark_completed(LevelKey::of(&p1)));
        assert!(
            !p.mark_completed(LevelKey::of(&p1)),
            "double-mark is not new"
        );
        assert_eq!(p.state_of(&p1), LevelState::Completed);
        assert_eq!(p.state_of(&p2), LevelState::Available);
        assert_eq!(p.state_of(&p3), LevelState::Locked);
        // The Study chain is independent of the Playroom.
        assert_eq!(p.state_of(&s2), LevelState::Locked);
        p.mark_completed(LevelKey::of(&s1));
        assert_eq!(p.state_of(&s2), LevelState::Available);
        assert_eq!(p.count(), 2);
    }

    #[test]
    fn save_text_round_trips_with_a_sim_payload() {
        let mut sim = Sim::new(
            milking_core::sim::Config::default(),
            SensitivityProfile::default(),
        );
        for i in 0..8u64 {
            let _ = sim.apply(milking_core::sim::InputEvent::Squeeze {
                tick: i * 30,
                hand: milking_core::sim::Hand::Left,
                pressure: 700,
            });
        }
        let mut game = sample_game();
        game.level = LevelKey {
            track: Track::Playroom,
            id: 1,
        };
        game.host = "Opal".to_string();
        game.sim = Some(SimPayload::capture(&sim));
        let back = SaveGame::from_text(&game.to_text()).expect("own format must parse");
        assert_eq!(back, game);
        let restored = back
            .sim
            .as_ref()
            .expect("payload present")
            .restore(milking_core::sim::Config::default())
            .expect("own payload must restore");
        assert_eq!(restored.snapshot(), sim.snapshot());
    }

    #[test]
    fn corrupt_saves_are_typed_refusals() {
        let good = sample_game().to_text();
        for bad in [
            String::new(),
            "SAVE 1\n".to_string(),
            good.replace("HANDSON-SAVE 1", "HANDSON-SAVE 1\ngarbage line"),
            good.replace("level: P4", "level: Q4"),
            good.replace("saved-at: 1767225600", "saved-at: soon"),
            good.replace("completed: P1,S1", "completed: P1,XX"),
            format!("{good}sim-version: 1\n"),
            good.replace("HANDSON-SAVE 1", "HANDSON-SAVE 1\nextra: field"),
        ] {
            assert!(
                matches!(SaveGame::from_text(&bad), Err(SaveError::Corrupt(_))),
                "input: {bad:?}"
            );
        }
        assert!(matches!(
            SaveGame::from_text("HANDSON-SAVE 7\n"),
            Err(SaveError::UnsupportedVersion(7))
        ));
        // A payload captured under another sim version parses
        // as a document but refuses at restore time.
        let mut game = sample_game();
        game.sim = Some(SimPayload {
            version: 99,
            bytes: vec![1, 2, 3],
        });
        let back = SaveGame::from_text(&game.to_text()).expect("document parses");
        assert!(matches!(
            back.sim
                .expect("payload")
                .restore(milking_core::sim::Config::default()),
            Err(StateError::UnsupportedVersion(99))
        ));
        // Non-hex and odd-length payloads refuse at parse.
        let mut game = sample_game();
        game.sim = Some(SimPayload {
            version: 1,
            bytes: vec![0xab],
        });
        let mut text = game.to_text();
        text = text.replace("sim-hex: ab", "sim-hex: abc");
        assert!(matches!(
            SaveGame::from_text(&text),
            Err(SaveError::Corrupt(_))
        ));
    }

    #[test]
    fn legacy_save_with_the_old_host_name_loads_as_opal() {
        // D-195 migration: a save written before the rename
        // persists the P1 hostess's old name. It must load
        // under her new name, and the next write persists it.
        let mut game = sample_game();
        game.level = LevelKey {
            track: Track::Playroom,
            id: 1,
        };
        game.host = "Opal".to_string();
        let legacy_text = game.to_text().replace("host: Opal", "host: Betsy");
        let back = SaveGame::from_text(&legacy_text).expect("legacy save must parse");
        assert_eq!(back.host, "Opal");
        assert!(back.to_text().contains("host: Opal\n"));
    }

    #[test]
    fn store_round_trips_slots_campaign_and_settings() {
        let store = temp_store("roundtrip");
        // Fresh store: nothing saved, settings are defaults.
        assert_eq!(store.load_slot(1).unwrap(), None);
        assert_eq!(store.load_campaign().unwrap(), None);
        assert_eq!(store.load_settings().unwrap(), GameSettings::defaults());
        // Slots are independent; the campaign is its own file.
        let game = sample_game();
        store.save_slot(2, &game).unwrap();
        assert_eq!(store.load_slot(2).unwrap(), Some(game.clone()));
        assert_eq!(store.load_slot(1).unwrap(), None);
        assert_eq!(store.load_campaign().unwrap(), None);
        store.save_campaign(&game).unwrap();
        assert_eq!(store.load_campaign().unwrap(), Some(game.clone()));
        // Overwrite replaces wholesale.
        let mut later = game.clone();
        later.name = "Later".to_string();
        store.save_slot(2, &later).unwrap();
        assert_eq!(
            store.load_slot(2).unwrap().map(|g| g.name),
            Some("Later".to_string())
        );
        // Settings persist through the same store.
        let mut s = GameSettings::defaults();
        s.sound_volume = 0.25;
        store.save_settings(&s).unwrap();
        assert_eq!(store.load_settings().unwrap().sound_volume, 0.25);
        // Atomic writes leave no temp files behind.
        let saves_dir = store.root().join("saves");
        let leftovers: Vec<_> = fs::read_dir(&saves_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp files: {leftovers:?}");
        let _ = fs::remove_dir_all(store.root());
    }

    #[test]
    fn store_refuses_corrupt_files_and_bad_slots() {
        let store = temp_store("refusals");
        fs::create_dir_all(store.root().join("saves")).unwrap();
        fs::write(store.root().join("saves/slot1.save"), "not a save at all").unwrap();
        assert!(matches!(store.load_slot(1), Err(SaveError::Corrupt(_))));
        // The refusal is not a deletion: the file stays for
        // the player to inspect or remove.
        assert!(store.root().join("saves/slot1.save").exists());
        assert!(matches!(store.load_slot(0), Err(SaveError::InvalidSlot(0))));
        assert!(matches!(store.load_slot(4), Err(SaveError::InvalidSlot(4))));
        assert!(matches!(
            store.save_slot(9, &sample_game()),
            Err(SaveError::InvalidSlot(9))
        ));
        let _ = fs::remove_dir_all(store.root());
    }

    #[test]
    fn names_and_summaries_are_flattened_to_one_line() {
        let mut game = sample_game();
        game.name = "two\nlines".to_string();
        game.summary = "a\rb".to_string();
        let back = SaveGame::from_text(&game.to_text()).unwrap();
        assert_eq!(back.name, "two lines");
        assert_eq!(back.summary, "a b");
    }

    #[test]
    fn unix_timestamps_format_as_utc() {
        assert_eq!(format_unix_utc(0), "1970-01-01 00:00 UTC");
        assert_eq!(format_unix_utc(1_767_225_600), "2026-01-01 00:00 UTC");
        assert_eq!(
            format_unix_utc(1_767_225_600 + 3_723),
            "2026-01-01 01:02 UTC"
        );
    }
}
