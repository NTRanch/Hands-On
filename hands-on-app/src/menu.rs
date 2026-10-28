//! menu — the menu system: the themed home screen, the
//! level picker, the settings screen, the save/load screen,
//! and the session console that hosts the session-layer
//! levels in this shell.
//!
//! Theming comes from found brand assets only: the Nice
//! Thoughts Ranch "splits" mark (the studio keeper logo,
//! embedded below), its palette (near-black indigo ground,
//! deep violet, teal rings, brass gold, the game's own pink),
//! and the game's established panel colors. There is no
//! Hands-On-specific logo asset in existence — the itch
//! materials leave the banner a conscious blank — so the
//! home screen flies the Ranch mark and says so.
//!
//! Persistence is the core's, shared with the TUI
//! (`hands_on::save`, `hands_on::settings`): three slots,
//! the campaign (Continue) file, and the settings file live
//! in the platform app-data store. This module is the app's
//! glue between those files and its screens — completion
//! recording, checkpoints, and slot operations all funnel
//! through here so every screen agrees on what is saved.
//!
//! U5 note: settings tune stimulation response only (gain +
//! threshold per host). Nothing in this module touches,
//! gates, or weakens STOP, aftercare, the limiter, or any
//! safety surface — those are not settings and never were.

use eframe::egui;
use hands_on::feedback::FeedbackEngine;
use hands_on::level::{LevelDescriptor, Track, registry};
use hands_on::play::playroom_session;
use hands_on::save::{LevelKey, LevelState, SaveGame, SimPayload};
use hands_on::session::{Session, reward_host};
use hands_on::settings::{GAIN_MAX, GAIN_MIN, THRESHOLD_MAX, THRESHOLD_MIN};
use hands_on::study::study_session;

use crate::audio::{AudioSettings, Cue};
use crate::{HandsOnApp, Screen};

/// The P1 level key (the only level whose host runs a live,
/// resumable sim in this shell).
pub(crate) const P1_KEY: LevelKey = LevelKey {
    track: Track::Playroom,
    id: 1,
};
/// The P4 level key.
pub(crate) const P4_KEY: LevelKey = LevelKey {
    track: Track::Playroom,
    id: 4,
};

/// Brass gold — the mark's belt lettering, the title voice.
pub(crate) const GOLD: egui::Color32 = egui::Color32::from_rgb(0xe8, 0xb3, 0x4b);
/// The game's pink — the P4 curve, the mark's blush family.
pub(crate) const PINK: egui::Color32 = egui::Color32::from_rgb(0xff, 0x5c, 0x8a);
/// The mark's teal ring.
pub(crate) const TEAL: egui::Color32 = egui::Color32::from_rgb(0x2f, 0xb9, 0xcc);
/// Success green (the same value the level screens use).
pub(crate) const GREEN: egui::Color32 = egui::Color32::from_rgb(0x7d, 0xe0, 0x8a);
/// Refusal red (the same value the level screens use).
pub(crate) const RED: egui::Color32 = egui::Color32::from_rgb(0xff, 0x3b, 0x30);

/// The Nice Thoughts Ranch splits mark (studio keeper),
/// downscaled for the menu. No Hands-On logo asset exists;
/// this is the brand the game ships under.
const NTR_LOGO_PNG: &[u8] = include_bytes!("../assets/ntr-splits.png");

/// Working state for the session console: one session-layer
/// sitting hosted in-app. The sitting itself is the core's
/// `Session` — the same object the TUI drives — and success
/// pays out through a feedback engine on the level's host,
/// exactly as the TUI's runner does (SPEC §6.3).
pub(crate) struct ConsoleState {
    /// The sitting in progress.
    pub session: Box<dyn Session>,
    /// The host's payout engine.
    pub engine: FeedbackEngine,
    /// The sitting's descriptor, cached for the header and
    /// the completion record.
    pub desc: LevelDescriptor,
    /// Transcript lines (intro, commands echoed, answers).
    pub transcript: Vec<String>,
    /// The command line being typed.
    pub input: String,
    /// The sitting has ended (success or failure).
    pub finished: bool,
}

