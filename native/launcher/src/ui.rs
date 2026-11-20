// SPDX-License-Identifier: MIT
use crate::{
    backend::{Backend, Command},
    design::*,
};
use aero_core::{Loader, Store, storage::Settings};
use freya::{prelude::*, sdk::use_track_watcher};
pub struct Aero {
    pub backend: Backend,
}
impl App for Aero {
    fn render(&self) -> impl IntoElement {
        use_track_watcher(&self.backend.rx);
        let s = self.backend.rx.borrow().clone();
        let b = self.backend.clone();
        let initial = s.settings.clone();
        let mut active_theme = use_init_theme(|| theme(&initial.appearance));
        let mut page = use_state(|| 0usize);
        let mut selected = use_state(String::new);
        let mut rotation = use_state(|| 5usize);
        let mut news_visible = use_state(|| initial.show_news);
        let search = use_state(String::new);
        let name = use_state(|| "My Minecraft".to_string());
        let version = use_state(|| "1.21.11".to_string());
        let loader = use_state(|| "Fabric".to_string());
        let rename = use_state(String::new);
        let mut confirm = use_state(String::new);
        let query = use_state(String::new);
        let kind = use_state(|| "mod".to_string());
        let filter_version = use_state(String::new);
        let filter_loader = use_state(String::new);
        let mut appearance = use_state(|| initial.appearance.clone());
        let mut memory = use_state(|| initial.memory_mib);
        let java = use_state(|| initial.custom_java.clone());
        let low = use_state(|| initial.low_performance);
        let reduced = use_state(|| initial.reduced_motion);
        let show_news = use_state(|| initial.show_news);
        let dark = *appearance.read() == "Aero Dark";
        let chosen = s
            .instances
            .iter()
            .find(|i| i.id == *selected.read())
            .or_else(|| s.instances.first())
            .cloned();
        let chosen_id = chosen.as_ref().map(|i| i.id.clone()).unwrap_or_default();
        let selector = Select::new()
            .selected_item(
                chosen
                    .as_ref()
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| "Choose an instance".into()),
            )
            .children(s.instances.iter().map(|i| {
                let id = i.id.clone();
                MenuItem::new()
                    .selected(id == chosen_id)
                    .on_press(move |_| selected.set(id.clone()))
                    .child(format!("{} · {}", i.name, i.version))
            }));
        let rail = rect()
            .width(Size::px(64.))
            .height(Size::fill())
            .padding((18., 8.))
            .spacing(14.)
            .background(glass(dark))
            .child(
                SvgViewer::new(include_bytes!("../assets/logo.svg"))
                    .width(Size::px(45.))
                    .height(Size::px(42.)),
            )
            .children(
                [
                    "Play",
                    "Instances",
                    "Discover",
                    "Customization",
                    "Downloads",
                    "Settings",
                ]
                .into_iter()
                .enumerate()
                .map(|(index, title)| {
                    TooltipContainer::new(Tooltip::new_text(title)).child(
                        Button::new()
                            .theme_layout(
                                ButtonLayoutThemePartial::default()
                                    .width(Size::px(46.))
                                    .height(Size::px(44.))
                                    .padding(9.)
                                    .corner_radius(8.),
                            )
                            .theme_colors(
                                ButtonColorsThemePartial::default()
                                    .background(if page() == index {
                                        (141, 216, 248)
                                    } else if dark {
                                        (84, 130, 148)
                                    } else {
                                        (234, 248, 255)
                                    })
                                    .hover_background((173, 232, 247)),
                            )
                            .on_press(move |_| page.set(index))
                            .child(icon(index)),
                    )
                }),
            );
        let header = rect()
            .horizontal()
            .width(Size::fill())
            .height(Size::px(66.))
            .padding((12., 20.))
            .spacing(16.)
            .cross_align(Alignment::Center)
            .background(glass(dark))
            .child(label().text("AERO CLIENT").font_size(17.))
            .child(selector)
            .child(
                label()
                    .text(
                        chosen
                            .as_ref()
                            .map(|i| format!("{} / {}", i.version, i.loader))
                            .unwrap_or_default(),
                    )
                    .font_size(12.),
            )
            .child(rect().width(Size::flex(1.)))
            .child(action("Guest · account settings").on_press(move |_| page.set(5)));
        let content: Element = match page() {
        0 => {
            let id = chosen_id.clone(); let install = b.clone();
            let text = if s.busy { "Installing…" } else if chosen.as_ref().is_some_and(|i| i.status == "READY") { "Repair installation" } else { "Install Minecraft" };
            let showcase = rect().height(Size::fill()).width(Size::flex(1.)).center().spacing(10.)
                .child(label().text("Your next world awaits.").font_size(25.).color((18,60,86)))
                .child(SvgViewer::new(CHARACTERS[if low() { 5 } else { rotation() }]).width(Size::px(270.)).height(Size::px(342.))
                    .a11y_alt("Original Aero explorer. Move the pointer to rotate.")
                    .on_pointer_move(move |e: Event<PointerEventData>| { if !low() { let next = ((e.element_location().x / 270. * 9.) as usize).min(8); if rotation() != next { rotation.set(next); } } }))
                .child(label().text(chosen.as_ref().map(|i| i.name.clone()).unwrap_or_else(|| "Welcome, explorer".into())).font_size(20.).color((18,60,86)))
                .child(label().text("Guest · Microsoft account not connected").font_size(12.).color((18,60,86)))
                .child(primary(text).enabled(!s.busy && chosen.is_some()).theme_layout(ButtonLayoutThemePartial::default().width(Size::px(266.)).height(Size::px(51.)))
                    .on_press(move |_| { install.send(Command::Install(id.clone())); page.set(4); }))
                .child(action(if chosen.is_some() { "Manage instances ▾" } else { "Create your first instance" }).on_press(move |_| page.set(1)))
                .child(label().text("Sign-in and game launch are coming in a future release.").font_size(11.).color((18,60,86)));
            let news = panel(dark).width(Size::percent(24.)).height(Size::fill())
                .child(rect().horizontal().spacing(10.).child(label().text("What's new").font_size(19.)).child(action("Hide").on_press(move |_| news_visible.set(false))))
                .child(ScrollView::new().height(Size::flex(1.)).children(s.news.iter().map(|n| rect().width(Size::fill()).spacing(8.).padding((0.,0.,20.,0.))
                    .child(SvgViewer::new(include_bytes!("../assets/landscape.svg")).width(Size::fill()).height(Size::px(92.)))
                    .child(label().text(n.title.clone()).font_size(16.)).child(label().text(format!("{} · {}", n.source,n.date)).font_size(11.)))))
                .maybe(s.news.is_empty(), |r| r.child("No cached announcements available. News appears when the sources respond."));
            rect().horizontal().content(Content::Flex).expanded().padding(20.).spacing(16.).child(showcase)
                .maybe(news_visible(), |r| r.child(news)).maybe(!news_visible(), |r| r.child(action("News").on_press(move |_| news_visible.set(true)))).into_element()
        }
        1 => {
            let create = b.clone(); let default = b.clone(); let check = b.clone();
            let form = panel(dark).width(Size::fill()).child(heading("Create an instance", "Select Install after creating an instance to download game files."))
                .child(rect().horizontal().spacing(12.).child(Input::new(name).placeholder("Instance name"))
                    .child(choice(version, aero_core::TARGETS.iter().map(|v| v.to_string()).collect())).child(choice(loader, vec!["Vanilla".into(),"Fabric".into(),"Forge".into()])))
                .child(rect().horizontal().spacing(8.)
                    .child(primary("Create custom").enabled(!s.busy).on_press(move |_| create.send(Command::Create(name.read().clone(),version.read().clone(),parse_loader(&loader.read()),false))))
                    .child(action("Add Aero Default").enabled(!s.busy).on_press(move |_| default.send(Command::Create(format!("Aero Default {}",version.read()),version.read().clone(),parse_loader(&loader.read()),true))))
                    .child(action("Check availability").enabled(!s.busy).on_press(move |_| check.send(Command::Refresh))))
                .child(label().text(format!("Verified available: {}", if s.versions.is_empty() { "not checked yet".into() } else { s.versions.join(", ") })).font_size(12.));
            let rows = s.instances.iter().filter(|i| format!("{} {} {}",i.name,i.version,i.loader).to_lowercase().contains(&search.read().to_lowercase())).map(|i| {
                let id = i.id.clone(); let use_id = id.clone(); let install_id = id.clone(); let duplicate_id = id.clone(); let remove_id = id.clone();
                let install = b.clone(); let duplicate = b.clone(); let remove = b.clone();
                panel(dark).width(Size::fill()).child(rect().horizontal().spacing(16.).cross_align(Alignment::Center)
                    .child(SvgViewer::new(include_bytes!("../assets/landscape.svg")).width(Size::px(92.)).height(Size::px(62.)))
                    .child(rect().width(Size::flex(1.)).spacing(5.).child(label().text(i.name.clone()).font_size(18.)).child(label().text(format!("{} · {} · {}",i.version,i.loader,i.status)).font_size(12.))))
                    .child(rect().horizontal().spacing(8.)
                        .child(action("Select").on_press(move |_| { selected.set(use_id.clone()); page.set(0); }))
                        .child(action("Install / repair").enabled(!s.busy).on_press(move |_| { install.send(Command::Install(install_id.clone())); selected.set(install_id.clone()); page.set(4); }))
                        .child(action("Duplicate").enabled(!s.busy).on_press(move |_| duplicate.send(Command::Duplicate(duplicate_id.clone()))))
                        .child(action("Remove").enabled(!s.busy).on_press(move |_| confirm.set(remove_id.clone()))))
                    .maybe(*confirm.read() == id, |r| r.child("Move this instance to recoverable Trash? Saves are preserved there.")
                        .child(rect().horizontal().spacing(8.).child(action("Keep instance").on_press(move |_| confirm.set(String::new())))
                            .child(action("Move to Trash").on_press(move |_| { remove.send(Command::Remove(id.clone())); confirm.set(String::new()); }))))
            }).collect::<Vec<_>>();
            let edit = b.clone(); let id = chosen_id.clone();
            ScrollView::new().expanded().child(rect().width(Size::fill()).padding(22.).spacing(16.)
                .child(heading("Your instances", "Separate worlds, mods and settings. Verified game files are shared."))
                .child(Input::new(search).placeholder("Search names, versions or loaders")).children(rows).child(form)
                .maybe(chosen.is_some(), |r| r.child(panel(dark).width(Size::fill()).child("Rename selected instance")
                    .child(Input::new(rename).placeholder("New instance name"))
                    .child(action("Save name").enabled(!s.busy).on_press(move |_| edit.send(Command::Rename(id.clone(),rename.read().clone()))))))).into_element()
        }
        2 => {
            let discover = b.clone();
            ScrollView::new().expanded().child(rect().width(Size::fill()).padding(22.).spacing(16.)
                .child(heading("Discover", "Browse Modrinth. Installing projects is coming in a future release."))
                .child(panel(dark).width(Size::fill()).child(Input::new(query).placeholder("Search projects"))
                    .child(rect().horizontal().spacing(10.)
                        .child(choice(kind, vec!["mod".into(),"modpack".into(),"resourcepack".into(),"shader".into()]))
                        .child(choice(filter_version, std::iter::once(String::new()).chain(aero_core::TARGETS.iter().map(|v|v.to_string())).collect()))
                        .child(choice(filter_loader, vec![String::new(),"fabric".into(),"forge".into()]))
                        .child(primary("Search").enabled(!s.busy).on_press(move |_| discover.send(Command::Discover(query.read().clone(),kind.read().clone(),filter_version.read().clone(),filter_loader.read().clone())))))
                    .child(label().text("Empty filters include all versions / loaders. Requests go to Modrinth only when you search.").font_size(12.)))
                .children(s.results.iter().map(|item| panel(dark).width(Size::fill()).child(label().text(item.title.clone()).font_size(18.))
                    .child(label().text(item.description.clone()).font_size(13.)).child(label().text(format!("modrinth.com/{}/{}",item.project_type,item.slug)).font_size(11.))))).into_element()
        }
        3 => panel(dark).margin(24.).width(Size::fill()).child(heading("Make it yours", "Original Aero customization, free for everyone."))
            .child("Player skins, cosmetics and in-game customization are planned for future releases.")
            .child("The explorer preview is original fallback artwork. No account or paid cosmetic is required.")
            .child(action("Change launcher appearance").on_press(move |_| page.set(5))).into_element(),
        4 => {
            let cancel = b.clone(); let retry = b.clone(); let id = chosen_id.clone();
            panel(dark).margin(24.).width(Size::fill()).child(heading("Downloads", "Checksums are verified before files become available."))
                .child(label().text(s.progress.item.clone()).font_size(13.)).child(progress(s.progress.bytes,s.progress.total))
                .child(label().text(format!("{:.1} / {:.1} MiB · {:.1} MiB/s · {} / {} files",s.progress.bytes as f64/1048576.,s.progress.total as f64/1048576.,s.progress.bytes_per_second as f64/1048576.,s.progress.files_done,s.progress.files_total)).font_size(13.))
                .child(s.message.clone()).child(rect().horizontal().spacing(10.)
                    .child(action("Cancel").enabled(s.busy).on_press(move |_| cancel.cancel()))
                    .child(primary("Retry / repair selected").enabled(!s.busy && chosen.is_some()).on_press(move |_| retry.send(Command::Install(id.clone()))))).into_element()
        }
        _ => {
            let save = b.clone();
            ScrollView::new().expanded().child(rect().width(Size::fill()).padding(22.).spacing(16.)
                .child(heading("Settings", "Your preferences stay on this device."))
                .child(panel(dark).width(Size::fill()).child(label().text("Appearance").font_size(18.))
                    .child(rect().horizontal().spacing(10.).children(["Aero Light","Aero Dark","Aero Lite"].into_iter().map(move |value|
                        action(value).on_press(move |_| { appearance.set(value.into()); active_theme.set(theme(value)); }))))
                    .child(rect().horizontal().spacing(12.).child(Switch::new(show_news)).child("Show cached news on startup")))
                .child(panel(dark).width(Size::fill()).child(label().text("Minecraft & Java").font_size(18.))
                    .child(format!("Game memory: {} MiB",memory()))
                    .child(Slider::new(move |percent| memory.set((1024. + percent / 100. * 31744.).round() as u32)).value((memory()-1024) as f64/31744.*100.).size(Size::px(420.)).step(1.))
                    .child("Game Java is detected or downloaded automatically from Mojang.")
                    .child(Input::new(java).placeholder("Optional full path to a compatible java.exe")))
                .child(panel(dark).width(Size::fill()).child(label().text("Accessibility & performance").font_size(18.))
                    .child(rect().horizontal().spacing(12.).child(Switch::new(low)).child("Low-performance mode: static character"))
                    .child(rect().horizontal().spacing(12.).child(Switch::new(reduced)).child("Reduced motion (no idle animations)")))
                .child(panel(dark).width(Size::fill()).child(label().text("Storage").font_size(18.)).child(Store::default_root().display().to_string())
                    .child("Instance removal moves files to Trash. Nothing is automatically deleted."))
                .child(panel(dark).width(Size::fill()).child(label().text("Accounts & updates").font_size(18.))
                    .child("Not signed in. Microsoft authentication and game launch are coming in a future release.")
                    .child("Updates are manual: github.com/karmangamerz-coder/Aero-client.").child("No telemetry, Aero account, advertisements or paid features."))
                .child(primary("Save settings").enabled(!s.busy).on_press(move |_| save.send(Command::Save(Settings {
                    appearance: appearance.read().clone(),memory_mib: memory(),custom_java: java.read().clone(),low_performance: low(),reduced_motion: reduced(),show_news: show_news(),
                }))))).into_element()
        }
    };
        rect()
            .expanded()
            .color(if dark { (234, 248, 255) } else { (18, 60, 86) })
            .font_size(14.)
            .background(if dark { (12, 37, 56) } else { (141, 216, 248) })
            .maybe(*appearance.read() != "Aero Lite", |r| {
                r.child(
                    SvgViewer::new(include_bytes!("../assets/landscape.svg"))
                        .position(Position::new_absolute())
                        .width(Size::fill())
                        .height(Size::fill())
                        .aspect_ratio(AspectRatio::Max),
                )
            })
            .child(
                rect()
                    .expanded()
                    .horizontal()
                    .content(Content::Flex)
                    .child(rail)
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .height(Size::fill())
                            .content(Content::Flex)
                            .child(header)
                            .child(
                                rect()
                                    .width(Size::fill())
                                    .height(Size::flex(1.))
                                    .child(content),
                            )
                            .child(
                                rect()
                                    .width(Size::fill())
                                    .height(Size::px(35.))
                                    .padding((8., 18.))
                                    .background(glass(dark))
                                    .child(label().text(s.message).font_size(11.)),
                            ),
                    ),
            )
    }
}
fn parse_loader(value: &str) -> Loader {
    match value {
        "Fabric" => Loader::Fabric,
        "Forge" => Loader::Forge,
        _ => Loader::Vanilla,
    }
}
