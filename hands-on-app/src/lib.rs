//! hands-on-app — the touch-first egui front-end for Hands-On.
//!
//! One crate, two shells: a desktop window (for verification)
//! and an Android `NativeActivity` (the same binary logic, the
//! same screens). All game logic stays where it already lives —
//! the deterministic fixed-step core in `milking-core`, the
//! level registry and feedback engine in `hands-on` — this
//! crate only renders and forwards input, folding wall-clock
//! time into the core's fixed ticks exactly the way the TUI
//! does (SPEC §4: rendering never feeds back into the sim).
//!
//! Playable here: **P1 First Milking** (the squeeze-cycle loop
//! on Opal, with the live sensitivity dial) and the **P4 Loop
//! Studio slice** (keyform loop authoring, closure +
//! twin-run-hash gate). Every level is built — the other
//! fifteen run on the session layer, hosted in this shell by
//! the session console (the `menu` module) and in the TUI by
//! its runner. The menu system — themed home, level picker,
//! save/load slots, settings — lives in `menu`; persistence
//! itself lives in the core (`hands_on::save` /
//! `hands_on::settings`), shared with the TUI.
//!
//! `deny`, not `forbid`, in this crate only: the Android
//! entry contract requires `#[unsafe(no_mangle)]` on
//! `android_main` (edition 2024 makes no_mangle an unsafe
//! attribute, and the `unsafe_code` lint covers it), and a
//! `forbid` cannot be overridden at item level (E0453).
//! The single sanctioned exception is `android_main` below,
//! with its own item-level allow; everything else in the
//! crate is denied unsafe by default. `milking-core` and
//! `hands-on` keep `forbid` — they have no platform entry
//! contract.
#![deny(unsafe_code)]

pub mod anim;
pub mod audio;
pub mod menu;

use std::time::Instant;

use eframe::egui;
use milking_core::affordance::Action;
use milking_core::authoring::{Easing, Keyform, Track};
use milking_core::expression::Expression;
use milking_core::sim::{Config, Hand, InputEvent, Outcome, SensitivityProfile};

use hands_on::feedback::FeedbackEngine;
use hands_on::level::{Level, MilkingLevel};
use hands_on::save::{LevelKey, Progression, SaveGame, Store};
use hands_on::settings::GameSettings;

use anim::{Burst, Glide, Pulse, Transition};
use audio::{AudioEngine, Bed, Cue};

/// Which screen is showing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Screen {
    /// The themed home screen (menu system).
    Menu,
    /// The level picker.
    Picker,
    /// The save/load slots.
    SaveLoad,
    /// The settings screen.
    Settings,
    /// P1 — First Milking.
    P1,
    /// P4 — Loop Studio slice.
    P4,
    /// A session-layer sitting in the console.
    Console,
}

/// Opal's expression portraits (the Opal commission,
/// Stages A–B): one House Cel portrait per sim expression,
/// in `Expression::ALL` order — the discriminants are the
/// progression order, so `expression as usize` indexes this
/// table directly. Panel crops (2:3) of the canonical set.
const OPAL_EXPR_JPEG: [&[u8]; 5] = [
    include_bytes!("../assets/opal/expr_neutral.jpg"),
    include_bytes!("../assets/opal/expr_smiling.jpg"),
    include_bytes!("../assets/opal/expr_flushed.jpg"),
    include_bytes!("../assets/opal/expr_aroused.jpg"),
    include_bytes!("../assets/opal/expr_ecstatic.jpg"),
];

/// Texture names for the expression portraits, same order.
const OPAL_EXPR_NAMES: [&str; 5] = [
    "opal-neutral",
    "opal-smiling",
    "opal-flushed",
    "opal-aroused",
    "opal-ecstatic",
];

/// Opal's fence-lean idle (Stage C): 12 frames, played at
/// 200 ms per frame while her expression is Neutral.
const OPAL_IDLE_JPEG: [&[u8]; 12] = [
    include_bytes!("../assets/opal/idle_01.jpg"),
    include_bytes!("../assets/opal/idle_02.jpg"),
    include_bytes!("../assets/opal/idle_03.jpg"),
    include_bytes!("../assets/opal/idle_04.jpg"),
    include_bytes!("../assets/opal/idle_05.jpg"),
    include_bytes!("../assets/opal/idle_06.jpg"),
    include_bytes!("../assets/opal/idle_07.jpg"),
    include_bytes!("../assets/opal/idle_08.jpg"),
    include_bytes!("../assets/opal/idle_09.jpg"),
    include_bytes!("../assets/opal/idle_10.jpg"),
    include_bytes!("../assets/opal/idle_11.jpg"),
    include_bytes!("../assets/opal/idle_12.jpg"),
];

/// Texture names for the idle frames, same order.
const OPAL_IDLE_NAMES: [&str; 12] = [
    "opal-idle-01",
    "opal-idle-02",
    "opal-idle-03",
    "opal-idle-04",
    "opal-idle-05",
    "opal-idle-06",
    "opal-idle-07",
    "opal-idle-08",
    "opal-idle-09",
    "opal-idle-10",
    "opal-idle-11",
    "opal-idle-12",
];

/// Decode one embedded Opal JPEG into an egui texture,
/// cached in `slot`. Returns `None` on decode failure — the
/// caller falls back to the drawn presence disc, so a bad
/// asset degrades the panel instead of breaking the screen.
fn opal_texture(
    slot: &mut Option<egui::TextureHandle>,
    ctx: &egui::Context,
    name: &str,
    bytes: &[u8],
) -> Option<egui::TextureId> {
    if slot.is_none() {
        let decoded = image::load_from_memory_with_format(bytes, image::ImageFormat::Jpeg)
            .ok()?
            .to_rgb8();
        let (w, h) = decoded.dimensions();
        let pixels = egui::ColorImage::from_rgb([w as usize, h as usize], decoded.as_raw());
        *slot = Some(ctx.load_texture(name, pixels, egui::TextureOptions::LINEAR));
    }
    slot.as_ref().map(|t| t.id())
}

