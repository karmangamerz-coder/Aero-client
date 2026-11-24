// SPDX-License-Identifier: GPL-3.0-only
//! Page composition. No hooks here: all reactive state is owned by the app root and
//! passed in, so pages stay deterministic and event driven.
use crate::{
    backend::{Backend, Command, Snapshot},
    design::*,
    ui::Ui,
};
use aero_core::{Instance, Loader, Performance, TARGETS, onboarding};
use freya::prelude::*;

pub const WIZARD: usize = 5;

pub fn page(index: usize, ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    match index {
        0 => play(ui, s, p, b),
        1 if !ui.detail.read().is_empty() => instance_detail(ui, s, p, b),
        1 => instances(ui, s, p, b),
        2 => discover(ui, s, p, b),
        3 => downloads(ui, s, p, b),
        4 => settings(ui, s, p, b),
        _ => wizard(ui, s, p, b),
    }
}

fn chosen<'a>(s: &'a Snapshot) -> Option<&'a Instance> {
    let id = s.selected.clone();
    s.instances
        .iter()
        .find(|i| i.id == id)
        .or_else(|| s.instances.first())
}

fn scroll() -> ScrollView {
    ScrollView::new().expanded()
}

fn body() -> Rect {
    rect().width(Size::fill()).padding((18., 20.)).spacing(14.)
}

fn hint(p: Palette, message: impl Into<String>) -> Rect {
    rect()
        .width(Size::fill())
        .padding((10., 12.))
        .corner_radius(6.)
        .background(p.surface_alt)
        .border(Border::new().width(1.).fill(p.border_soft))
        .child(dim(message.into(), 11.5, p))
}

fn kv(p: Palette, key: &str, value: String) -> Rect {
    rect()
        .horizontal()
        .width(Size::fill())
        .spacing(10.)
        .child(
            rect()
                .width(Size::px(140.))
                .child(dim(key.to_owned(), 11.5, p)),
        )
        .child(text(value, 11.5, p.text))
}

fn parse_loader(value: &str) -> Loader {
    match value {
        "Fabric" => Loader::Fabric,
        "Forge" => Loader::Forge,
        _ => Loader::Vanilla,
    }
}

// --------------------------------------------------------------------- play

pub fn play(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let instance = chosen(s).cloned();
    let account = s.accounts.active_account().cloned();
    let page = ui.page;
    let detail = ui.detail;
    let username = account
        .as_ref()
        .map(|a| a.name.clone())
        .unwrap_or_else(|| "Not signed in".to_owned());
    let status = if account.is_some() {
        "Online"
    } else {
        "Signed out"
    };
    let percent = if s.progress.total > 0 {
        s.progress.bytes as f32 / s.progress.total as f32 * 100.
    } else if s.progress.files_total > 0 {
        s.progress.files_done as f32 / s.progress.files_total as f32 * 100.
    } else {
        0.
    };
    let installing = s.busy && percent > 0.;

    let ready = instance.as_ref().is_some_and(|i| i.status == "READY");
    let launch_action: Element = if s.launching {
        rect()
            .width(Size::px(270.))
            .height(Size::px(54.))
            .corner_radius(10.)
            .background(p.surface_alt)
            .border(Border::new().width(1.).fill(p.green_border))
            .center()
            .child(strong("Launching Minecraft…", 15., p.text))
            .into_element()
    } else if installing {
        launch_progress(p, percent, "Installing").into_element()
    } else if account.is_none() {
        launch_button(p, "Sign in to play", true)
            .on_press({
                let backend = b.clone();
                move |_| backend.send(Command::SignIn)
            })
            .into_element()
    } else if instance.is_none() {
        launch_button(p, "Create an instance", true)
            .on_press({
                let mut page = page;
                move |_| page.set(1)
            })
            .into_element()
    } else if !ready {
        launch_button(p, "Install game files", !s.busy)
            .on_press({
                let backend = b.clone();
                let id = instance.as_ref().map(|i| i.id.clone()).unwrap_or_default();
                move |_| backend.send(Command::Install(id.clone()))
            })
            .into_element()
    } else {
        launch_button(p, "Launch", !s.busy)
            .on_press({
                let backend = b.clone();
                let id = instance.as_ref().map(|i| i.id.clone()).unwrap_or_default();
                move |_| backend.send(Command::Launch(id.clone()))
            })
            .into_element()
    };

    let instance_label = instance
        .as_ref()
        .map(|i| i.name.clone())
        .unwrap_or_else(|| "No instance".to_owned());
    let version_label = instance
        .as_ref()
        .map(|i| i.version.clone())
        .unwrap_or_else(|| "1.21.11".to_owned());
    let loader_label = instance
        .as_ref()
        .map(|i| i.loader.to_string())
        .unwrap_or_else(|| "Vanilla".to_owned());

    let mut instance_items: Vec<MenuItem> = vec![];
    for item in &s.instances {
        let id = item.id.clone();
        let backend = b.clone();
        instance_items.push(
            MenuItem::new()
                .selected(instance.as_ref().is_some_and(|i| i.id == id))
                .on_press(move |_| backend.send(Command::Select(id.clone())))
                .child(item.name.clone()),
        );
    }
    if instance_items.is_empty() {
        instance_items.push(MenuItem::new().child("No instances yet"));
    }
    let version_items: Vec<MenuItem> = TARGETS
        .iter()
        .map(|version| {
            let picked = (*version).to_owned();
            let loader = instance
                .as_ref()
                .map(|i| i.loader)
                .unwrap_or(Loader::Vanilla);
            let backend = b.clone();
            MenuItem::new()
                .selected(&version_label == *version)
                .on_press(move |_| backend.send(Command::Target(picked.clone(), loader)))
                .child((*version).to_owned())
        })
        .collect();
    let loader_items: Vec<MenuItem> = [Loader::Vanilla, Loader::Fabric, Loader::Forge]
        .into_iter()
        .map(|loader| {
            let version = instance
                .as_ref()
                .map(|i| i.version.clone())
                .unwrap_or_else(|| "1.21.11".to_owned());
            let backend = b.clone();
            let label = loader.to_string();
            MenuItem::new()
                .selected(loader_label == label)
                .on_press(move |_| backend.send(Command::Target(version.clone(), loader)))
                .child(label)
        })
        .collect();

    let selectors = rect()
        .horizontal()
        .spacing(8.)
        .child(
            rect().width(Size::px(186.)).height(Size::px(38.)).child(
                Select::new()
                    .selected_item(instance_label)
                    .children(instance_items),
            ),
        )
        .child(
            rect().width(Size::px(160.)).height(Size::px(38.)).child(
                Select::new()
                    .selected_item(version_label)
                    .children(version_items),
            ),
        )
        .child(
            rect().width(Size::px(132.)).height(Size::px(38.)).child(
                Select::new()
                    .selected_item(loader_label)
                    .children(loader_items),
            ),
        );

    rect()
        .expanded()
        .padding((0., 0.))
        .child(
            rect()
                .expanded()
                .padding((18., 26., 26., 18.))
                .content(Content::Flex)
                .cross_align(Alignment::Center)
                .spacing(4.)
                .child(
                    rect()
                        .horizontal()
                        .spacing(8.)
                        .center()
                        .child(art(username, 24., p)),
                )
                .child(
                    rect()
                        .horizontal()
                        .spacing(6.)
                        .center()
                        .child(
                            rect()
                                .width(Size::px(7.))
                                .height(Size::px(7.))
                                .corner_radius(4.)
                                .background(if account.is_some() {
                                    p.green
                                } else {
                                    p.text_dim
                                }),
                        )
                        .child(art_dim(status, 11.5, p)),
                )
                .child(rect().expanded().center().child(character(&s.skin)))
                .child(selectors)
                .child(rect().horizontal().spacing(10.).child(launch_action).child(
                    TooltipContainer::new(Tooltip::new_text("Instance settings")).child(
                        gear_button(p).on_press({
                            let backend = b.clone();
                            let mut page = page;
                            let mut detail = detail;
                            let id = instance.as_ref().map(|i| i.id.clone());
                            move |_| {
                                if let Some(id) = id.clone() {
                                    backend.send(Command::LoadPlan(id.clone()));
                                    detail.set(id);
                                    page.set(1);
                                } else {
                                    page.set(1);
                                }
                            }
                        }),
                    ),
                ))
                .child(rect().height(Size::px(4.)).child(art_dim(
                    if s.launching {
                        "Live logs are on the Downloads page."
                    } else if instance.is_some() && !ready {
                        "Official files are downloaded, verified and repairable."
                    } else if account.is_none() {
                        "Sign in with the Microsoft account that owns Minecraft."
                    } else {
                        ""
                    },
                    10.5,
                    p,
                ))),
        )
        .into_element()
}

