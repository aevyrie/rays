#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use rays_gui_lib::RaysApp;

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::Vec2::new(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Rays",
        native_options,
        Box::new(|cc| Box::new(RaysApp::new(cc))),
    );
}