/// The House Cel style line per expression (SPEC §7.3), the
/// same grammar the TUI's feedback panel names in text.
fn style_line(expression: Expression) -> &'static str {
    match expression {
        Expression::Neutral => {
            "House Cel — heart pupils (the eye rule), gaze level, blush not yet risen"
        }
        Expression::Smiling => {
            "House Cel — heart pupils, first catchlight sparkles, blush rising at the edges"
        }
        Expression::Flushed => {
            "House Cel — heart pupils glowing under heavy lids, blush rising deep, sparkles multiplying"
        }
        Expression::Aroused => {
            "House Cel — heart pupils bright, gaze turning up, blush full, sparkles dancing"
        }
        Expression::Ecstatic => {
            "House Cel — TOROGAO: half-lidded upturned eyes, luminous heart pupils, open mouth, heavy blush, sparkles everywhere"
        }
    }
}

/// FNV-1a 64-bit — the same tiny deterministic hash the TUI's
/// Loop Studio gate compares twin bakes with.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// One editable keyform row in the Loop Studio.
#[derive(Clone, Copy, Debug, PartialEq)]
struct KeyformDraft {
    /// Milliseconds inside the cycle.
    ms: i32,
    /// Parameter value, 0..=1.
    value: f32,
}

/// Loop Studio working state.
struct StudioState {
    /// Cycle length, ms.
    cycle_ms: i32,
    /// Segment easing family.
    easing: Easing,
    /// Editable keyform rows.
    keyforms: Vec<KeyformDraft>,
    /// Last gate evaluation, if the player ran it.
    gate: Option<StudioGate>,
}

/// The outcome of one Loop Studio gate run.
struct StudioGate {
    /// Loop closure held.
    closure_ok: bool,
    /// Closure detail (error text when open).
    closure_note: String,
    /// Tempo score vs the 1000 ms ideal cycle.
    tempo_score: f64,
    /// Twin-run bake hashes.
    hash_a: u64,
    /// Twin-run bake hashes.
    hash_b: u64,
    /// Gate verdict: closure AND byte-identical twin bakes.
    passed: bool,
    /// Baked frame count at the 10 ms step.
    frames: usize,
}

impl StudioState {
    fn starter() -> Self {
        // The TUI demo's starter track: a closed 0 → 1 → 0 loop.
        Self {
            cycle_ms: 1000,
            easing: Easing::Smooth,
            keyforms: vec![
                KeyformDraft { ms: 0, value: 0.0 },
                KeyformDraft {
                    ms: 400,
                    value: 1.0,
                },
                KeyformDraft {
                    ms: 1000,
                    value: 0.0,
                },
            ],
            gate: None,
        }
    }

    /// Build the core track from the drafts, if they are valid.
    fn track(&self) -> Result<Track, String> {
        let keyforms: Vec<Keyform> = self
            .keyforms
            .iter()
            .map(|k| Keyform {
                at_ms: k.ms.max(0) as u32,
                value: f64::from(k.value),
            })
            .collect();
        Track::new(self.cycle_ms.max(0) as u32, keyforms, self.easing)
            .map_err(|e| format!("track rejected: {e:?}"))
    }

    /// The P4 pass gate, mechanically (SPEC §6.5): closure AND
    /// a twin-run bake that comes out byte-identical.
    fn run_gate(&mut self) {
        self.gate = None;
        let track = match self.track() {
            Ok(t) => t,
            Err(_) => return,
        };
        let (closure_ok, closure_note) = match track.check_closure() {
            Ok(()) => (true, "end value == start value".to_string()),
            Err(e) => (false, format!("{e:?}")),
        };
        let bake_bytes = |t: &Track| -> Option<Vec<u8>> {
            t.bake(10)
                .ok()
                .map(|table| table.iter().flat_map(|v| v.to_le_bytes()).collect())
        };
        if let (Some(a), Some(b)) = (bake_bytes(&track), bake_bytes(&track)) {
            let frames = track.bake(10).map(|t| t.len()).unwrap_or(0);
            self.gate = Some(StudioGate {
                closure_ok,
                closure_note,
                tempo_score: track.tempo_score(1000),
                hash_a: fnv1a(&a),
                hash_b: fnv1a(&b),
                passed: closure_ok && a == b,
                frames,
            });
        }
    }
}