// ---------------------------------------------------------------- instances

pub fn instances(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let create = b.clone();
    let name = ui.name;
    let mut version = ui.version;
    let mut loader = ui.loader;
    let open = b.clone();
    let detail = ui.detail;
    let query = ui.search.read().to_lowercase();

    let form = panel(p)
        .width(Size::fill())
        .child(section(
            p,
            "New instance",
            "Each instance owns its saves, mods and settings. Verified game files are shared.",
        ))
        .child(
            rect()
                .horizontal()
                .width(Size::fill())
                .spacing(8.)
                .child(
                    rect()
                        .width(Size::px(230.))
                        .child(Input::new(name).placeholder("Instance name")),
                )
                .child(
                    rect().width(Size::px(120.)).child(
                        Select::new()
                            .selected_item(version.read().clone())
                            .children(TARGETS.iter().map(|target| {
                                let value = (*target).to_owned();
                                let picked = value.clone();
                                MenuItem::new()
                                    .selected(*version.read() == value)
                                    .on_press(move |_| version.set(picked.clone()))
                                    .child(value)
                            })),
                    ),
                )
                .child(rect().width(Size::px(110.)).child(
                    Select::new().selected_item(loader.read().clone()).children(
                        ["Vanilla", "Fabric", "Forge"].into_iter().map(|value| {
                            let label = value.to_owned();
                            let picked = label.clone();
                            MenuItem::new()
                                .selected(*loader.read() == label)
                                .on_press(move |_| loader.set(picked.clone()))
                                .child(label)
                        }),
                    ),
                ))
                .child(primary_create(p, !s.busy).on_press({
                    let create = create.clone();
                    move |_| {
                        create.send(Command::Create(
                            name.read().clone(),
                            version.read().clone(),
                            parse_loader(&loader.read()),
                        ))
                    }
                }))
                .child(action("Check versions", p).enabled(!s.busy).on_press({
                    let create = create.clone();
                    move |_| create.send(Command::RefreshVersions)
                })),
        )
        .child(dim(
            format!(
                "Official metadata: {}",
                if s.versions.is_empty() {
                    "not checked in this session".to_owned()
                } else {
                    s.versions.join(", ")
                }
            ),
            10.5,
            p,
        ));

    let rows: Vec<Rect> = s
        .instances
        .iter()
        .filter(|i| {
            format!("{} {} {}", i.name, i.version, i.loader)
                .to_lowercase()
                .contains(&query)
        })
        .map(|instance| {
            let id = instance.id.clone();
            let manage = open.clone();
            let mut detail = detail;
            card(p).width(Size::fill()).child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .spacing(12.)
                    .cross_align(Alignment::Center)
                    .child(
                        rect()
                            .width(Size::px(74.))
                            .height(Size::px(46.))
                            .corner_radius(6.)
                            .background(p.surface_alt)
                            .center()
                            .child(strong(instance.version.clone(), 13., p.text)),
                    )
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .spacing(3.)
                            .child(strong(instance.name.clone(), 14.5, p.text))
                            .child(dim(
                                format!(
                                    "{} · {} · {}",
                                    instance.version, instance.loader, instance.detail
                                ),
                                11.,
                                p,
                            )),
                    )
                    .child(status_chip(&instance.status, p))
                    .child(action("Manage", p).on_press(move |_| {
                        manage.send(Command::LoadPlan(id.clone()));
                        detail.set(id.clone());
                    })),
            )
        })
        .collect();

    let mut list = rect().width(Size::fill()).spacing(8.);
    if rows.is_empty() {
        list = list.child(hint(
            p,
            "No instances yet. Create one above to get started.",
        ));
    }
    list = list.child(rect().width(Size::fill()).spacing(8.).children(rows));

    scroll()
        .child(body().child(form).child(list))
        .into_element()
}

