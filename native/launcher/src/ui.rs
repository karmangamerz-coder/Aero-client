// SPDX-License-Identifier: MIT
//! App root: owns every piece of reactive state, routes pages and composes the
//! window chrome. Rendering is event driven — no timers, no polling, no animation
//! loop — which keeps idle CPU near zero.
use crate::{backend::Backend, design::*, pages, shell};
use freya::{prelude::*, sdk::use_track_watcher};

pub struct Aero {
    pub backend: Backend,
}

#[derive(Clone, Copy)]
pub struct Ui {
    pub page: State<usize>,
    pub detail: State<String>,
    pub selected: State<String>,
    pub rotation: State<usize>,
    pub news_visible: State<bool>,
    pub appearance: State<String>,
    pub theme: State<Theme>,
    pub memory: State<u32>,
    pub java: State<String>,
    pub low: State<bool>,
    pub reduced: State<bool>,
    pub show_news: State<bool>,
    pub name: State<String>,
    pub version: State<String>,
    pub loader: State<String>,
    pub search: State<String>,
    pub query: State<String>,
    pub kind: State<String>,
    pub filter_version: State<String>,
    pub filter_loader: State<String>,
    pub confirm: State<String>,
    pub rename: State<String>,
    pub inst_memory: State<String>,
    pub inst_java: State<String>,
}

impl App for Aero {
    fn render(&self) -> impl IntoElement {
        use_track_watcher(&self.backend.rx);
        let snapshot = self.backend.rx.borrow().clone();
        let backend = self.backend.clone();
        let settings = snapshot.settings.clone();

        let theme_state = use_init_theme(|| theme(&settings.appearance));
        let ui = Ui {
            page: use_state(|| 0usize),
            detail: use_state(String::new),
            selected: use_state(String::new),
            rotation: use_state(|| 5usize),
            news_visible: use_state(|| settings.show_news),
            appearance: use_state(|| settings.appearance.clone()),
            theme: theme_state,
            memory: use_state(|| settings.memory_mib),
            java: use_state(|| settings.custom_java.clone()),
            low: use_state(|| settings.low_performance),
            reduced: use_state(|| settings.reduced_motion),
            show_news: use_state(|| settings.show_news),
            name: use_state(|| "My Minecraft".to_owned()),
            version: use_state(|| "1.21.11".to_owned()),
            loader: use_state(|| "Fabric".to_owned()),
            search: use_state(String::new),
            query: use_state(String::new),
            kind: use_state(|| "mod".to_owned()),
            filter_version: use_state(String::new),
            filter_loader: use_state(String::new),
            confirm: use_state(String::new),
            rename: use_state(String::new),
            inst_memory: use_state(String::new),
            inst_java: use_state(String::new),
        };

        let p = palette(&ui.appearance.read().clone());
        let chosen_id = ui.selected.read().clone();
        let chosen = snapshot
            .instances
            .iter()
            .find(|i| i.id == chosen_id)
            .or_else(|| snapshot.instances.first())
            .cloned();

        let content = pages::page(*ui.page.read(), &ui, &snapshot, p, &backend);

        let mut root = rect()
            .expanded()
            .color(p.text)
            .font_size(13.)
            .font_family(FONT)
            .background(p.bg);
        if p.photo {
            root = root.child(landscape());
        }
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
                        .child(shell::top_bar(
                            &ui,
                            p,
                            &snapshot.settings,
                            &snapshot.instances,
                            chosen.as_ref(),
                            &backend,
                            snapshot.busy,
                        ))
                        .child(
                            rect()
                                .width(Size::fill())
                                .height(Size::flex(1.))
                                .child(content),
                        )
                        .child(shell::status_bar(
                            p,
                            &snapshot.message,
                            &snapshot.progress,
                            snapshot.busy,
                        )),
                ),
        )
    }
}
