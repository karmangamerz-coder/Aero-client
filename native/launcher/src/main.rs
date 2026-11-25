// SPDX-License-Identifier: GPL-3.0-only
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use freya::prelude::*;
mod backend;
mod design;
mod pages;
mod shell;
mod ui;

fn main() {
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
    let accounts = runtime
        .block_on(aero_core::auth::load_accounts(&root))
        .unwrap_or_default();
    let onboarding = runtime.block_on(aero_core::onboarding::load(&root));
    let backend = backend::Backend::start(settings, accounts, onboarding);
    launch(
        LaunchConfig::new()
            // 32 MiB of GPU resource cache holds the static landscape, the logo and
            // the composed character; the window repaints only on input.
            .with_gpu_resource_cache_limit(32usize * 1024 * 1024)
            .with_window(
                WindowConfig::new_app(ui::Aero { backend })
                    .with_title("Aero Client")
                    .with_size(1180., 760.)
                    .with_min_size(880., 560.),
            ),
    );
}