fn primary_create(p: Palette, enabled: bool) -> Button {
    Button::new()
        .theme_layout(
            ButtonLayoutThemePartial::default()
                .corner_radius(6.)
                .padding((0., 18.))
                .height(Size::px(34.)),
        )
        .theme_colors(
            ButtonColorsThemePartial::default()
                .color(Color::WHITE)
                .background(p.green)
                .hover_background(p.green_hover)
                .border_fill(p.green_border)
                .focus_border_fill(p.accent),
        )
        .enabled(enabled)
        .child(strong("Create", 12.5, Color::WHITE))
}

pub fn instance_detail(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let id = ui.detail.read().clone();
    let Some(instance) = s.instances.iter().find(|i| i.id == id).cloned() else {
        return hint(p, "That instance no longer exists.").into_element();
    };
    let mut detail = ui.detail;
    let mut page = ui.page;
    let rename = ui.rename;
    let inst_memory = ui.inst_memory;
    let inst_java = ui.inst_java;
    let confirm = ui.confirm;
    let backend = b.clone();
    let install_backend = b.clone();
    let launch_backend = b.clone();
    let account = s.accounts.active_account().cloned();

    let memory_value = inst_memory.read().clone();
    let java_value = inst_java.read().clone();
    let ready = instance.status == "READY";

    let installation = panel(p)
        .width(Size::flex(1.))
        .child(section(
            p,
            "Installation",
            "Verified official files, libraries, assets and the managed Java runtime.",
        ))
        .child(kv(
            p,
            "State",
            format!("{} · {}", instance.status, instance.detail),
        ))
        .child(kv(
            p,
            "Version",
            format!("{} · {}", instance.version, instance.loader),
        ))
        .child(kv(
            p,
            "Memory",
            format!(
                "{} MiB",
                instance.memory_mib.unwrap_or(s.settings.memory_mib)
            ),
        ))
        .child(kv(
            p,
            "Java",
            if instance.java_path.is_empty() {
                "Managed automatically".into()
            } else {
                instance.java_path.clone()
            },
        ))
        .child(
            rect()
                .horizontal()
                .spacing(8.)
                .child(
                    launch_button_small(
                        p,
                        if ready { "Launch" } else { "Install · Repair" },
                        !s.busy,
                    )
                    .on_press({
                        let backend = install_backend.clone();
                        let launch_backend = launch_backend.clone();
                        let id = instance.id.clone();
                        let ready = ready;
                        let signed_in = account.is_some();
                        move |_| {
                            if ready && signed_in {
                                launch_backend.send(Command::Launch(id.clone()));
                            } else {
                                backend.send(Command::Install(id.clone()));
                            }
                        }
                    }),
                )
                .child(action("Cancel", p).enabled(s.busy).on_press({
                    let backend = b.clone();
                    move |_| backend.cancel()
                })),
        );

    let advanced = panel(p)
        .width(Size::flex(1.))
        .child(section(
            p,
            "Advanced",
            "Overrides stay in this instance's manifest only.",
        ))
        .child(
            rect()
                .horizontal()
                .spacing(8.)
                .cross_align(Alignment::Center)
                .child(dim("Memory MiB", 11.5, p))
                .child(
                    rect()
                        .width(Size::px(120.))
                        .child(Input::new(inst_memory).placeholder("4096")),
                )
                .child(dim("Java path", 11.5, p))
                .child(
                    rect()
                        .width(Size::px(260.))
                        .child(Input::new(inst_java).placeholder("Automatic")),
                )
                .child(action("Save", p).enabled(!s.busy).on_press({
                    let backend = backend.clone();
                    let id = instance.id.clone();
                    move |_| {
                        let memory = memory_value.trim().parse::<u32>().ok();
                        backend.send(Command::Configure(id.clone(), memory, java_value.clone()));
                    }
                })),
        )
        .child(
            rect()
                .horizontal()
                .spacing(8.)
                .cross_align(Alignment::Center)
                .child(
                    rect()
                        .width(Size::px(230.))
                        .child(Input::new(rename).placeholder("New name")),
                )
                .child(action("Rename", p).on_press({
                    let backend = backend.clone();
                    let id = instance.id.clone();
                    move |_| backend.send(Command::Rename(id.clone(), rename.read().clone()))
                }))
                .child(action("Duplicate", p).on_press({
                    let backend = backend.clone();
                    let id = instance.id.clone();
                    move |_| backend.send(Command::Duplicate(id.clone()))
                }))
                .child(danger("Remove", p).on_press({
                    let backend = backend.clone();
                    let id = instance.id.clone();
                    move |_| backend.send(Command::Remove(id.clone()))
                }))
                .child(dim(confirm.read().clone(), 10.5, p)),
        );

    let plan = s.plan.as_ref().map(|plan| {
        card(p).width(Size::fill()).child(dim(
            format!(
                "Verified plan · client {} · assets {} · {} libraries",
                plan["clientSha1"]
                    .as_str()
                    .unwrap_or("?")
                    .chars()
                    .take(8)
                    .collect::<String>(),
                plan["assetIndexId"].as_str().unwrap_or("?"),
                plan["libraries"].as_array().map(|l| l.len()).unwrap_or(0)
            ),
            11.,
            p,
        ))
    });

    scroll()
        .child(
            body()
                .child(
                    rect()
                        .horizontal()
                        .spacing(10.)
                        .cross_align(Alignment::Center)
                        .child(action("Back", p).on_press(move |_| detail.set(String::new())))
                        .child(strong(instance.name.clone(), 20., p.text))
                        .child(status_chip(&instance.status, p))
                        .child(rect().width(Size::flex(1.)))
                        .child(ghost("Downloads", p).on_press(move |_| page.set(3))),
                )
                .child(
                    rect()
                        .horizontal()
                        .spacing(12.)
                        .children([installation, advanced]),
                )
                .maybe(plan.is_some(), |r| match plan {
                    Some(plan) => r.child(plan),
                    None => r,
                }),
        )
        .into_element()
}