/// The application.
pub struct HandsOnApp {
    screen: Screen,
    cfg: Config,
    /// P1 session: the shared feedback engine on Opal.
    engine: FeedbackEngine,
    /// Hard-squeeze toggle (pressure 950 vs the normal 700).
    hard: bool,
    /// Loop Studio state.
    studio: StudioState,
    /// Wall-clock anchor for folding real time into fixed ticks.
    last_frame: Instant,
    /// App start: the Loop Studio playhead loops against this.
    started: Instant,
    /// First-frame style pass done.
    styled: bool,
    /// Sound: settings + optional live backend (audio module).
    audio: AudioEngine,
    /// Squeeze pulse on the host portrait (anim module).
    pulse: Option<Pulse>,
    /// When the last spill's squash started, if any.
    squash_start: Option<Instant>,
    /// Payout sparkle burst (anim module).
    burst: Burst,
    /// Opal's expression portraits, uploaded as GPU textures
    /// on first use (indexed by `Expression` discriminant).
    opal_expr: [Option<egui::TextureHandle>; 5],
    /// Opal's idle frames, uploaded as GPU textures on first
    /// use (12 frames, fence-lean loop).
    opal_idle: [Option<egui::TextureHandle>; 12],
    /// Screen-change transition (anim module).
    transition: Option<Transition>,
    /// Climax count at the last frame (payout delta detector).
    prev_climax: u32,
    /// Bottle-full already celebrated.
    prev_won: bool,
    /// Smoothed meter/bar values (anim::Glide).
    meter_glide: Glide,
    /// Smoothed meter/bar values (anim::Glide).
    bottle_glide: Glide,
    /// Seconds since the previous frame (glide stepping).
    frame_dt: f32,
    /// Navigation requested from inside a ui closure; applied
    /// by `update` once the closure's borrows are released.
    pending_nav: Option<Screen>,
    /// The persistence store (menu system), when a data
    /// directory exists on this platform.
    store: Option<Store>,
    /// The shared settings: per-host stimulation tuning +
    /// sound (the core store both shells read and write).
    settings: GameSettings,
    /// Player progression — picker states derive from it.
    progression: Progression,
    /// The campaign save (Continue state), as last loaded or
    /// written.
    campaign: Option<SaveGame>,
    /// Save context: the level the player is in or last
    /// entered — where Continue lands and what slots record.
    context_level: LevelKey,
    /// The context's completion note (a save's summary when
    /// one was recorded; empty → a computed room note).
    context_note: String,
    /// Slot metadata cache for the Save/Load screen.
    slots: Vec<Option<SaveGame>>,
    /// Save/Load screen feedback line.
    save_status: String,
    /// A store refusal to surface on the menu screens
    /// (corrupt file, failed write) — typed, never a crash.
    store_notice: Option<String>,
    /// The NT Ranch mark, uploaded on first menu frame.
    logo: Option<egui::TextureHandle>,
    /// The session console's sitting, when one is open.
    console: Option<menu::ConsoleState>,
    /// Screenshot plan from `HANDS_ON_SHOT_DIR` (desktop only).
    shot_dir: Option<std::path::PathBuf>,
    /// Frames rendered so far (drives the screenshot plan).
    frame_no: u64,
    /// Screenshot stage: 0 = menu, 1 = picker, 2 = settings,
    /// 3 = P1, 4 = done.
    shot_stage: u8,
    /// Frame number the current screenshot stage began at.
    shot_base: u64,
    /// A screenshot request is in flight for the current
    /// stage (guards against stale captures from the previous
    /// screen being saved under the next stage's name).
    shot_requested: bool,
}

impl Default for HandsOnApp {
    fn default() -> Self {
        Self::new()
    }
}

impl HandsOnApp {
    /// A fresh app on the menu, with a P1 session ready.
    /// Settings, progression, and the campaign (Continue)
    /// state load from the shared store; a corrupt file is a
    /// notice on the menu, never a failed launch.
    #[must_use]
    pub fn new() -> Self {
        let cfg = Config::default();
        let level = MilkingLevel::new();
        let host = level.descriptor().host;
        let store = Store::default_location();
        let mut settings = GameSettings::defaults();
        let mut progression = Progression::new();
        let mut campaign: Option<SaveGame> = None;
        let mut store_notice: Option<String> = None;
        if let Some(store) = &store {
            match store.load_settings() {
                Ok(loaded) => settings = loaded,
                Err(e) => {
                    store_notice = Some(format!(
                        "settings file refused ({e}) — playing on the tuned defaults"
                    ));
                }
            }
            match store.load_campaign() {
                Ok(loaded) => {
                    if let Some(game) = &loaded {
                        progression = game.progression.clone();
                    }
                    campaign = loaded;
                }
                Err(e) => {
                    store_notice = Some(format!(
                        "campaign file refused ({e}) — starting a fresh campaign"
                    ));
                }
            }
        }
        let context_level = campaign.as_ref().map_or(menu::P1_KEY, |g| g.level);
        let mut engine = FeedbackEngine::new(host, cfg);
        engine
            .sim_mut()
            .set_profile(settings.effective_profile(&host));
        let audio = AudioEngine::new(audio::AudioSettings {
            enabled: settings.sound_enabled,
            volume: settings.sound_volume,
        });
        let mut app = Self {
            screen: Screen::Menu,
            cfg,
            engine,
            hard: false,
            studio: StudioState::starter(),
            last_frame: Instant::now(),
            started: Instant::now(),
            styled: false,
            audio,
            pulse: None,
            squash_start: None,
            burst: Burst::new(),
            opal_expr: std::array::from_fn(|_| None),
            opal_idle: std::array::from_fn(|_| None),
            transition: None,
            prev_climax: 0,
            prev_won: false,
            meter_glide: Glide::new(0.0),
            bottle_glide: Glide::new(0.0),
            frame_dt: 0.0,
            pending_nav: None,
            store,
            settings,
            progression,
            campaign,
            context_level,
            context_note: String::new(),
            slots: Vec::new(),
            save_status: String::new(),
            store_notice,
            logo: None,
            console: None,
            shot_dir: std::env::var_os("HANDS_ON_SHOT_DIR").map(std::path::PathBuf::from),
            frame_no: 0,
            shot_stage: 0,
            shot_base: 0,
            shot_requested: false,
        };
        app.refresh_slots();
        app
    }

    /// Fold elapsed wall-clock time into fixed ticks, capped —
    /// the TUI's rule, so tempo is graded by the core and a
    /// stalled frame cannot teleport the session.
    fn advance_clock(&mut self) {
        let elapsed_ms = self.last_frame.elapsed().as_millis() as u64;
        self.frame_dt = self.last_frame.elapsed().as_secs_f32().min(0.1);
        self.last_frame = Instant::now();
        let elapsed_ticks = (elapsed_ms * u64::from(self.cfg.tick_hz) / 1000).min(60 * 30);
        let now = self.engine.snapshot().tick;
        self.engine.sim_mut().advance_to(now + elapsed_ticks);
        // Payout deltas the clock (not a click) may have caused:
        // a risen climax count earns its burst whenever it lands.
        let s = self.engine.snapshot();
        if s.climax_count > self.prev_climax {
            self.burst.spawn(26);
        }
        self.prev_climax = s.climax_count;
        if s.won && !self.prev_won {
            self.prev_won = true;
            self.burst.spawn(40);
            self.audio.play(Cue::Success);
            // The win is a completion: progression + the
            // campaign record it the moment it happens.
            self.complete_level(
                menu::P1_KEY,
                format!("bottle full · climaxes {}", s.climax_count),
            );
        }
    }

