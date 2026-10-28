//! hands-on-app — desktop entry point for the egui front-end.
//!
//! ```text
//! hands-on-app           the touch-first GUI (desktop shell)
//! hands-on-app --smoke   scripted headless session, prints, exits
//! ```
//!
//! On Android there is no `main`: the activity loads the crate
//! as a library and enters through `android_main` in the lib.
#![forbid(unsafe_code)]

fn main() -> eframe::Result<()> {
    if std::env::args().any(|a| a == "--smoke") {
        match hands_on_app::smoke() {
            Ok(line) => {
                println!("{line}");
                return Ok(());
            }
            Err(e) => {
                eprintln!("smoke FAILED: {e}");
                std::process::exit(1);
            }
        }
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Hands-On")
            .with_inner_size([480.0, 860.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Hands-On",
        options,
        Box::new(|_cc| Ok(Box::new(hands_on_app::HandsOnApp::new()))),
    )
}