fn launch_button_small(p: Palette, label: &str, enabled: bool) -> Button {
    Button::new()
        .theme_layout(
            ButtonLayoutThemePartial::default()
                .corner_radius(7.)
                .padding((0., 20.))
                .height(Size::px(38.)),
        )
        .theme_colors(
            ButtonColorsThemePartial::default()
                .color(Color::WHITE)
                .background(p.green)
                .hover_background(p.green_hover)
                .border_fill(p.green_border)
                .focus_border_fill(p.accent),
        )
        .enabled(enabled)
        .child(strong(label.to_owned(), 13.5, Color::WHITE))
}

// ---------------------------------------------------------------- discover

pub fn discover(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let query = ui.query;
    let mut kind = ui.kind;
    let backend = b.clone();
    let current_kind = kind.read().clone();
    let selected = s.selected.clone();
    let results: Vec<Rect> = s
        .results
        .iter()
        .map(|item| {
            let backend = b.clone();
            let instance = selected.clone();
            let project = item.slug.clone();
            card(p)
                .width(Size::fill())
                .child(strong(item.title.clone(), 13.5, p.text))
                .child(dim(item.description.clone(), 11., p))
                .child(dim(
                    format!("{} · {}", item.project_type, item.slug),
                    10.,
                    p,
                ))
                .child(
                    action("Install", p)
                        .enabled(!s.busy && !instance.is_empty() && item.project_type != "modpack")
                        .on_press(move |_| backend.send(Command::InstallContent(instance.clone(), project.clone()))),
                )
        })
        .collect();
    scroll()
        .child(
            body()
                .child(section(
                    p,
                    "Discover",
                    "Search Modrinth and install verified compatible mods, resource packs and shaders into the selected instance.",
                ))
                .child(
                    rect().horizontal().spacing(8.).cross_align(Alignment::Center)
                        .child(rect().width(Size::px(320.)).child(Input::new(query).placeholder("Search Modrinth")))
                        .child(rect().width(Size::px(130.)).child(
                            Select::new().selected_item(current_kind.clone()).children(
                                ["mod", "modpack", "resourcepack", "shader"].into_iter().map(|value| {
                                    let picked = value.to_owned();
                                    MenuItem::new()
                                        .selected(current_kind == value)
                                        .on_press(move |_| kind.set(picked.clone()))
                                        .child(value.to_owned())
                                }),
                            ),
                        ))
                        .child(action("Search", p).enabled(!s.busy).on_press({
                            let backend = backend.clone();
                            move |_| {
                                backend.send(Command::Discover(
                                    query.read().clone(),
                                    kind.read().clone(),
                                    String::new(),
                                    String::new(),
                                ))
                            }
                        })),
                )
                .child(rect().width(Size::fill()).spacing(8.).children(results))
                .maybe(s.results.is_empty(), |r| r.child(hint(p, "Search results will appear here."))),
        )
        .into_element()
}

// ---------------------------------------------------------------- downloads