    /// Navigate: set the screen, start the transition veil,
    /// switch the room's ambient bed, and click.
    fn go(&mut self, screen: Screen) {
        if screen == self.screen {
            return;
        }
        // Leaving P1 checkpoints her session into the
        // campaign — Continue resumes this exact sim.
        if self.screen == Screen::P1 {
            self.checkpoint(menu::P1_KEY);
        }
        // Entering P1 re-derives her profile from the shared
        // settings (the Settings screen may have tuned her).
        if screen == Screen::P1 {
            let host = self.engine.host();
            let profile = self.settings.effective_profile(&host);
            self.engine.sim_mut().set_profile(profile);
        }
        self.audio.play(if screen == Screen::Menu {
            Cue::UiBack
        } else {
            Cue::UiClick
        });
        self.screen = screen;
        self.transition = Some(Transition::now());
        self.audio.set_bed(match screen {
            Screen::Menu | Screen::Picker | Screen::SaveLoad | Screen::Settings => Bed::Menu,
            Screen::P1 | Screen::Console => Bed::Playroom,
            Screen::P4 => Bed::Studio,
        });
    }

    fn feed_squeeze(&mut self, hand: Hand) {
        let pressure = if self.hard { 950 } else { 700 };
        let tick = self.engine.snapshot().tick;
        let res = self.engine.feed(InputEvent::Squeeze {
            tick,
            hand,
            pressure,
        });
        self.pulse = Some(Pulse::now());
        match res {
            Ok(outcome) => {
                let cue = audio::cue_for_outcome(&outcome, self.hard);
                self.audio.play(cue);
                if matches!(outcome, Outcome::Spill) {
                    self.squash_start = Some(Instant::now());
                }
                if matches!(outcome, Outcome::Climax { .. }) {
                    self.burst.spawn(26);
                }
            }
            Err(_) => self.audio.play(Cue::BeatMiss),
        }
    }

    fn feed_side(&mut self, action: Action) {
        let tick = self.engine.snapshot().tick;
        let res = self.engine.feed(InputEvent::Side { tick, action });
        self.pulse = Some(Pulse::now());
        match res {
            Ok(outcome) => {
                let cue = audio::cue_for_outcome(&outcome, self.hard);
                self.audio.play(cue);
            }
            Err(_) => self.audio.play(Cue::BeatMiss),
        }
    }

    fn header(&mut self, ui: &mut egui::Ui, title: &str) {
        // `go` needs &mut self after the button closure releases
        // the ui borrow: record the click, navigate after.
        let mut back = false;
        ui.horizontal(|ui| {
            if self.screen != Screen::Menu && ui.button("← Menu").clicked() {
                back = true;
            }
            ui.heading(title);
        });
        if back {
            self.go(Screen::Menu);
        }
        ui.separator();
    }

