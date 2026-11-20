// SPDX-License-Identifier: MIT
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use freya::prelude::*;
fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Runtime initialization failed");
    let _guard = runtime.enter();
    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(app)
                .with_title("Aero Client")
                .with_size(1180., 760.),
        ),
    );
}
fn app() -> impl IntoElement {
    rect()
        .expanded()
        .background((234, 248, 255))
        .color((18, 60, 86))
        .padding(24.)
        .child(label().text("Aero Client").font_size(28.))
        .child("Native Rust launcher — integration validation in progress")
}
