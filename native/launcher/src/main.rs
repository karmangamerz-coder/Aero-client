// SPDX-License-Identifier: MIT
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use freya::prelude::*;
mod backend;
mod design;
mod ui;
fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Runtime initialization failed");
    let _guard = runtime.enter();
    let settings = runtime
        .block_on(aero_core::storage::Settings::load(
            &aero_core::Store::default_root(),
        ))
        .unwrap_or_default();
    let backend = backend::Backend::start(settings);
    launch(
        LaunchConfig::new()
            .with_gpu_resource_cache_limit(32usize * 1024 * 1024)
            .with_window(
                WindowConfig::new_app(ui::Aero { backend })
                    .with_title("Aero Client")
                    .with_size(1180., 760.),
            ),
    );
}
