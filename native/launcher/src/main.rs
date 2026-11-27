// SPDX-License-Identifier: GPL-3.0-only
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use freya::prelude::*;
use std::io::Write;
mod backend;
mod design;
mod pages;
mod shell;
mod ui;

fn main() {
    install_panic_reporter();
    let (window_width, window_height) = initial_window_size();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Runtime initialization failed");
    let _guard = runtime.enter();
    let root = aero_core::Store::default_root();
    let settings = runtime
        .block_on(aero_core::storage::Settings::load(&root))
        .unwrap_or_default();
    let account_result = runtime.block_on(aero_core::auth::load_accounts(&root));
    let account_error = account_result.as_ref().err().map(ToString::to_string);
    let accounts = account_result.unwrap_or_default();
    let onboarding = runtime.block_on(aero_core::onboarding::load(&root));
    let backend = backend::Backend::start(settings, accounts, onboarding, account_error);
    launch(
        LaunchConfig::new()
            // 32 MiB of GPU resource cache holds the static landscape, the logo and
            // the composed character; the window repaints only on input.
            .with_gpu_resource_cache_limit(32usize * 1024 * 1024)
            .with_window(
                WindowConfig::new_app(ui::Aero { backend })
                    .with_title("Aero Client")
                    .with_size(window_width, window_height)
                    // The responsive shell is verified down to this supported size.
                    // Refusing a smaller surface avoids hiding required setup and
                    // launch controls behind the native window frame.
                    .with_min_size(1000., 640.),
            ),
    );
}

fn initial_window_size() -> (f64, f64) {
    fn dimension(name: &str, fallback: f64, minimum: f64) -> f64 {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .map(|value| value.max(minimum))
            .unwrap_or(fallback)
    }

    (
        dimension("AERO_WINDOW_WIDTH", 1180., 1000.),
        dimension("AERO_WINDOW_HEIGHT", 760., 640.),
    )
}

fn install_panic_reporter() {
    std::panic::set_hook(Box::new(|info| {
        let directory = aero_core::Store::default_root().join("logs");
        if std::fs::create_dir_all(&directory).is_err() {
            return;
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(directory.join("panic.log"))
        {
            let location = info
                .location()
                .map(|value| format!("{}:{}", value.file(), value.line()))
                .unwrap_or_else(|| "unknown location".into());
            // Never serialize the panic payload: it might contain private state.
            let _ = writeln!(file, "Unexpected launcher failure at {location}");
        }
    }));
}