impl HandsOnApp {
    // -- persistence glue -----------------------------------

    /// Current Unix time for save timestamps (the core is
    /// clock-free; shells supply this).
    pub(crate) fn now_unix() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// The save context's host name (registry lookup, with
    /// the context level's own host as the fallback).
    fn context_host(&self) -> String {
        registry()
            .into_iter()
            .find(|d| d.track == self.context_level.track && d.id == self.context_level.id)
            .map(|d| d.host.name.to_string())
            .unwrap_or_else(|| "Opal".to_string())
    }

    /// A progress summary for the current context, for slot
    /// metadata: live numbers in P1, a room note elsewhere.
    fn context_summary(&self) -> String {
        if self.context_level == P1_KEY {
            let s = self.engine.snapshot();
            format!(
                "bottle {:.1} · climaxes {} · {}",
                s.bottle,
                s.climax_count,
                if s.won { "session won" } else { "in progress" }
            )
        } else if let Some(cs) = &self.console {
            if cs.finished {
                "session complete".to_string()
            } else {
                "session in progress (console)".to_string()
            }
        } else if self.context_level == P4_KEY {
            if self.studio.gate.as_ref().is_some_and(|g| g.passed) {
                "loop studio gate passed".to_string()
            } else {
                "in the Loop Studio".to_string()
            }
        } else {
            "at the menu".to_string()
        }
    }

    /// Build the save document for the current state — the
    /// one shape slots and the campaign share. The summary
    /// is the completion note when one is recorded for this
    /// context, else a computed room note.
    fn current_save(&self, name: &str) -> SaveGame {
        let sim = if self.context_level == P1_KEY {
            Some(SimPayload::capture(self.engine.sim()))
        } else {
            None
        };
        let summary = if self.context_note.is_empty() {
            self.context_summary()
        } else {
            self.context_note.clone()
        };
        SaveGame {
            name: name.to_string(),
            saved_at_unix: Self::now_unix(),
            level: self.context_level,
            host: self.context_host(),
            summary,
            progression: self.progression.clone(),
            sim,
        }
    }

    /// Persist the campaign (Continue) file from the current
    /// state. Keeps the in-memory campaign in step.
    pub(crate) fn persist_campaign(&mut self) {
        let game = self.current_save("Campaign");
        self.campaign = Some(game.clone());
        if let Some(store) = &self.store
            && let Err(e) = store.save_campaign(&game)
        {
            self.store_notice = Some(format!("campaign save failed: {e}"));
        }
    }

    /// Record a level completion and persist the campaign:
    /// the one funnel every screen's success flows through
    /// (P1's bottle, P4's gate, a console sitting's success).
    pub(crate) fn complete_level(&mut self, key: LevelKey, summary: String) {
        self.progression.mark_completed(key);
        self.context_level = key;
        self.context_note = summary;
        self.persist_campaign();
    }

    /// Checkpoint without a completion — entering or leaving
    /// a room updates where Continue will land.
    pub(crate) fn checkpoint(&mut self, key: LevelKey) {
        if self.context_level != key {
            self.context_note.clear();
        }
        self.context_level = key;
        self.persist_campaign();
    }

    /// Persist the settings file after an edit.
    pub(crate) fn persist_settings(&mut self) {
        if let Some(store) = &self.store
            && let Err(e) = store.save_settings(&self.settings)
        {
            self.store_notice = Some(format!("settings save failed: {e}"));
        }
    }

    /// Refresh the slot metadata cache (Save/Load screen).
    pub(crate) fn refresh_slots(&mut self) {
        if let Some(store) = &self.store {
            match store.slot_listing() {
                Ok(listing) => self.slots = listing,
                Err(e) => {
                    self.slots = Vec::new();
                    self.save_status = format!("slot listing refused: {e}");
                }
            }
        }
    }