    fn p1_ui(&mut self, ui: &mut egui::Ui) {
        self.header(ui, "P1 — First Milking");
        let s = self.engine.snapshot();
        let host = self.engine.host();
        ui.label(format!("{} — {}", host.name, host.sketch));
        ui.label(format!("expression: {:?} · tick {}", s.expression, s.tick));
        ui.weak(style_line(s.expression));
        ui.add_space(4.0);
        // Opal herself (the Opal commission): the sim's
        // expression state selects her portrait; Neutral
        // plays the 12-frame fence-lean idle at 200 ms per
        // frame. The feedback motion that used to animate
        // the placeholder disc — pulse on every touch
        // (anticipation + overshoot), squash on a spill,
        // sparkle bursts on payout — now plays ON her: the
        // portrait scales about its fence-line anchor and
        // the burst orbits the panel. The drawn disc remains
        // only as the decode-failure fallback.
        {
            let width = ui.available_width().min(360.0);
            let (rect, _resp) =
                ui.allocate_exact_size(egui::vec2(width, width * 1.5), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            let pulse_scale = self.pulse.map_or(1.0, |p| p.scale());
            let (sq_w, sq_h) = self.squash_start.map_or((1.0, 1.0), |t| {
                anim::squash(t.elapsed(), std::time::Duration::from_millis(300), 0.16)
            });
            let tex = if s.expression == Expression::Neutral {
                let frame = (ui.input(|i| i.time) / 0.2) as usize % OPAL_IDLE_JPEG.len();
                opal_texture(
                    &mut self.opal_idle[frame],
                    ui.ctx(),
                    OPAL_IDLE_NAMES[frame],
                    OPAL_IDLE_JPEG[frame],
                )
            } else {
                let idx = s.expression as usize;
                opal_texture(
                    &mut self.opal_expr[idx],
                    ui.ctx(),
                    OPAL_EXPR_NAMES[idx],
                    OPAL_EXPR_JPEG[idx],
                )
            };
            // Scale about the bottom-center: her fence line
            // stays planted while the portrait breathes.
            let anchor = egui::pos2(rect.center().x, rect.bottom());
            let half = egui::vec2(
                rect.width() * 0.5 * pulse_scale * sq_w,
                rect.height() * 0.5 * pulse_scale * sq_h,
            );
            let img_rect = egui::Rect::from_min_max(
                egui::pos2(anchor.x - half.x, anchor.y - 2.0 * half.y),
                egui::pos2(anchor.x + half.x, anchor.y),
            );
            let center = img_rect.center();
            if let Some(id) = tex {
                painter.image(
                    id,
                    img_rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            } else {
                // Fallback: the original drawn presence disc —
                // pulse, squash, heat tint, glint dots.
                let base_r = 34.0f32;
                let heat = (s.meter / 100.0).clamp(0.0, 1.0) as f32;
                let lerp = |a: f32, b: f32| (a + (b - a) * heat).clamp(0.0, 255.0) as u8;
                let disc = egui::Color32::from_rgb(
                    lerp(92.0, 255.0),
                    lerp(74.0, 122.0),
                    lerp(122.0, 154.0),
                );
                painter.circle_filled(
                    center,
                    base_r * pulse_scale,
                    egui::Color32::from_rgba_unmultiplied(0x19, 0x14, 0x24, 200),
                );
                painter.add(egui::Shape::ellipse_filled(
                    center,
                    egui::vec2(base_r * pulse_scale * sq_w, base_r * pulse_scale * sq_h),
                    disc,
                ));
                // Heart-pupil glint (the eye rule, as a drawn
                // beat): two catchlight dots that brighten
                // with expression.
                let glint = 90 + (140.0 * heat) as u8;
                for dx in [-9.0f32, 9.0] {
                    painter.circle_filled(
                        center + egui::vec2(dx * pulse_scale, -4.0),
                        3.2,
                        egui::Color32::from_rgba_unmultiplied(0xff, 0xd1, 0x66, glint),
                    );
                }
            }
            // Payout sparkles orbit her panel.
            for (off, r, alpha) in self.burst.draw_list() {
                painter.circle_filled(
                    center + off,
                    r,
                    egui::Color32::from_rgba_unmultiplied(0xff, 0xe9, 0x8a, (alpha * 230.0) as u8),
                );
            }
        }
        // Bars glide toward the sim's numbers (frame-rate
        // independent); the text always shows the true values.
        let bottle_t = (s.bottle / self.cfg.bottle_capacity).clamp(0.0, 1.0) as f32;
        let bottle_v = self.bottle_glide.step(bottle_t, self.frame_dt, 7.0);
        ui.add(egui::ProgressBar::new(bottle_v).text(format!(
            "bottle {:.0}/{}",
            s.bottle, self.cfg.bottle_capacity
        )));
        let meter_v = self
            .meter_glide
            .step((s.meter / 100.0) as f32, self.frame_dt, 7.0);
        ui.add(egui::ProgressBar::new(meter_v).text(format!("arousal {:.1}/100", s.meter)));
        ui.add(
            egui::ProgressBar::new(s.fullness as f32)
                .text(format!("reservoir {:.0} stored", s.reservoir)),
        );
        ui.label(format!(
            "combo {} · spills {} · climaxes {} · bucket {:.0}",
            s.combo, s.spills, s.climax_count, s.total_produced
        ));
        if s.refractory_remaining > 0 {
            ui.label(format!(
                "refractory: {} ticks (sensitivity halved)",
                s.refractory_remaining
            ));
        }
        ui.label(format!(
            "haptics> FAKE router would emit scalar {:.3} (ceiling 0.20) — no device attached",
            self.engine.fake_haptic_scalar()
        ));
        if s.won {
            ui.colored_label(
                egui::Color32::from_rgb(0x7d, 0xe0, 0x8a),
                "*** BOTTLE FULL — level complete ***",
            );
        }
        ui.add_space(6.0);

        // Action pad: big touch targets. Locked actions stay
        // visible (SPEC §2.4) and refuse through the engine,
        // which narrates the refusal in the log.
        let full = [ui.available_width(), 56.0];
        let half = [(ui.available_width() - 8.0) / 2.0, 56.0];
        ui.horizontal(|ui| {
            if ui
                .add_sized(half, egui::Button::new("🤚 Milk — Left"))
                .clicked()
            {
                self.feed_squeeze(Hand::Left);
            }
            if ui
                .add_sized(half, egui::Button::new("🤚 Milk — Right"))
                .clicked()
            {
                self.feed_squeeze(Hand::Right);
            }
        });
        let both_locked = !Action::MilkBoth.is_unlocked(s.expression);
        if ui
            .add_sized(
                full,
                egui::Button::new(if both_locked {
                    "🔒 Both Hands (unlocks at Aroused)"
                } else {
                    "🙌 Both Hands (+50% flow, tighter window)"
                }),
            )
            .clicked()
        {
            self.feed_squeeze(Hand::Both);
        }
        ui.horizontal(|ui| {
            let caress_locked = !Action::Caress.is_unlocked(s.expression);
            if ui
                .add_sized(
                    half,
                    egui::Button::new(if caress_locked {
                        "🔒 Caress (at Smiling)"
                    } else {
                        "💗 Caress Cheek"
                    }),
                )
                .clicked()
            {
                self.feed_side(Action::Caress);
            }
            let spank_locked = !Action::Spank.is_unlocked(s.expression);
            if ui
                .add_sized(
                    half,
                    egui::Button::new(if spank_locked {
                        "🔒 Spank (at Flushed)"
                    } else {
                        "🍑 Spank"
                    }),
                )
                .clicked()
            {
                self.feed_side(Action::Spank);
            }
        });
        ui.horizontal(|ui| {
            if ui
                .add_sized(half, egui::Button::new("⏱ Wait one beat"))
                .clicked()
            {
                let tick = self.engine.snapshot().tick;
                self.engine
                    .sim_mut()
                    .advance_to(tick + self.cfg.ideal_interval_ticks);
            }
            ui.checkbox(&mut self.hard, "Hard squeeze (pressure 950)");
        });

        // The live sensitivity dial (SPEC §5.1): edits land in
        // the sim during play; the response changes in real
        // time. The dial reads and writes the shared settings
        // store (the same per-host values the Settings screen
        // edits) — one source of truth; her reservoir and
        // refractory stay her own tuned profile's.
        ui.add_space(6.0);
        ui.heading("Sensitivity dial (live)");
        let host = self.engine.host();
        let mut tuning = self.settings.tuning_for(host.name);
        let before = tuning;
        let gain_resp = ui.add(
            egui::Slider::new(
                &mut tuning.gain,
                hands_on::settings::GAIN_MIN..=hands_on::settings::GAIN_MAX,
            )
            .text("gain")
            .fixed_decimals(2),
        );
        let thr_resp = ui.add(
            egui::Slider::new(
                &mut tuning.threshold,
                hands_on::settings::THRESHOLD_MIN..=hands_on::settings::THRESHOLD_MAX,
            )
            .text("threshold")
            .fixed_decimals(0),
        );
        ui.horizontal(|ui| {
            if ui.button("Preset: balanced").clicked() {
                tuning =
                    hands_on::settings::HostTuning::from_profile(SensitivityProfile::default());
            }
            if ui.button("Preset: HIGH").clicked() {
                tuning = hands_on::settings::HostTuning::from_profile(SensitivityProfile::high());
            }
            if ui.button("Preset: LOW").clicked() {
                tuning = hands_on::settings::HostTuning::from_profile(SensitivityProfile::low());
            }
        });
        if tuning != before {
            self.settings.set_tuning(host.name, tuning);
            let profile = self.settings.effective_profile(&host);
            self.engine.sim_mut().set_profile(profile);
        }
        if gain_resp.drag_stopped()
            || thr_resp.drag_stopped()
            || gain_resp.lost_focus()
            || thr_resp.lost_focus()
        {
            self.persist_settings();
        }
        let p = self.engine.sim().profile();
        ui.weak(format!(
            "gain {:.2} · threshold {:.0} · reservoir cap {:.0} · refractory {} ticks",
            p.gain, p.threshold, p.reservoir_capacity, p.refractory_ticks
        ));

        ui.add_space(6.0);
        ui.heading("Session log");
        for line in self.engine.log_tail(5) {
            ui.label(format!("· {line}"));
        }
        // Repaint discipline: 16 ms while any animation is
        // live (pulse / burst / squash), the standing 33 ms
        // session cadence otherwise. The sim is tick-driven;
        // repaint rate never touches it.
        let animating = self.pulse.is_some_and(|p| p.active())
            || self.burst.active()
            || self
                .squash_start
                .is_some_and(|t| t.elapsed() < std::time::Duration::from_millis(300));
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(if animating {
                16
            } else {
                33
            }));
    }

    fn p4_ui(&mut self, ui: &mut egui::Ui) {
        self.header(ui, "P4 — Loop Studio (slice)");
        ui.label(
            "Author a keyform loop for the squeeze parameter. The gate is \
             mechanical: the loop must CLOSE (end value == start value) and \
             a twin-run bake from the named seed must come out byte-identical.",
        );
        ui.weak("seed: opal-loop-a1 · host: Mireille (this slice runs on Opal)");
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("cycle");
            ui.add(egui::DragValue::new(&mut self.studio.cycle_ms).range(100..=8000));
            ui.label("ms · easing");
            for (e, name) in [
                (Easing::Linear, "Linear"),
                (Easing::Smooth, "Smooth"),
                (Easing::EaseInOut, "EaseInOut"),
                (Easing::Hold, "Hold"),
            ] {
                ui.selectable_value(&mut self.studio.easing, e, name);
            }
        });

