// SPDX-License-Identifier: MIT
//! App root: owns all reactive state, routes screens and composes the window. Setup
//! is shown until it is completed; after that the rail, top bar and pages take over.
use crate::{backend::Backend, design::*, pages, shell};
use freya::{prelude::*, sdk::use_track_watcher};

pub struct Aero {
    pub backend: Backend,
}

#[derive(Clone, Copy)]
pub struct Ui {
    pub page: State<usize>,
    pub detail: State<String>,
    pub appearance: State<String>,
    pub theme: State<Theme>,
    pub memory: State<String>,
    pub java: State<String>,
    pub low: State<bool>,
    pub reduced: State<bool>,
    pub name: State<String>,
    pub version: State<String>,
    pub loader: State<String>,
    pub search: State<String>,
    pub query: State<String>,
    pub kind: State<String>,
    pub confirm: State<String>,
    pub rename: State<String>,
    pub inst_memory: State<String>,
    pub inst_java: State<String>,
    pub w_perf: State<String>,
    pub w_auto: State<bool>,
}

impl App for Aero {
    fn render(&self) -> impl IntoElement {
        use_track_watcher(&self.backend.rx);
        let snapshot = self.backend.rx.borrow().clone();
        let backend = self.backend.clone();
        let settings = snapshot.settings.clone();

        let theme_state = use_init_theme(|| theme(&settings.appearance));
        let mut ui = Ui {
            page: use_state(|| 0usize),
            detail: use_state(String::new),
            appearance: use_state(|| settings.appearance.clone()),
            theme: theme_state,
            memory: use_state(|| settings.memory_mib.to_string()),
            java: use_state(|| settings.custom_java.clone()),
            low: use_state(|| settings.low_performance),
            reduced: use_state(|| settings.reduced_motion),
            name: use_state(|| "My Minecraft".to_owned()),
            version: use_state(|| snapshot.onboarding.answers.version.clone()),
            loader: use_state(|| snapshot.onboarding.answers.loader.clone()),
            search: use_state(String::new),
            query: use_state(String::new),
            kind: use_state(|| "mod".to_owned()),
            confirm: use_state(String::new),
            rename: use_state(String::new),
            inst_memory: use_state(String::new),
            inst_java: use_state(String::new),
            w_perf: use_state(|| snapshot.onboarding.answers.performance.clone()),
            w_auto: use_state(|| snapshot.onboarding.answers.auto_java),
        };

        // The appearance preset is the source of truth; changing it re-themes
        // every control at once.
        if ui.theme.read().name != ui.appearance.read().as_str() {
            ui.theme.set(theme(&ui.appearance.read().clone()));
        }
        let p = palette(&ui.appearance.read().clone());
        let index = if snapshot.onboarding.needs_setup() {
            pages::WIZARD
        } else {
            *ui.page.read()
        };

        let mut root = rect()
            .expanded()
            .color(p.text)
            .font_size(13.)
            .font_family(FONT)
            .background(p.bg);
        if p.photo && index != pages::WIZARD {
            root = root.child(landscape());
        }
        if index == pages::WIZARD {
            return root
                .child(pages::page(pages::WIZARD, &ui, &snapshot, p, &backend))
                .into_element();
        }
        #[allow(clippy::needless_return)]
        root.child(
            rect()
                .expanded()
                .horizontal()
                .content(Content::Flex)
                .child(shell::rail(&ui, p))
                .child(
                    rect()
                        .width(Size::flex(1.))
                        .height(Size::fill())
                        .content(Content::Flex)
                        .child(shell::top_bar(&ui, &snapshot, p, &backend))
                        .child(
                            rect()
                                .width(Size::fill())
                                .height(Size::flex(1.))
                                .child(pages::page(index, &ui, &snapshot, p, &backend)),
                        )
                        .maybe(snapshot.busy || snapshot.launching, |r| {
                            r.child(shell::status_strip(
                                p,
                                &snapshot.message,
                                &snapshot.progress,
                                snapshot.busy,
                                snapshot.launching,
                            ))
                        }),
                ),
        )
        .into_element()
    }
}