pub fn downloads(_ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let percent = if s.progress.total > 0 {
        s.progress.bytes as f32 / s.progress.total as f32 * 100.
    } else if s.progress.files_total > 0 {
        s.progress.files_done as f32 / s.progress.files_total as f32 * 100.
    } else {
        0.
    };
    let mut controls = rect().horizontal().spacing(8.);
    controls = controls
        .child(action("Cancel download", p).enabled(s.busy).on_press({
            let backend = b.clone();
            move |_| backend.cancel()
        }))
        .child(action("Stop game", p).enabled(s.launching).on_press({
            let backend = b.clone();
            move |_| backend.cancel_launch()
        }));
    let game_logs: Vec<Rect> = s
        .game_logs
        .iter()
        .map(|line| rect().child(text(line.clone(), 10.5, p.text)))
        .collect();
    let session: Vec<Rect> = s
        .log
        .iter()
        .map(|line| rect().child(dim(line.clone(), 10.5, p)))
        .collect();
    scroll()
        .child(
            body()
                .child(section(
                    p,
                    "Downloads",
                    "Verified files, managed runtimes and live game output.",
                ))
                .child(
                    card(p)
                        .width(Size::fill())
                        .spacing(6.)
                        .child(kv(
                            p,
                            "Activity",
                            if s.progress.item.is_empty() {
                                "Idle".into()
                            } else {
                                s.progress.item.clone()
                            },
                        ))
                        .child(progress(percent, p))
                        .child(dim(format!("{percent:.0}%"), 10.5, p))
                        .child(controls),
                )
                .child(
                    card(p)
                        .width(Size::fill())
                        .child(strong("Session", 13.5, p.text))
                        .child(
                            ScrollView::new()
                                .height(Size::px(120.))
                                .child(rect().spacing(1.).children(session)),
                        ),
                )
                .child(
                    card(p)
                        .width(Size::fill())
                        .child(strong("Minecraft logs", 13.5, p.text))
                        .child(
                            ScrollView::new()
                                .height(Size::px(260.))
                                .child(rect().spacing(1.).children(game_logs)),
                        )
                        .maybe(s.game_logs.is_empty(), |r| {
                            r.child(dim("The game writes here while it runs.", 10.5, p))
                        }),
                ),
        )
        .into_element()
}

// ---------------------------------------------------------------- settings

pub fn settings(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let save = b.clone();
    let mut appearance = ui.appearance;
    let memory = ui.memory;
    let java = ui.java;
    let low = ui.low;
    let reduced = ui.reduced;
    let current = appearance.read().clone();

    let theme_row = rect().horizontal().spacing(8.).children(
        ["Aero Light", "Aero Dark", "Aero Lite"]
            .into_iter()
            .map(|name| {
                let label = name.to_owned();
                let picked = label.clone();
                action(label, p)
                    .enabled(current != name)
                    .on_press(move |_| appearance.set(picked.clone()))
            }),
    );

    let hardware_text = s
        .hardware
        .as_ref()
        .map(|h| format!("{}\nGPU: {}\n{}", h.summary(), h.gpu, h.os))
        .unwrap_or_else(|| "Detecting hardware…".into());

    let accounts: Vec<Rect> = s
        .accounts
        .accounts
        .iter()
        .map(|account| {
            let id = account.uuid.clone();
            let remove = b.clone();
            rect()
                .horizontal()
                .width(Size::fill())
                .spacing(10.)
                .cross_align(Alignment::Center)
                .child(strong(account.name.clone(), 12.5, p.text))
                .child(dim(
                    format!(
                        "Owns Minecraft · {}",
                        &account.uuid[..8.min(account.uuid.len())]
                    ),
                    10.5,
                    p,
                ))
                .child(rect().width(Size::flex(1.)))
                .child(
                    ghost("Remove", p)
                        .on_press(move |_| remove.send(Command::RemoveAccount(id.clone()))),
                )
        })
        .collect();

    let memory_value: u32 = memory.read().parse().unwrap_or(s.settings.memory_mib);
    scroll()
        .child(
            body()
                .child(section(p, "Settings", "Appearance, performance and accounts. Nothing leaves this device without an action you take."))
                .child(
                    panel(p).width(Size::fill())
                        .child(strong("Appearance", 14., p.text))
                        .child(theme_row)
                        .child(dim("Aero Lite disables the landscape artwork and uses flat surfaces on low-end hardware.", 10.5, p)),
                )
                .child(
                    panel(p).width(Size::fill())
                        .child(strong("Performance", 14., p.text))
                        .child(
                            rect().horizontal().spacing(8.).cross_align(Alignment::Center)
                                .child(dim("Game memory (MiB)", 11.5, p))
                                .child(rect().width(Size::px(110.)).child(Input::new(memory).placeholder("4096")))
                                .child(dim("Custom Java", 11.5, p))
                                .child(rect().width(Size::px(300.)).child(Input::new(java).placeholder("Managed automatically"))),
                        )
                        .child(
                            rect().horizontal().spacing(16.).cross_align(Alignment::Center)
                                .child(dim("Low-performance mode", 11.5, p))
                                .child(Switch::new(low))
                                .child(dim("Reduce motion", 11.5, p))
                                .child(Switch::new(reduced)),
                        )
                        .child(hardware_text.clone())
                        .child(action("Save preferences", p).enabled(!s.busy).on_press({
                            let save = save.clone();
                            let base = s.settings.clone();
                            move |_| {
                                let mut settings = base.clone();
                                settings.appearance = appearance.read().clone();
                                settings.memory_mib = memory_value.clamp(1024, 32768);
                                settings.custom_java = java.read().clone();
                                settings.low_performance = *low.read();
                                settings.reduced_motion = *reduced.read();
                                save.send(Command::Save(settings.clone()));
                            }
                        })),
                )
                .child(
                    panel(p).width(Size::fill())
                        .child(strong("Accounts", 14., p.text))
                        .child(rect().width(Size::fill()).spacing(6.).children(accounts))
                        .child(rect().horizontal().spacing(8.)
                            .child(action("Add account", p).on_press({
                                let backend = b.clone();
                                move |_| backend.send(Command::SignIn)
                            }))
                            .child(action("Sign out", p).on_press({
                                let backend = b.clone();
                                move |_| backend.send(Command::SignOut)
                            }))),
                )
                .child(
                    panel(p).width(Size::fill())
                        .child(strong("Setup and storage", 14., p.text))
                        .child(kv(p, "Data folder", s.storage.clone()))
                        .child(rect().horizontal().spacing(8.)
                            .child(action("Run setup again", p).on_press({
                                let backend = b.clone();
                                move |_| backend.send(Command::ReopenSetup)
                            }))
                            .child(action("Detect hardware", p).enabled(!s.busy).on_press({
                                let backend = b.clone();
                                move |_| backend.send(Command::DetectHardware)
                            }))),
                ),
        )
        .into_element()
}