        // Keyform editor: one row per keyform.
        let mut remove: Option<usize> = None;
        for (i, k) in self.studio.keyforms.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("#{i}"));
                ui.add(egui::DragValue::new(&mut k.ms).range(0..=8000).prefix("t="));
                ui.label("ms");
                ui.add(egui::Slider::new(&mut k.value, 0.0..=1.0).text("value"));
                if ui.button("✕").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            self.studio.keyforms.remove(i);
            self.studio.gate = None;
            self.audio.play(Cue::UiBack);
        }
        ui.horizontal(|ui| {
            if ui.button("＋ Add keyform").clicked() {
                let value = self.studio.keyforms.first().map(|k| k.value).unwrap_or(0.0);
                self.studio.keyforms.push(KeyformDraft {
                    ms: self.studio.cycle_ms,
                    value,
                });
                self.studio.gate = None;
                self.audio.play(Cue::UiClick);
            }
            if ui.button("Sort by time").clicked() {
                self.studio.keyforms.sort_by_key(|k| k.ms);
                self.audio.play(Cue::UiClick);
            }
        });

        // Curve preview + playhead: the authored loop, looping.
        ui.add_space(6.0);
        match self.studio.track() {
            Ok(track) => {
                let (rect, _resp) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 150.0),
                    egui::Sense::hover(),
                );
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(0x18, 0x14, 0x22));
                let x_of = |ms: f64| {
                    rect.left() + (ms / f64::from(track.cycle_ms())) as f32 * rect.width()
                };
                let y_of = |v: f64| rect.bottom() - v as f32 * rect.height();
                let samples = 120usize;
                let pts: Vec<egui::Pos2> = (0..=samples)
                    .map(|i| {
                        let ms = f64::from(track.cycle_ms()) * i as f64 / samples as f64;
                        egui::pos2(x_of(ms), y_of(track.evaluate(ms as u32)))
                    })
                    .collect();
                painter.add(egui::Shape::line(
                    pts,
                    egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(0xff, 0x5c, 0x8a)),
                ));
                // Closure gap: draw the open seam in red at the right edge.
                if let Err(milking_core::authoring::TrackError::LoopOpen { start, end }) =
                    track.check_closure()
                {
                    painter.line_segment(
                        [
                            egui::pos2(rect.right(), y_of(start)),
                            egui::pos2(rect.right(), y_of(end)),
                        ],
                        egui::Stroke::new(3.0_f32, egui::Color32::from_rgb(0xff, 0x3b, 0x30)),
                    );
                }
                for k in track.keyforms() {
                    painter.circle_filled(
                        egui::pos2(x_of(f64::from(k.at_ms)), y_of(k.value)),
                        4.0,
                        egui::Color32::from_rgb(0xff, 0xd1, 0x66),
                    );
                }
                // Playhead: loops with session time, value read out below.
                let cycle = u128::from(track.cycle_ms());
                let phase = (self.started.elapsed().as_millis() % cycle) as u32;
                let v = track.evaluate(phase);
                painter.line_segment(
                    [
                        egui::pos2(x_of(f64::from(phase)), rect.top()),
                        egui::pos2(x_of(f64::from(phase)), rect.bottom()),
                    ],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x8a, 0xe0, 0xa8)),
                );
                // The playhead's dot breathes (a 1 Hz swell) so
                // the moving point reads at a glance.
                let swell = 1.0
                    + 0.25 * (self.started.elapsed().as_secs_f32() * std::f32::consts::TAU).sin();
                painter.circle_filled(
                    egui::pos2(x_of(f64::from(phase)), y_of(v)),
                    4.5 * swell,
                    egui::Color32::from_rgb(0x8a, 0xe0, 0xa8),
                );
                ui.label(format!(
                    "preview: phase {phase} ms → squeeze parameter {v:.2} (this curve drives the same spring the game uses)"
                ));
                ui.label(format!(
                    "tempo score vs the 1000 ms ideal cycle: {:.0}/100",
                    track.tempo_score(1000)
                ));
            }
            Err(e) => {
                ui.colored_label(egui::Color32::from_rgb(0xff, 0x3b, 0x30), e);
            }
        }

        ui.add_space(6.0);
        if ui
            .add_sized(
                [ui.available_width(), 52.0],
                egui::Button::new("🔁 Bake twice & run the gate"),
            )
            .clicked()
        {
            self.studio.run_gate();
            match &self.studio.gate {
                Some(g) if g.passed => {
                    self.audio.play(Cue::GatePass);
                    self.burst.spawn(18);
                    // The gate passing is P4's completion.
                    self.complete_level(menu::P4_KEY, "loop studio gate passed".to_string());
                }
                _ => self.audio.play(Cue::GateFail),
            }
        }
        if let Some(g) = &self.studio.gate {
            ui.label(format!(
                "closure: {} ({})",
                if g.closure_ok { "PASS" } else { "FAIL" },
                g.closure_note
            ));
            ui.label(format!(
                "bake @10 ms ({} frames) · twin-run hashes {:016x} / {:016x} · identical: {}",
                g.frames,
                g.hash_a,
                g.hash_b,
                g.hash_a == g.hash_b
            ));
            ui.label(format!(
                "gate tempo score vs the 1000 ms ideal cycle: {:.0}/100",
                g.tempo_score
            ));
            if g.passed {
                ui.colored_label(
                    egui::Color32::from_rgb(0x7d, 0xe0, 0x8a),
                    "GATE: PASS — Opal runs your loop seamlessly, identically, twice.",
                );
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(0xff, 0x3b, 0x30),
                    "GATE: FAIL — the loop does not close (a loop that does not close cannot loop seamlessly in an engine).",
                );
            }
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    }

    /// Drive the screenshot plan (desktop metadata capture):
    /// the menu system's three new screens first (home,
    /// picker, settings), then a played P1 session, then exit.
    fn shot_step(&mut self, ctx: &egui::Context) {
        let Some(_dir) = &self.shot_dir else { return };
        // The plan is frame-driven: keep the repaint loop
        // alive until it completes, or a headless run stalls
        // between stages waiting for input that never comes.
        if self.shot_stage < 4 {
            ctx.request_repaint();
        }
        self.frame_no += 1;
        let due = match self.shot_stage {
            0 => self.frame_no == 5,
            1..=3 => self.frame_no >= self.shot_base + 20,
            _ => false,
        };
        if due && !self.shot_requested {
            self.shot_requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        // Screenshot images arrive as events on a later frame.
        let shots: Vec<std::sync::Arc<egui::ColorImage>> = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
                .collect()
        });
        for image in shots {
            // On Android the save below is cfg'd out; the shot
            // still drives the stage machine.
            let _ = &image;
            #[cfg(not(target_os = "android"))]
            self.save_shot(&image);
            self.shot_requested = false;
            self.shot_base = self.frame_no;
            match self.shot_stage {
                0 => {
                    self.shot_stage = 1;
                    self.go(Screen::Picker);
                }
                1 => {
                    self.shot_stage = 2;
                    self.go(Screen::Settings);
                }
                2 => {
                    // Move to P1 and play a few on-beat squeezes so
                    // the screenshot shows a live session, not an
                    // empty room.
                    self.shot_stage = 3;
                    self.go(Screen::P1);
                    self.engine
                        .sim_mut()
                        .set_profile(SensitivityProfile::high());
                    for i in 0..6u64 {
                        let tick = i * self.cfg.ideal_interval_ticks;
                        let _ = self.engine.feed(InputEvent::Squeeze {
                            tick,
                            hand: if i % 2 == 0 { Hand::Left } else { Hand::Right },
                            pressure: 700,
                        });
                    }
                }
                3 => {
                    self.shot_stage = 4;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                _ => {}
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    fn save_shot(&self, image: &egui::ColorImage) {
        let Some(dir) = &self.shot_dir else { return };
        let name = match self.shot_stage {
            0 => "menu.png",
            1 => "picker.png",
            2 => "settings.png",
            _ => "p1-live.png",
        };
        let path = dir.join(name);
        let [w, h] = image.size;
        let mut pixels = Vec::with_capacity(w * h * 4);
        for c in &image.pixels {
            pixels.extend_from_slice(&c.to_array());
        }
        if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, pixels) {
            let _ = buf.save(&path);
        }
    }
}

impl eframe::App for HandsOnApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.styled {
            self.styled = true;
            ctx.style_mut(|s| {
                s.spacing.button_padding = egui::vec2(14.0, 10.0);
                s.spacing.item_spacing = egui::vec2(8.0, 8.0);
                if let Some(body) = s.text_styles.get_mut(&egui::TextStyle::Body) {
                    body.size = 17.0;
                }
                if let Some(button) = s.text_styles.get_mut(&egui::TextStyle::Button) {
                    button.size = 17.0;
                }
                // The Ranch theme (menu module): the mark's
                // near-black indigo ground, its violet-black
                // panels, brass/teal accents in the labels.
                let mut v = egui::Visuals::dark();
                v.panel_fill = egui::Color32::from_rgb(0x0d, 0x0b, 0x16);
                v.window_fill = egui::Color32::from_rgb(0x0d, 0x0b, 0x16);
                v.extreme_bg_color = egui::Color32::from_rgb(0x07, 0x05, 0x0e);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x18, 0x14, 0x22);
                v.widgets.noninteractive.weak_bg_fill = egui::Color32::from_rgb(0x18, 0x14, 0x22);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(0x21, 0x1a, 0x30);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x2d, 0x21, 0x42);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(0x3a, 0x2a, 0x55);
                v.selection.bg_fill = egui::Color32::from_rgb(0x5a, 0x3d, 0x8a);
                v.hyperlink_color = menu::TEAL;
                s.visuals = v;
            });
            self.audio.set_bed(Bed::Menu);
        }
        self.advance_clock();
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Screen-entry rise (transition grammar).
                if let Some(tr) = &self.transition
                    && tr.active()
                {
                    ui.add_space(tr.rise());
                }
                match self.screen {
                    Screen::Menu => self.menu_home_ui(ui),
                    Screen::Picker => self.picker_ui(ui),
                    Screen::SaveLoad => self.saveload_ui(ui),
                    Screen::Settings => self.settings_ui(ui),
                    Screen::P1 => self.p1_ui(ui),
                    Screen::P4 => self.p4_ui(ui),
                    Screen::Console => self.console_ui(ui),
                }
            });
        });
        if let Some(nav) = self.pending_nav.take() {
            self.go(nav);
        }
        // Transition veil: a foreground wash that fades out
        // over the new screen (anim::Transition, 240 ms).
        if let Some(tr) = &self.transition
            && tr.active()
        {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("screen-veil"),
            ));
            let rect = ctx.screen_rect();
            painter.rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(
                    0x0a,
                    0x0a,
                    0x0e,
                    (tr.veil_alpha() * 255.0) as u8,
                ),
            );
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
        self.shot_step(ctx);
    }
}