    /// Write the current state into a slot.
    pub(crate) fn save_to_slot(&mut self, slot: u8) {
        let Some(store) = &self.store else {
            self.save_status = "saves are off (no app-data directory)".to_string();
            return;
        };
        let game = self.current_save(&format!("Slot {slot}"));
        match store.save_slot(slot, &game) {
            Ok(()) => {
                self.save_status = format!("saved to slot {slot}: {}", game.describe());
                self.audio.play(Cue::Success);
            }
            Err(e) => {
                self.save_status = format!("save refused: {e}");
                self.audio.play(Cue::Failure);
            }
        }
        self.refresh_slots();
    }

    /// Load a slot as the live state and go where it points.
    pub(crate) fn load_from_slot(&mut self, slot: u8) {
        let Some(store) = &self.store else {
            self.save_status = "saves are off (no app-data directory)".to_string();
            return;
        };
        let game = match store.load_slot(slot) {
            Ok(Some(game)) => game,
            Ok(None) => {
                self.save_status = format!("slot {slot} is empty");
                return;
            }
            Err(e) => {
                // A corrupt slot is a typed refusal on the
                // status line — never a crash, never a
                // half-applied load.
                self.save_status = format!("load refused: {e}");
                self.audio.play(Cue::Failure);
                return;
            }
        };
        self.apply_save(game);
    }

    /// Apply a loaded save (slot or campaign): progression,
    /// context, and the P1 sim when the save carries one;
    /// then navigate to the level it names.
    fn apply_save(&mut self, game: SaveGame) {
        self.progression = game.progression.clone();
        self.context_level = game.level;
        if game.level == P1_KEY {
            if let Some(payload) = &game.sim {
                match payload.restore(self.cfg) {
                    Ok(sim) => {
                        *self.engine.sim_mut() = sim;
                        self.prev_climax = self.engine.snapshot().climax_count;
                        self.prev_won = self.engine.snapshot().won;
                    }
                    Err(e) => {
                        self.save_status =
                            format!("her saved session was refused ({e}); P1 starts fresh");
                    }
                }
            }
            self.pending_nav = Some(Screen::P1);
        } else if game.level == P4_KEY {
            self.pending_nav = Some(Screen::P4);
        } else if let Some(desc) = registry()
            .into_iter()
            .find(|d| d.track == game.level.track && d.id == game.level.id)
        {
            self.open_console(&desc);
        }
        self.campaign = Some(game);
        self.persist_campaign();
    }

    /// Continue: apply the campaign save, if one exists.
    pub(crate) fn continue_campaign(&mut self) {
        if let Some(game) = self.campaign.clone() {
            self.apply_save(game);
        }
    }

    /// Load the Ranch mark texture on first use.
    fn logo_id(&mut self, ctx: &egui::Context) -> Option<egui::TextureId> {
        if self.logo.is_none() {
            let decoded =
                image::load_from_memory_with_format(NTR_LOGO_PNG, image::ImageFormat::Png)
                    .ok()?
                    .to_rgb8();
            let (w, h) = decoded.dimensions();
            let pixels = egui::ColorImage::from_rgb([w as usize, h as usize], decoded.as_raw());
            self.logo = Some(ctx.load_texture("ntr-splits", pixels, egui::TextureOptions::LINEAR));
        }
        self.logo.as_ref().map(|t| t.id())
    }

    // -- navigation -----------------------------------------

    /// Enter a level from the picker: bespoke screens route
    /// to their screens; session-layer levels open in the
    /// session console (their real sitting, hosted in-app).
    pub(crate) fn enter_level(&mut self, desc: &LevelDescriptor) {
        match (desc.track, desc.id) {
            (Track::Playroom, 1) => {
                self.checkpoint(LevelKey::of(desc));
                self.pending_nav = Some(Screen::P1);
            }
            (Track::Playroom, 4) => {
                self.checkpoint(LevelKey::of(desc));
                self.pending_nav = Some(Screen::P4);
            }
            _ => self.open_console(desc),
        }
    }