// ------------------------------------------------------------------ wizard

fn step_dots(current: usize, p: Palette) -> Rect {
    let mut row = rect().horizontal().spacing(6.);
    for index in 0..onboarding::STEPS {
        row = row.child(
            rect()
                .width(Size::px(if index == current { 22. } else { 8. }))
                .height(Size::px(8.))
                .corner_radius(4.)
                .background(if index <= current {
                    p.green
                } else {
                    p.border_soft
                }),
        );
    }
    row
}

fn summary_row(p: Palette, key: &str, value: String) -> Rect {
    rect()
        .horizontal()
        .width(Size::fill())
        .child(
            rect()
                .width(Size::px(150.))
                .child(dim(key.to_owned(), 11.5, p)),
        )
        .child(text(value, 11.5, p.text))
}

fn wizard_preview(name: &str, selected: bool) -> Rect {
    let p = palette(name);
    let mut preview = rect()
        .width(Size::px(150.))
        .height(Size::px(96.))
        .corner_radius(8.)
        .background(if p.photo {
            Color::from_rgb(120, 200, 240)
        } else {
            p.bg
        })
        .border(
            Border::new()
                .width(if selected { 2. } else { 1. })
                .fill(if selected {
                    palette("Aero Light").green
                } else {
                    p.border
                }),
        )
        .padding((6., 6.))
        .spacing(4.);
    if p.photo {
        preview = preview.child(
            SvgViewer::new(include_bytes!("../assets/landscape.svg"))
                .position(Position::new_absolute())
                .width(Size::fill())
                .height(Size::fill())
                .aspect_ratio(AspectRatio::Max),
        );
    }
    preview = preview
        .child(
            rect()
                .width(Size::px(56.))
                .height(Size::px(6.))
                .corner_radius(3.)
                .background(p.text_dim),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(28.))
                .corner_radius(5.)
                .background(p.glass),
        )
        .child(
            rect()
                .horizontal()
                .spacing(4.)
                .child(
                    rect()
                        .width(Size::px(40.))
                        .height(Size::px(14.))
                        .corner_radius(4.)
                        .background(p.green),
                )
                .child(
                    rect()
                        .width(Size::px(14.))
                        .height(Size::px(14.))
                        .corner_radius(4.)
                        .background(p.glass),
                ),
        );
    preview.child(rect().horizontal().width(Size::fill()).center().child(text(
        name.to_owned(),
        10.5,
        p.text,
    )))
}