/// Scripted headless session: the same proof the TUI's `--demo`
/// runs, through this crate's own wiring — P1 on the ideal beat
/// with a determinism twin-check, and the P4 gate on the starter
/// track. Used by `hands-on-app --smoke` and by the tests.
///
/// # Errors
/// A human-readable failure line when any assertion fails.
pub fn smoke() -> Result<String, String> {
    let cfg = Config::default();
    let run_p1 = || {
        let level = MilkingLevel::new();
        let mut engine = FeedbackEngine::new(level.descriptor().host, cfg);
        engine.sim_mut().set_profile(SensitivityProfile::high());
        for i in 0..14u64 {
            engine
                .feed(InputEvent::Squeeze {
                    tick: i * cfg.ideal_interval_ticks,
                    hand: if i % 2 == 0 { Hand::Left } else { Hand::Right },
                    pressure: 700,
                })
                .map_err(|e| format!("P1 feed failed: {e:?}"))?;
        }
        engine.sim_mut().advance_to(14 * cfg.ideal_interval_ticks);
        Ok::<_, String>(engine)
    };
    let a = run_p1()?;
    let b = run_p1()?;
    let s = a.snapshot();
    if s.bottle <= 0.0 {
        return Err(format!("P1 smoke: bottle did not fill ({})", s.bottle));
    }
    if a.sim().trace_bytes() != b.sim().trace_bytes() {
        return Err("P1 smoke: twin runs diverged".to_string());
    }
    let mut studio = StudioState::starter();
    studio.run_gate();
    let gate = studio.gate.ok_or("P4 smoke: gate did not run")?;
    if !gate.passed {
        return Err("P4 smoke: starter track failed its own gate".to_string());
    }
    Ok(format!(
        "smoke ok: P1 tick {} expression {:?} bottle {:.1} climaxes {} spills {} · twin-run identical · P4 gate PASS (hash {:016x}, {} frames)",
        s.tick, s.expression, s.bottle, s.climax_count, s.spills, gate.hash_a, gate.frames
    ))
}

