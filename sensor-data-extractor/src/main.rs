#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod async_runtime;
mod commands;
mod serial_messages;
mod ui;
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types

use std::thread::JoinHandle;

use eframe::egui;
use tokio_serial::{SerialPortInfo, SerialStream};

fn main() {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        resizable: true,
        initial_window_size: Some(egui::vec2(1600.0, 900.0)),
        ..Default::default()
    };
}