    /// Open a session-layer level in the console: a fresh
    /// sitting on a settings-tuned payout engine.
    pub(crate) fn open_console(&mut self, desc: &LevelDescriptor) {
        let session = match desc.track {
            Track::Playroom => playroom_session(desc.id),
            Track::Study => study_session(desc.id),
        };
        let Some(session) = session else {
            self.save_status = format!("{} has no session to open", desc.title);
            return;
        };
        let mut engine = FeedbackEngine::new(desc.host, self.cfg);
        engine
            .sim_mut()
            .set_profile(self.settings.effective_profile(&desc.host));
        let intro = session.intro();
        self.console = Some(ConsoleState {
            session,
            engine,
            desc: *desc,
            transcript: vec![intro],
            input: String::new(),
            finished: false,
        });
        self.checkpoint(LevelKey::of(desc));
        self.pending_nav = Some(Screen::Console);
    }

    // -- screens --------------------------------------------

    /// The home screen: the Ranch mark, Continue first when
    /// a campaign exists, then Picker / Save / Load /
    /// Settings.
    pub(crate) fn menu_home_ui(&mut self, ui: &mut egui::Ui) {
        if let Some(id) = self.logo_id(ui.ctx()) {
            ui.vertical_centered(|ui| {
                ui.image((id, egui::vec2(232.0, 131.0)));
            });
        }
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("HANDS-ON")
                    .size(34.0)
                    .strong()
                    .color(GOLD),
            );
            ui.weak("Nice Thoughts Ranch · a teaching farm");
        });
        ui.add_space(6.0);
        ui.label(
            "Learn a working studio's game systems by playing them. \
             Success drives live feedback on each level's host — her \
             reactions run on the same deterministic tick as your inputs.",
        );
        if let Some(notice) = &self.store_notice {
            ui.colored_label(RED, notice.clone());
        }
        ui.add_space(10.0);
        let full = [ui.available_width(), 52.0];
        // Owned strings: the campaign borrow must be gone
        // before the click handler can mutate self.
        let campaign_line = self
            .campaign
            .as_ref()
            .map(|g| (format!("▶ Continue — {}", g.describe()), g.summary.clone()));
        if let Some((label, last)) = campaign_line {
            if ui.add_sized(full, egui::Button::new(label)).clicked() {
                self.continue_campaign();
            }
            ui.weak(format!("last time: {last}"));
            ui.add_space(4.0);
        }
        if ui
            .add_sized(full, egui::Button::new("🗺  Level Picker"))
            .clicked()
        {
            self.pending_nav = Some(Screen::Picker);
        }
        ui.horizontal(|ui| {
            let half = [(ui.available_width() - 8.0) / 2.0, 52.0];
            if ui.add_sized(half, egui::Button::new("💾  Save")).clicked() {
                self.refresh_slots();
                self.pending_nav = Some(Screen::SaveLoad);
            }
            if ui.add_sized(half, egui::Button::new("📂  Load")).clicked() {
                self.refresh_slots();
                self.pending_nav = Some(Screen::SaveLoad);
            }
        });
        if ui
            .add_sized(full, egui::Button::new("⚙  Settings"))
            .clicked()
        {
            self.pending_nav = Some(Screen::Settings);
        }
        ui.add_space(10.0);
        ui.weak(format!("{}/17 levels complete", self.progression.count()));
        match &self.store {
            Some(store) => ui.weak(format!("saves + settings: {}", store.root().display())),
            None => ui.weak("no app-data directory — saves and settings are off"),
        };
    }

    /// The level picker: every level, its host, its state
    /// from real progression, and its unlock condition when
    /// locked.
    pub(crate) fn picker_ui(&mut self, ui: &mut egui::Ui) {
        self.header(ui, "Level Picker");
        let mut enter: Option<LevelDescriptor> = None;
        for (track, title) in [
            (Track::Playroom, "The Playroom"),
            (Track::Study, "The Study — Rust mastery"),
        ] {
            ui.label(egui::RichText::new(title).size(20.0).strong().color(GOLD));
            let levels: Vec<LevelDescriptor> = registry()
                .into_iter()
                .filter(|d| d.track == track)
                .collect();
            for desc in levels {
                let state = self.progression.state_of(&desc);
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(format!("{} — {}", desc_key_label(&desc), desc.title));
                        match state {
                            LevelState::Completed => {
                                ui.colored_label(GREEN, "✓ complete");
                            }
                            LevelState::Available => {
                                ui.colored_label(TEAL, "▶ available");
                            }
                            LevelState::Locked => {
                                ui.weak("🔒 locked");
                            }
                        }
                    });
                    ui.label(format!("host: {} — {}", desc.host.name, desc.host.sketch));
                    ui.weak(desc.source_system);
                    match state {
                        LevelState::Locked => {
                            if let Some(pre) = self.progression.prerequisite(&desc) {
                                ui.weak(format!("unlock: complete {} first", pre.code()));
                            }
                        }
                        _ => {
                            let label = enter_label(&desc);
                            if ui
                                .add_sized([ui.available_width(), 44.0], egui::Button::new(label))
                                .clicked()
                            {
                                enter = Some(desc);
                            }
                        }
                    }
                });
            }
            ui.add_space(6.0);
        }
        if let Some(desc) = enter {
            self.enter_level(&desc);
        }
    }

    /// The settings screen: per-host stimulation response
    /// (the shared store the P1 dial also writes), plus the
    /// sound settings that used to live on the menu.
    pub(crate) fn settings_ui(&mut self, ui: &mut egui::Ui) {
        self.header(ui, "Settings");
        if let Some(notice) = &self.store_notice {
            ui.colored_label(RED, notice.clone());
        }
        ui.label(
            egui::RichText::new("How she responds")
                .size(20.0)
                .strong()
                .color(GOLD),
        );
        ui.weak(
            "How strongly each girl feels stimulation, and how much it takes \
             to tip her over. These are the same values as the live dial in \
             her room — one set of numbers, everywhere. Nothing here can \
             touch her STOP, her limits, or aftercare.",
        );
        ui.add_space(4.0);
        // Distinct hosts, in registry order.
        let mut hosts: Vec<(String, String)> = Vec::new();
        for desc in registry() {
            if !hosts.iter().any(|(n, _)| n == desc.host.name) {
                hosts.push((desc.host.name.to_string(), desc.host.sketch.to_string()));
            }
        }
        let live_host = self.engine.host().name.to_string();
        let mut persist = false;
        for (name, sketch) in hosts {
            ui.group(|ui| {
                ui.label(egui::RichText::new(name.clone()).strong().color(PINK));
                ui.weak(sketch);
                let mut tuning = self.settings.tuning_for(&name);
                let before = tuning;
                let gain_resp = ui.add(
                    egui::Slider::new(&mut tuning.gain, GAIN_MIN..=GAIN_MAX)
                        .text("sensitivity — how much each touch counts")
                        .fixed_decimals(2),
                );
                let thr_resp = ui.add(
                    egui::Slider::new(&mut tuning.threshold, THRESHOLD_MIN..=THRESHOLD_MAX)
                        .text("her edge — fullness that tips her over")
                        .fixed_decimals(0),
                );
                if tuning != before {
                    self.settings.set_tuning(&name, tuning);
                    if name == live_host {
                        let host = self.engine.host();
                        let profile = self.settings.effective_profile(&host);
                        self.engine.sim_mut().set_profile(profile);
                    }
                }
                if gain_resp.drag_stopped()
                    || thr_resp.drag_stopped()
                    || gain_resp.lost_focus()
                    || thr_resp.lost_focus()
                {
                    persist = true;
                }
                ui.horizontal(|ui| {
                    if ui.button("Reset her to her tuned defaults").clicked() {
                        self.settings.reset_host(&name);
                        if name == live_host {
                            let host = self.engine.host();
                            let profile = self.settings.effective_profile(&host);
                            self.engine.sim_mut().set_profile(profile);
                        }
                        persist = true;
                    }
                    let t = self.settings.tuning_for(&name);
                    ui.weak(format!("gain {:.2} · threshold {:.0}", t.gain, t.threshold));
                });
            });
        }
        if ui
            .button("Reset all girls to their tuned defaults")
            .clicked()
        {
            self.settings.reset_all_tunings();
            let host = self.engine.host();
            let profile = self.settings.effective_profile(&host);
            self.engine.sim_mut().set_profile(profile);
            persist = true;
        }
        if persist {
            self.persist_settings();
        }
        ui.add_space(10.0);
        ui.label(egui::RichText::new("Sound").size(20.0).strong().color(GOLD));
        ui.weak("Presentation only — sound owns no game state and gates nothing.");
        ui.group(|ui| {
            ui.horizontal(|ui| {
                let mut changed = false;
                let mut enabled = self.settings.sound_enabled;
                if ui.checkbox(&mut enabled, "Sound").changed() {
                    self.settings.sound_enabled = enabled;
                    changed = true;
                }
                let mut volume = f64::from(self.settings.sound_volume);
                let resp = ui.add(
                    egui::Slider::new(&mut volume, 0.0..=1.0)
                        .text("volume")
                        .fixed_decimals(2),
                );
                if resp.changed() {
                    self.settings.sound_volume = volume as f32;
                    changed = true;
                }
                if resp.drag_stopped() || resp.lost_focus() {
                    self.persist_settings();
                }
                if changed {
                    self.audio.settings = AudioSettings {
                        enabled: self.settings.sound_enabled,
                        volume: self.settings.sound_volume,
                    };
                    self.audio.apply_settings();
                }
                ui.weak(if self.audio.backend_live() {
                    "output: live"
                } else {
                    "output: silent (no device)"
                });
            });
        });
    }

    /// The save/load screen: three slots with their
    /// metadata; save writes the current state, load
    /// restores a slot and goes where it points.
    pub(crate) fn saveload_ui(&mut self, ui: &mut egui::Ui) {
        self.header(ui, "Save / Load");
        if !self.save_status.is_empty() {
            ui.label(self.save_status.clone());
            ui.add_space(4.0);
        }
        let mut action: Option<(bool, u8)> = None;
        for slot in 1..=hands_on::save::SLOT_COUNT {
            let game = self.slots.get((slot - 1) as usize).cloned().flatten();
            ui.group(|ui| {
                ui.strong(format!("Slot {slot}"));
                match &game {
                    Some(game) => {
                        ui.label(game.describe());
                        ui.weak(format!("\"{}\" · {}", game.name, game.summary));
                    }
                    None => {
                        ui.weak("— empty —");
                    }
                }
                ui.horizontal(|ui| {
                    let half = [(ui.available_width() - 8.0) / 2.0, 44.0];
                    ui.add_enabled_ui(game.is_some(), |ui| {
                        if ui
                            .add_sized(half, egui::Button::new("📂 Load"))
                            .on_hover_text("restore this slot and go where it points")
                            .clicked()
                        {
                            action = Some((false, slot));
                        }
                    });
                    if ui
                        .add_sized(half, egui::Button::new("💾 Save here"))
                        .clicked()
                    {
                        action = Some((true, slot));
                    }
                });
            });
        }
        ui.add_space(6.0);
        ui.weak(
            "Saving records where you are, your progression, and — in P1 — \
             her exact session. A corrupt slot refuses with a message; it \
             never half-loads.",
        );
        if let Some((is_save, slot)) = action {
            if is_save {
                self.save_to_slot(slot);
            } else {
                self.load_from_slot(slot);
            }
        }
    }

    /// The session console: a session-layer sitting hosted
    /// in-app — the same `Session` the TUI drives, with the
    /// same payout on success.
    pub(crate) fn console_ui(&mut self, ui: &mut egui::Ui) {
        self.header(ui, "Session Console");
        let Some(mut cs) = self.console.take() else {
            ui.label("No session is open. Pick one in the Level Picker.");
            return;
        };
        let key = LevelKey::of(&cs.desc);
        ui.strong(format!("{} — {}", desc_key_label(&cs.desc), cs.desc.title));
        ui.label(format!(
            "host: {} — {}",
            cs.desc.host.name, cs.desc.host.sketch
        ));
        ui.weak(
            "This level plays as a session — the same sitting the TUI hosts, \
             here in the app. Its commands are in the introduction below; \
             an unknown command answers with help.",
        );
        ui.add_space(4.0);
        ui.group(|ui| {
            for line in cs.transcript.iter().rev().take(14).rev() {
                ui.label(line.clone());
            }
        });
        let mut submit = false;
        if !cs.finished {
            ui.horizontal(|ui| {
                let resp = ui.add_sized(
                    [ui.available_width() - 96.0, 40.0],
                    egui::TextEdit::singleline(&mut cs.input).hint_text("her command…"),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    submit = true;
                }
                if ui
                    .add_sized([88.0, 40.0], egui::Button::new("Say it"))
                    .clicked()
                {
                    submit = true;
                }
            });
        }
        if submit && !cs.finished {
            let cmd = cs.input.trim().to_string();
            cs.input.clear();
            if !cmd.is_empty() {
                cs.transcript.push(format!("> {cmd}"));
                let step = cs.session.handle(&cmd);
                if !step.text.is_empty() {
                    cs.transcript.push(step.text);
                }
                if step.done {
                    cs.finished = true;
                    if step.success {
                        let gained = reward_host(&mut cs.engine, &self.cfg);
                        cs.transcript.push(format!(
                            "— payout: {} responds through the feedback engine (climaxes +{gained}) —",
                            cs.desc.host.name
                        ));
                        self.audio.play(Cue::Climax);
                        self.complete_level(key, format!("session complete · payout +{gained}"));
                    } else {
                        self.audio.play(Cue::Failure);
                    }
                }
            }
        }
        if cs.finished {
            ui.colored_label(
                GREEN,
                if self.progression.is_completed(key) {
                    "✓ complete — recorded in your progression"
                } else {
                    "the sitting has ended"
                },
            );
            if ui.button("Back to the Level Picker").clicked() {
                self.pending_nav = Some(Screen::Picker);
            }
        }
        self.console = Some(cs);
    }
}