pub fn wizard(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let step = s.onboarding.current_step();
    let mut version = ui.version;
    let mut loader = ui.loader;
    let mut w_memory = ui.memory;
    let memory_value: u32 = w_memory.read().parse().unwrap_or(4096);
    let mut w_appearance = ui.appearance;
    let mut w_perf = ui.w_perf;
    let w_auto = ui.w_auto;
    let backend = b.clone();

    // Wizard answers are collected from the state values so every step persists
    // exactly what the player has selected, including a resumed session.
    let base = onboarding::Onboarding {
        schema: 1,
        version: version.read().clone(),
        loader: loader.read().clone(),
        memory_mib: memory_value,
        appearance: w_appearance.read().clone(),
        performance: w_perf.read().clone(),
        installation_directory: s.storage.clone(),
        auto_java: *w_auto.read(),
    };
    let mut content = rect().width(Size::fill()).spacing(10.);
    match step {
        0 => {
            content = content
                .child(rect().width(Size::fill()).center().child(logo(64.)))
                .child(strong("Welcome to Aero Client", 22., p.text))
                .child(dim(
                    "A free, open-source Minecraft launcher built on a native Rust interface. No accounts with Aero, no advertising, no telemetry — ever.",
                    12.,
                    p,
                ))
                .child(dim(
                    "Aero installs official game files, verifies every download and manages Java for you.",
                    11.5,
                    p,
                ));
        }
        1 => {
            content = content
                .child(strong("Microsoft account", 20., p.text))
                .child(dim(
                    "Sign in with the Microsoft account that owns Minecraft: Java Edition. Aero uses the official browser sign-in, so your password is never entered or stored here.",
                    12.,
                    p,
                ))
                .child(dim(
                    "Ownership is verified with Microsoft. Aero cannot and will not bypass it. You can skip this step and browse the launcher without signing in.",
                    11.5,
                    p,
                ))
                .child(action("Sign in with Microsoft", p).on_press({
                    let backend = backend.clone();
                    move |_| backend.send(Command::SignIn)
                }))
                .child(dim(
                    if s.signin_state.is_empty() {
                        s.accounts
                            .active_account()
                            .map(|a| format!("Signed in as {}.", a.name))
                            .unwrap_or_else(|| "Not signed in.".into())
                    } else {
                        s.signin_state.clone()
                    },
                    11.,
                    p,
                ));
        }
        2 => {
            let hardware = s.hardware.clone();
            let mut details = rect().width(Size::fill()).spacing(4.);
            if let Some(hardware) = &hardware {
                details = details
                    .child(summary_row(p, "CPU", hardware.cpu.clone()))
                    .child(summary_row(p, "Cores", hardware.cores.to_string()))
                    .child(summary_row(
                        p,
                        "Memory",
                        format!("{} GB", hardware.memory_mib / 1024),
                    ))
                    .child(summary_row(
                        p,
                        "Storage",
                        format!("{} GB free", hardware.storage_free_gib),
                    ))
                    .child(summary_row(p, "GPU", hardware.gpu.clone()))
                    .child(summary_row(p, "Operating system", hardware.os.clone()))
                    .child(summary_row(
                        p,
                        "Recommended",
                        format!(
                            "{} MiB · {}",
                            hardware.recommended_memory_mib(),
                            hardware.recommended_performance().label()
                        ),
                    ));
            } else {
                details = details.child(dim("Detecting hardware…", 11.5, p));
            }
            content = content
                .child(strong("Hardware", 20., p.text))
                .child(dim(
                    "Aero only reads this information to suggest settings. Nothing is changed on your computer without your permission.",
                    12.,
                    p,
                ))
                .child(details)
                .child(
                    rect().horizontal().spacing(8.)
                        .child(action("Use recommended", p).on_press({
                            let recommendation = hardware.as_ref().map(|h| (h.recommended_memory_mib(), h.recommended_performance()));
                            move |_| {
                                if let Some((memory, performance)) = recommendation {
                                    w_memory.set(memory.to_string());
                                    w_perf.set(performance.label().to_owned());
                                }
                            }
                        }))
                        .child(action("Detect again", p).on_press({
                            let backend = backend.clone();
                            move |_| backend.send(Command::DetectHardware)
                        })),
                );
        }
        3 => {
            let current = w_appearance.read().clone();
            content = content
                .child(strong("Appearance", 20., p.text))
                .child(dim(
                    "Choose a look now; you can change it any time in Settings.",
                    12.,
                    p,
                ))
                .child(
                    rect().horizontal().spacing(10.).children(
                        ["Aero Light", "Aero Dark", "Aero Lite"]
                            .into_iter()
                            .map(|name| {
                                let picked = name.to_owned();
                                let active = current == name;
                                Button::new()
                                    .theme_layout(
                                        ButtonLayoutThemePartial::default()
                                            .corner_radius(9.)
                                            .padding(0.)
                                            .width(Size::px(160.))
                                            .height(Size::px(112.)),
                                    )
                                    .theme_colors(
                                        ButtonColorsThemePartial::default()
                                            .background(Color::TRANSPARENT)
                                            .hover_background(Color::TRANSPARENT)
                                            .border_fill(if active {
                                                p.green
                                            } else {
                                                p.border_soft
                                            })
                                            .focus_border_fill(p.accent),
                                    )
                                    .on_press(move |_| w_appearance.set(picked.clone()))
                                    .child(wizard_preview(name, active))
                            })
                            .collect::<Vec<_>>(),
                    ),
                );
        }
        4 => {
            content = content
                .child(strong("Minecraft", 20., p.text))
                .child(dim("Pick the version and loader for your Aero Default instance. Only versions in official metadata are offered.", 12., p))
                .child(
                    rect().horizontal().spacing(8.).cross_align(Alignment::Center)
                        .child(dim("Version", 11.5, p))
                        .child(rect().width(Size::px(130.)).child(
                            Select::new().selected_item(version.read().clone()).children(TARGETS.iter().map(|value| {
                                let label = (*value).to_owned();
                                let picked = label.clone();
                                let mut selected_loader = loader;
                                MenuItem::new()
                                    .selected(*version.read() == label)
                                    .on_press(move |_| {
                                        let current_loader = selected_loader.read().clone();
                                        if !onboarding::compatible(&picked, &current_loader) {
                                            selected_loader.set("Vanilla".into());
                                        }
                                        version.set(picked.clone());
                                    })
                                    .child(label)
                            })),
                        ))
                        .child(dim("Loader", 11.5, p))
                        .child(rect().width(Size::px(130.)).child(
                            Select::new().selected_item(loader.read().clone()).children(["Vanilla", "Fabric", "Forge"].into_iter().map(|value| {
                                let label = value.to_owned();
                                let picked = label.clone();
                                MenuItem::new()
                                    .selected(*loader.read() == label)
                                    .on_press(move |_| {
                                        if let Err(_) = validate_loader(&picked, &version.read()) {
                                            return;
                                        }
                                        loader.set(picked.clone())
                                    })
                                    .child(label)
                            })),
                        )),
                )
                .child(summary_row(p, "Installs into", s.storage.clone()))
                .child(dim(
                    "Aero manages the matching Java runtime automatically. External game folders remain disabled in this beta to protect unrelated files.",
                    10.5,
                    p,
                ))
                .child(action("Check official availability", p).enabled(!s.busy).on_press({
                    let backend = backend.clone();
                    move |_| backend.send(Command::RefreshVersions)
                }))
                .maybe(!s.versions.is_empty(), |r| {
                    r.child(dim(
                        format!("Available now: {}", s.versions.join(", ")),
                        10.5,
                        p,
                    ))
                })
                .child(
                    rect().horizontal().spacing(8.).cross_align(Alignment::Center)
                        .child(dim("Automatic Java runtime", 11.5, p))
                        .child(Switch::new(w_auto)),
                );
        }
        5 => {
            let current = w_perf.read().clone();
            content = content
                .child(strong("Performance", 20., p.text))
                .child(dim("Presets set memory and Aero rendering behaviour. Compatible performance mod manifests are available through the M7 core.", 12., p))
                .child(
                    rect().width(Size::fill()).spacing(6.).children(Performance::all().into_iter().map(|preset| {
                        let label = preset.label().to_owned();
                        let picked = label.clone();
                        let active = current == label;
                        card(p).width(Size::fill())
                            .child(strong(label.clone(), 13., p.text))
                            .child(dim(preset.hint().to_owned(), 10.5, p))
                            .child(if active {
                                chip("Selected", p, true).into_element()
                            } else {
                                action("Use this preset", p).on_press(move |_| w_perf.set(picked.clone())).into_element()
                            })
                    })),
                );
        }
        _ => {
            let account = s
                .accounts
                .active_account()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "Not signed in (you can sign in later)".into());
            content = content
                .child(strong("You're ready", 20., p.text))
                .child(summary_row(p, "Account", account))
                .child(summary_row(p, "Minecraft", format!("{} · {}", version.read(), loader.read())))
                .child(summary_row(p, "Memory", format!("{} MiB", w_memory.read())))
                .child(summary_row(p, "Appearance", w_appearance.read().clone()))
                .child(summary_row(p, "Performance", w_perf.read().clone()))
                .child(summary_row(p, "Instance", "Aero Default".into()))
                .child(dim(
                    "Installation can be deferred: the instance is created now and installed whenever you choose.",
                    11.,
                    p,
                ))
                .child(
                    rect().horizontal().spacing(8.)
                        .child(action("Create instance and install", p).enabled(!s.busy)                        .on_press({
                            let backend = backend.clone();
                            let answers = base.clone();
                            move |_| backend.send(Command::FinishSetup(step, answers.clone(), true))
                        }))
                        .child(action("Create instance only", p).enabled(!s.busy).on_press({
                            let backend = backend.clone();
                            let answers = base.clone();
                            move |_| backend.send(Command::FinishSetup(step, answers.clone(), false))
                        })),
                );
        }
    }

    let mut nav = rect()
        .horizontal()
        .width(Size::fill())
        .spacing(8.)
        .cross_align(Alignment::Center);
    if step > 0 {
        nav = nav.child(action("Back", p).enabled(!s.busy).on_press({
            let backend = backend.clone();
            let answers = base.clone();
            move |_| {
                backend.send(Command::NavigateSetup(
                    onboarding::Direction::Back,
                    step,
                    answers.clone(),
                ));
            }
        }));
    } else {
        nav = nav.child(rect().width(Size::px(60.)));
    }
    nav = nav.child(rect().width(Size::flex(1.)));
    if step == 1 {
        nav = nav.child(ghost("Skip for now", p).enabled(!s.busy).on_press({
            let backend = backend.clone();
            let answers = base.clone();
            move |_| {
                backend.send(Command::NavigateSetup(
                    onboarding::Direction::Skip,
                    step,
                    answers.clone(),
                ));
            }
        }));
    }
    if step < 6 {
        nav = nav.child(
            Button::new()
                .theme_layout(
                    ButtonLayoutThemePartial::default()
                        .corner_radius(8.)
                        .padding((0., 26.))
                        .height(Size::px(40.)),
                )
                .theme_colors(
                    ButtonColorsThemePartial::default()
                        .color(Color::WHITE)
                        .background(p.green)
                        .hover_background(p.green_hover)
                        .border_fill(p.green_border)
                        .focus_border_fill(p.accent),
                )
                .enabled(!s.busy)
                .child(strong("Next", 14., Color::WHITE))
                .on_press({
                    let backend = backend.clone();
                    let answers = base.clone();
                    move |_| {
                        backend.send(Command::NavigateSetup(
                            onboarding::Direction::Next,
                            step,
                            answers.clone(),
                        ));
                    }
                }),
        );
    }
    nav = nav.child(
        ghost(
            if step == onboarding::LAST_STEP {
                "Open the launcher"
            } else {
                "Set up later"
            },
            p,
        )
        .enabled(!s.busy)
        .on_press({
            let backend = backend.clone();
            let answers = base.clone();
            move |_| {
                if step == onboarding::LAST_STEP {
                    backend.send(Command::FinishSetup(step, answers.clone(), false));
                } else {
                    backend.send(Command::DeferSetup(step, answers.clone()));
                }
            }
        }),
    );
    let _ = w_auto;

    rect()
        .expanded()
        .center()
        .child(
            panel(p)
                .width(Size::px(640.))
                .padding((22., 24.))
                .spacing(12.)
                .child(
                    rect()
                        .horizontal()
                        .width(Size::fill())
                        .cross_align(Alignment::Center)
                        .child(strong(
                            format!("Setup · step {} of {}", step + 1, onboarding::STEPS),
                            11.5,
                            p.text_dim,
                        ))
                        .child(rect().width(Size::flex(1.)))
                        .child(step_dots(step, p)),
                )
                .child(content)
                .maybe(!s.message.is_empty() && s.message != "Ready.", |r| {
                    r.child(dim(s.message.clone(), 10.5, p))
                })
                .child(hairline(p))
                .child(nav),
        )
        .into_element()
}

fn validate_loader(loader: &str, version: &str) -> Result<(), ()> {
    if loader == "Forge" && version != "1.8.9" {
        return Err(());
    }
    if loader == "Fabric" && version == "1.8.9" {
        return Err(());
    }
    Ok(())
}