/// Android entry point: the activity hands us the app handle
/// and we run the same native shell as the desktop binary.
/// The symbol name is the platform contract (android-activity's
/// onCreate shim calls `android_main` by name), and edition
/// 2024 spells the exporting attribute `#[unsafe(no_mangle)]`
/// — the one item-level unsafe exception in the crate (see
/// the crate header for why this crate denies, not forbids).
#[cfg(target_os = "android")]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
fn android_main(app: android_activity::AndroidApp) {
    // The store's home on Android is the activity's internal
    // data path (the platform's app-data location); hand it
    // to the core store before the app constructs.
    if let Some(files) = app.internal_data_path() {
        hands_on::save::set_data_dir_override(files);
    }
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Hands-On",
        options,
        Box::new(|_cc| Ok(Box::new(HandsOnApp::new()))),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use hands_on::level::{playroom_levels, study_levels};

    #[test]
    fn smoke_session_passes_both_levels() {
        let line = smoke().expect("smoke must pass");
        assert!(line.contains("twin-run identical"), "{line}");
        assert!(line.contains("P4 gate PASS"), "{line}");
    }

    #[test]
    fn registry_covers_all_seventeen_levels_all_built() {
        // The session layer built the remaining fifteen; this
        // shell renders P1 and P4 and labels the rest honestly.
        let built: Vec<u8> = playroom_levels()
            .iter()
            .filter(|l| l.built)
            .map(|l| l.id)
            .collect();
        assert_eq!(built, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(playroom_levels().len() + study_levels().len(), 17);
        assert!(study_levels().iter().all(|l| l.built));
    }

    #[test]
    fn app_starts_on_the_menu_with_a_live_engine() {
        let app = HandsOnApp::new();
        assert_eq!(app.screen, Screen::Menu);
        assert_eq!(app.engine.host().name, "Opal");
    }

    #[test]
    fn open_loop_fails_the_studio_gate() {
        let mut studio = StudioState::starter();
        studio.keyforms[2].value = 0.4; // end != start: the loop pops
        studio.run_gate();
        let gate = studio.gate.expect("gate runs on a valid track");
        assert!(!gate.closure_ok);
        assert!(!gate.passed);
    }
}