/// The picker's per-level button text: bespoke screens name
/// their screens; session levels name the console honestly.
fn enter_label(desc: &LevelDescriptor) -> String {
    match (desc.track, desc.id) {
        (Track::Playroom, 1) => "Play P1 — First Milking".to_string(),
        (Track::Playroom, 4) => "Open P4 — Loop Studio".to_string(),
        _ => format!(
            "Enter {} — session console (its TUI sitting, in-app)",
            desc_key_label(desc)
        ),
    }
}

/// `P1` / `S3` style label for a descriptor.
fn desc_key_label(desc: &LevelDescriptor) -> String {
    LevelKey::of(desc).code()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HandsOnApp;

    #[test]
    fn entering_a_session_level_opens_the_console() {
        let mut app = HandsOnApp::new();
        let p2 = registry()
            .into_iter()
            .find(|d| d.track == Track::Playroom && d.id == 2)
            .expect("P2 registered");
        // P2 is locked on a fresh progression: entering is
        // the picker's business to gate; the console opens
        // the real sitting when it is reached.
        app.enter_level(&p2);
        let cs = app.console.as_ref().expect("console session opens");
        assert_eq!(cs.desc.id, 2);
        assert!(!cs.transcript.is_empty(), "the intro is the first line");
        assert_eq!(app.pending_nav, Some(Screen::Console));
    }

    #[test]
    fn settings_tunings_cover_every_host() {
        let app = HandsOnApp::new();
        for name in ["Opal", "Sable", "Vega", "Mireille", "Vigil"] {
            let t = app.settings.tuning_for(name);
            assert!(t.gain.is_finite() && t.threshold.is_finite(), "{name}");
        }
    }
}
