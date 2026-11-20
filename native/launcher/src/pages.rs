// SPDX-License-Identifier: MIT
//! Page composition. No hooks are called here: every piece of state is owned by
//! the app root and passed in, so page re-renders stay deterministic.
use crate::{
    backend::{Backend, Command, Snapshot},
    design::*,
    ui::Ui,
};
use aero_core::{Instance, Loader, TARGETS};
use freya::prelude::*;

pub fn page(index: usize, ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    match index {
        0 => home(ui, s, p, b),
        1 if !ui.detail.read().is_empty() => instance_detail(ui, s, p, b),
        1 => instances(ui, s, p, b),
        2 => discover(ui, s, p, b),
        3 => downloads(ui, s, p, b),
        _ => settings(ui, s, p, b),
    }
}

fn chosen_instance<'a>(ui: &Ui, s: &'a Snapshot) -> Option<&'a Instance> {
    let id = ui.selected.read().clone();
    s.instances
        .iter()
        .find(|i| i.id == id)
        .or_else(|| s.instances.first())
}

fn scroll_column() -> ScrollView {
    ScrollView::new().expanded()
}

fn page_body() -> Rect {
    rect().width(Size::fill()).padding((18., 20.)).spacing(14.)
}

fn empty(p: Palette, message: &str) -> Rect {
    rect()
        .width(Size::fill())
        .padding((16., 18.))
        .corner_radius(6.)
        .background(p.surface_alt)
        .border(Border::new().width(1.).fill(p.border_soft))
        .child(dim(message.to_owned(), 12., p))
}

fn kv(p: Palette, key: &str, value: String) -> Rect {
    rect()
        .horizontal()
        .width(Size::fill())
        .spacing(10.)
        .child(
            rect()
                .width(Size::px(132.))
                .child(dim(key.to_owned(), 11.5, p)),
        )
        .child(text(value, 11.5, p.text))
}

// ---------------------------------------------------------------- home

pub fn home(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let chosen = chosen_instance(ui, s);
    let mut rotation = ui.rotation;
    let low = *ui.low.read();
    let page = ui.page;
    let detail = ui.detail;
    let selected = ui.selected;
    let mut news_visible = ui.news_visible;
    let install = b.clone();
    let open_detail = b.clone();
    let refresh_news = b.clone();

    let id = chosen.map(|i| i.id.clone()).unwrap_or_default();
    let ready = chosen.is_some_and(|i| i.status == "READY");
    let name = chosen
        .map(|i| i.name.clone())
        .unwrap_or_else(|| "Welcome to Aero".into());

    let launch = if chosen.is_none() {
        primary("Create an instance", p).on_press({
            let mut page = page;
            move |_| page.set(1)
        })
    } else if ready {
        primary("Launch", p).enabled(false)
    } else {
        primary("Install game files", p).enabled(!s.busy).on_press({
            let install = install.clone();
            let id = id.clone();
            let mut page = page;
            move |_| {
                install.send(Command::Install(id.clone()));
                page.set(3);
            }
        })
    };

    let hero = rect()
        .width(Size::flex(1.))
        .height(Size::fill())
        .padding((20., 24.))
        .spacing(8.)
        .corner_radius(10.)
        .background(p.glass)
        .border(rim(p))
        .child(
            rect()
                .horizontal()
                .width(Size::fill())
                .spacing(8.)
                .cross_align(Alignment::Center)
                .child(dim("SELECTED INSTANCE", 10., p))
                .child(rect().width(Size::flex(1.)))
                .maybe(chosen.is_some(), |r| {
                    r.child(action("Manage", p).on_press({
                        let open_detail = open_detail.clone();
                        let mut selected = selected;
                        let id = id.clone();
                        move |_| {
                            selected.set(id.clone());
                            open_details(&open_detail, detail, &id);
                        }
                    }))
                }),
        )
        .child(
            rect()
                .width(Size::fill())
                .center()
                .spacing(3.)
                .child(strong(name.clone(), 23., p.text))
                .child(
                    rect().horizontal().spacing(6.).children(
                        chosen
                            .map(|i| {
                                vec![
                                    chip(i.version.clone(), p, false),
                                    chip(format!("{}", i.loader), p, false),
                                    status_chip(&i.status, p),
                                ]
                            })
                            .unwrap_or_default(),
                    ),
                ),
        )
        .child(
            rect()
                .expanded()
                .center()
                .child(
                    SvgViewer::new(CHARACTERS[if low { 5 } else { *rotation.read() }])
                        .width(Size::px(206.))
                        .height(Size::px(261.))
                        .a11y_alt("Original Aero explorer preview. Move the pointer to rotate.")
                        .on_pointer_move(move |e: Event<PointerEventData>| {
                            if !low {
                                let next = ((e.element_location().x / 206. * 9.) as usize).min(8);
                                if *rotation.read() != next {
                                    rotation.set(next);
                                }
                            }
                        }),
                )
                .child(dim(
                    if low {
                        "Static preview (low-performance mode)"
                    } else {
                        "Move the pointer across the explorer to rotate"
                    },
                    10.5,
                    p,
                )),
        )
        .child(
            rect()
                .width(Size::fill())
                .center()
                .spacing(6.)
                .child(
                    rect()
                        .horizontal()
                        .spacing(8.)
                        .child(launch)
                        .maybe(ready, |r| {
                            r.child(action("Repair", p).enabled(!s.busy).on_press({
                                let install = install.clone();
                                let id = id.clone();
                                move |_| {
                                    install.send(Command::Install(id.clone()));
                                }
                            }))
                        }),
                )
                .child(dim(
                    if ready {
                        "Launch is disabled until Microsoft sign-in lands in M3."
                    } else if chosen.is_none() {
                        "Instances keep saves, mods and settings separate."
                    } else {
                        "Official files are downloaded, verified and can be repaired."
                    },
                    10.5,
                    p,
                )),
        );

    let news = {
        let entries = s.news.clone();
        let mut panel = rect()
            .width(Size::percent(24.))
            .height(Size::fill())
            .padding((14., 14.))
            .spacing(10.)
            .corner_radius(10.)
            .background(p.glass)
            .border(rim(p))
            .child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .spacing(8.)
                    .cross_align(Alignment::Center)
                    .child(strong("Aero news", 14., p.text))
                    .child(rect().width(Size::flex(1.)))
                    .child(ghost("Refresh", p).enabled(!s.busy).on_press({
                        let refresh_news = refresh_news.clone();
                        move |_| refresh_news.send(Command::RefreshNews)
                    }))
                    .child(ghost("Hide", p).on_press(move |_| news_visible.set(false))),
            )
            .child(
                ScrollView::new().height(Size::flex(1.)).child(
                    rect()
                        .width(Size::fill())
                        .spacing(10.)
                        .children(entries.iter().map(|item| {
                            rect()
                                .width(Size::fill())
                                .spacing(5.)
                                .padding((0., 0., 10., 0.))
                                .child(
                                    SvgViewer::new(include_bytes!("../assets/landscape.svg"))
                                        .width(Size::fill())
                                        .height(Size::px(74.))
                                        .aspect_ratio(AspectRatio::Max),
                                )
                                .child(text(item.title.clone(), 12.5, p.text))
                                .child(dim(format!("{} · {}", item.source, item.date), 10., p))
                                .child(hairline(p))
                        })),
                ),
            );
        if entries.is_empty() {
            panel = panel.child(dim(
                "No cached announcements yet. Refresh to read official release metadata.",
                10.5,
                p,
            ));
        }
        panel
    };

    rect()
        .horizontal()
        .expanded()
        .padding((16., 16.))
        .spacing(14.)
        .child(hero)
        .maybe(*news_visible.read(), |r| r.child(news))
        .maybe(!*news_visible.read(), |r| {
            r.child(action("News", p).on_press(move |_| news_visible.set(true)))
        })
        .into_element()
}

fn open_details(backend: &Backend, mut detail: State<String>, id: &str) {
    detail.set(id.to_owned());
    backend.send(Command::LoadPlan(id.to_owned()));
}

// ---------------------------------------------------------------- instances

pub fn instances(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let create = b.clone();
    let select = b.clone();
    let detail = ui.detail;
    let name = ui.name;
    let mut version = ui.version;
    let mut loader = ui.loader;
    let search = ui.search;
    let selected = ui.selected;
    let page = ui.page;
    let mut inst_memory = ui.inst_memory;
    let mut inst_java = ui.inst_java;

    let form = panel(p)
        .width(Size::fill())
        .child(section(
            p,
            "Create an instance",
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
                                let selected = *version.read() == value;
                                MenuItem::new()
                                    .selected(selected)
                                    .on_press({
                                        let picked = value.clone();
                                        move |_| version.set(picked.clone())
                                    })
                                    .child(value)
                            })),
                    ),
                )
                .child(rect().width(Size::px(110.)).child(
                    Select::new().selected_item(loader.read().clone()).children(
                        ["Vanilla", "Fabric", "Forge"].into_iter().map(|value| {
                            let label = value.to_owned();
                            let chosen = *loader.read() == label;
                            MenuItem::new()
                                .selected(chosen)
                                .on_press({
                                    let picked = label.clone();
                                    move |_| loader.set(picked.clone())
                                })
                                .child(label)
                        }),
                    ),
                ))
                .child(primary("Create", p).enabled(!s.busy).on_press({
                    let create = create.clone();
                    move |_| {
                        create.send(Command::Create(
                            name.read().clone(),
                            version.read().clone(),
                            parse_loader(&loader.read()),
                            false,
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

    let query = search.read().to_lowercase();
    let rows = s
        .instances
        .iter()
        .filter(|i| {
            format!("{} {} {}", i.name, i.version, i.loader)
                .to_lowercase()
                .contains(&query)
        })
        .map(|instance| {
            let id = instance.id.clone();
            let manage = select.clone();
            let current = *selected.read() == id
                || (selected.read().is_empty() && s.instances.first().is_some_and(|f| f.id == id));
            let memory = instance
                .memory_mib
                .map(|m| m.to_string())
                .unwrap_or_default();
            let java = instance.java_path.clone();
            panel(p)
                .width(Size::fill())
                .child(
                    rect()
                        .horizontal()
                        .width(Size::fill())
                        .spacing(12.)
                        .cross_align(Alignment::Center)
                        .child(
                            SvgViewer::new(include_bytes!("../assets/landscape.svg"))
                                .width(Size::px(78.))
                                .height(Size::px(50.)),
                        )
                        .child(
                            rect()
                                .width(Size::flex(1.))
                                .spacing(4.)
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
                        .child(chip(instance.version.clone(), p, false))
                        .child(status_chip(&instance.status, p))
                        .child(action("Manage", p).enabled(!s.busy).on_press({
                            let manage = manage.clone();
                            let mut selected = selected;
                            let id = id.clone();
                            move |_| {
                                selected.set(id.clone());
                                inst_memory.set(memory.clone());
                                inst_java.set(java.clone());
                                open_details(&manage, detail, &id);
                            }
                        })),
                )
                .maybe(current, |r| {
                    r.child(dim("Currently selected for the home screen.", 10., p))
                })
        })
        .collect::<Vec<_>>();

    let mut body = page_body().child(
        rect()
            .horizontal()
            .width(Size::fill())
            .spacing(10.)
            .cross_align(Alignment::Center)
            .child(section(
                p,
                "Library",
                "Instances are isolated by UUID; verified files and runtimes are shared.",
            ))
            .child(rect().width(Size::flex(1.)))
            .child(
                rect()
                    .width(Size::px(220.))
                    .child(Input::new(search).placeholder("Search instances")),
            )
            .child(action("Home", p).on_press({
                let mut page = page;
                move |_| page.set(0)
            })),
    );
    if rows.is_empty() {
        body = body.child(empty(
            p,
            "No instances yet. Create one below to download official game files.",
        ));
    } else {
        body = body.children(rows);
    }
    body = body.child(form);
    scroll_column().child(body).into_element()
}

fn parse_loader(value: &str) -> Loader {
    match value {
        "Fabric" => Loader::Fabric,
        "Forge" => Loader::Forge,
        _ => Loader::Vanilla,
    }
}

// ---------------------------------------------------------------- instance detail

pub fn instance_detail(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let id = ui.detail.read().clone();
    let Some(instance) = s.instances.iter().find(|i| i.id == id) else {
        let mut clear = ui.detail;
        clear.set(String::new());
        return scroll_column()
            .child(page_body().child(empty(p, "That instance is no longer available.")))
            .into_element();
    };
    let install = b.clone();
    let configure = b.clone();
    let duplicate = b.clone();
    let remove = b.clone();
    let rename = b.clone();
    let cancel = b.clone();
    let detail = ui.detail;
    let confirm = ui.confirm;
    let rename_value = ui.rename;
    let inst_memory = ui.inst_memory;
    let inst_java = ui.inst_java;
    let page = ui.page;
    let selected = ui.selected;
    let instance_id = instance.id.clone();
    let name = instance.name.clone();

    let plan = s.plan.clone();
    let plan_value = |key: &str| {
        plan.as_ref()
            .and_then(|v| v.get(key))
            .map(|v| match v {
                serde_json::Value::String(text) => text.clone(),
                other => other.to_string(),
            })
            .unwrap_or_else(|| "—".into())
    };
    let library_count = plan
        .as_ref()
        .and_then(|v| v["libraries"].as_array().map(Vec::len))
        .map(|n| n.to_string())
        .unwrap_or_else(|| "—".into());

    let header = rect()
        .horizontal()
        .width(Size::fill())
        .spacing(10.)
        .cross_align(Alignment::Center)
        .child(action("Back", p).on_press({
            let mut detail = detail;
            move |_| detail.set(String::new())
        }))
        .child(section(
            p,
            &format!("{} · {}", instance.name, instance.version),
            "Advanced installation settings for this instance only.",
        ))
        .child(rect().width(Size::flex(1.)))
        .child(status_chip(&instance.status, p))
        .child(chip(format!("{}", instance.loader), p, false));

    let installation = panel(p)
        .width(Size::flex(1.))
        .child(section(
            p,
            "Installation",
            "Runs on demand; nothing downloads while idle.",
        ))
        .child(
            rect()
                .horizontal()
                .spacing(8.)
                .child(primary("Install · Repair", p).enabled(!s.busy).on_press({
                    let install = install.clone();
                    let id = instance_id.clone();
                    let mut page = page;
                    move |_| {
                        install.send(Command::Install(id.clone()));
                        page.set(3);
                    }
                }))
                .child(action("Cancel", p).enabled(s.busy).on_press({
                    let cancel = cancel.clone();
                    move |_| cancel.cancel()
                })),
        )
        .child(kv(p, "Status", instance.detail.clone()))
        .child(kv(p, "Minecraft", instance.version.clone()))
        .child(kv(p, "Loader", format!("{}", instance.loader)))
        .child(divider_block(p))
        .child(section(
            p,
            "Runtime overrides",
            "Leave empty to use Mojang metadata and automatic Java management.",
        ))
        .child(
            rect()
                .horizontal()
                .spacing(8.)
                .child(
                    rect()
                        .width(Size::px(150.))
                        .child(Input::new(inst_memory).placeholder("Memory MiB")),
                )
                .child(
                    rect()
                        .width(Size::px(300.))
                        .child(Input::new(inst_java).placeholder("Custom java.exe (optional)")),
                ),
        )
        .child(
            action("Save instance settings", p)
                .enabled(!s.busy)
                .on_press({
                    let saved = instance_id.clone();
                    move |_| {
                        let memory = inst_memory.read().trim().to_owned();
                        let parsed = if memory.is_empty() {
                            None
                        } else {
                            memory.parse::<u32>().ok()
                        };
                        configure.send(Command::Configure(
                            saved.clone(),
                            parsed,
                            inst_java.read().clone(),
                        ));
                    }
                }),
        );

    let details = panel(p)
        .width(Size::flex(1.))
        .child(section(
            p,
            "Verified installation",
            "Read only, produced by the last successful install or repair.",
        ))
        .child(kv(p, "Client JAR SHA-1", plan_value("clientSha1")))
        .child(kv(p, "Asset index", plan_value("assetIndexId")))
        .child(kv(p, "Libraries", library_count))
        .child(kv(p, "Java runtime", plan_value("runtime")))
        .child(kv(p, "Authentication", "Required (M3)".into()))
        .child(dim(
            "Full launch arguments and classpath overrides are resolved in M3.",
            10.5,
            p,
        ));

    let management = panel(p)
        .width(Size::fill())
        .child(section(
            p,
            "Manage",
            "Removal is reversible: files move to the Aero trash folder.",
        ))
        .child(
            rect()
                .horizontal()
                .spacing(8.)
                .child(
                    rect()
                        .width(Size::px(260.))
                        .child(Input::new(rename_value).placeholder("New instance name")),
                )
                .child(action("Rename", p).enabled(!s.busy).on_press({
                    let rename = rename.clone();
                    let id = instance_id.clone();
                    move |_| rename.send(Command::Rename(id.clone(), rename_value.read().clone()))
                }))
                .child(action("Duplicate", p).enabled(!s.busy).on_press({
                    let duplicate = duplicate.clone();
                    let id = instance_id.clone();
                    move |_| duplicate.send(Command::Duplicate(id.clone()))
                }))
                .child(danger("Move to trash", p).enabled(!s.busy).on_press({
                    let mut confirm = confirm;
                    let id = instance_id.clone();
                    move |_| confirm.set(id.clone())
                })),
        )
        .maybe(*confirm.read() == instance_id, |r| {
            r.child(
                rect()
                    .horizontal()
                    .spacing(8.)
                    .child(dim(
                        "Move this instance to recoverable trash? Saves are preserved there.",
                        11.,
                        p,
                    ))
                    .child(ghost("Keep", p).on_press({
                        let mut confirm = confirm;
                        move |_| confirm.set(String::new())
                    }))
                    .child(danger("Confirm", p).on_press({
                        let remove = remove.clone();
                        let mut confirm = confirm;
                        let mut detail = detail;
                        let mut selected = selected;
                        let id = instance_id.clone();
                        move |_| {
                            remove.send(Command::Remove(id.clone()));
                            confirm.set(String::new());
                            detail.set(String::new());
                            selected.set(String::new());
                        }
                    })),
            )
        })
        .child(kv(p, "Folder", s.storage.clone()))
        .child(kv(p, "Name", name));

    scroll_column()
        .child(
            page_body()
                .child(header)
                .child(
                    rect()
                        .horizontal()
                        .width(Size::fill())
                        .spacing(12.)
                        .child(installation)
                        .child(details),
                )
                .child(management),
        )
        .into_element()
}

fn divider_block(p: Palette) -> Rect {
    rect()
        .width(Size::fill())
        .padding((6., 0.))
        .child(hairline(p))
}

// ---------------------------------------------------------------- discover

pub fn discover(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let discover = b.clone();
    let query = ui.query;
    let mut kind = ui.kind;
    let mut filter_version = ui.filter_version;
    let mut filter_loader = ui.filter_loader;

    let filters = panel(p)
        .width(Size::fill())
        .child(section(
            p,
            "Discover",
            "Searches Modrinth only when you ask. Project installation belongs to M4.",
        ))
        .child(
            rect()
                .horizontal()
                .width(Size::fill())
                .spacing(8.)
                .child(
                    rect()
                        .width(Size::flex(1.))
                        .child(Input::new(query).placeholder("Search mods, packs, shaders")),
                )
                .child(
                    rect().width(Size::px(126.)).child(
                        Select::new().selected_item(kind.read().clone()).children(
                            ["mod", "modpack", "resourcepack", "shader"]
                                .into_iter()
                                .map(|value| {
                                    let label = value.to_owned();
                                    let selected = *kind.read() == label;
                                    MenuItem::new()
                                        .selected(selected)
                                        .on_press({
                                            let picked = label.clone();
                                            move |_| kind.set(picked.clone())
                                        })
                                        .child(label)
                                }),
                        ),
                    ),
                )
                .child(
                    rect().width(Size::px(116.)).child(
                        Select::new()
                            .selected_item(if filter_version.read().is_empty() {
                                "Any version".to_owned()
                            } else {
                                filter_version.read().clone()
                            })
                            .children(
                                std::iter::once("Any version".to_owned())
                                    .chain(TARGETS.iter().map(|v| (*v).to_owned()))
                                    .map(|value| {
                                        let stored = if value == "Any version" {
                                            String::new()
                                        } else {
                                            value.clone()
                                        };
                                        let selected = *filter_version.read() == stored;
                                        MenuItem::new()
                                            .selected(selected)
                                            .on_press(move |_| filter_version.set(stored.clone()))
                                            .child(value)
                                    }),
                            ),
                    ),
                )
                .child(
                    rect().width(Size::px(104.)).child(
                        Select::new()
                            .selected_item(if filter_loader.read().is_empty() {
                                "Any loader".to_owned()
                            } else {
                                filter_loader.read().clone()
                            })
                            .children(["Any loader", "fabric", "forge"].into_iter().map(|value| {
                                let stored = if value == "Any loader" {
                                    String::new()
                                } else {
                                    value.to_owned()
                                };
                                let selected = *filter_loader.read() == stored;
                                MenuItem::new()
                                    .selected(selected)
                                    .on_press(move |_| filter_loader.set(stored.clone()))
                                    .child(value.to_owned())
                            })),
                    ),
                )
                .child(primary("Search", p).enabled(!s.busy).on_press({
                    let discover = discover.clone();
                    move |_| {
                        discover.send(Command::Discover(
                            query.read().clone(),
                            kind.read().clone(),
                            filter_version.read().clone(),
                            filter_loader.read().clone(),
                        ))
                    }
                })),
        );

    let results = s
        .results
        .iter()
        .map(|item| {
            panel(p).width(Size::fill()).child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .spacing(10.)
                    .cross_align(Alignment::Center)
                    .child(
                        rect()
                            .spacing(3.)
                            .width(Size::flex(1.))
                            .child(strong(item.title.clone(), 14., p.text))
                            .child(dim(item.description.clone(), 11., p)),
                    )
                    .child(chip(item.project_type.clone(), p, false))
                    .child(dim(format!("modrinth.com/{}", item.slug), 10., p)),
            )
        })
        .collect::<Vec<_>>();

    let mut body = page_body().child(filters);
    if results.is_empty() {
        body = body.child(empty(
            p,
            "No results yet. Enter a query and search Modrinth; no background browsing happens.",
        ));
    } else {
        body = body.children(results);
    }
    scroll_column().child(body).into_element()
}

// ---------------------------------------------------------------- downloads

pub fn downloads(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let cancel = b.clone();
    let retry = b.clone();
    let chosen = chosen_instance(ui, s);
    let id = chosen.map(|i| i.id.clone()).unwrap_or_default();
    let percent = if s.progress.total > 0 {
        s.progress.bytes as f32 / s.progress.total as f32 * 100.
    } else if s.progress.files_total > 0 {
        s.progress.files_done as f32 / s.progress.files_total as f32 * 100.
    } else {
        0.
    };

    let current = panel(p)
        .width(Size::fill())
        .child(section(
            p,
            "Current activity",
            "One installation at a time; partial files resume after cancellation.",
        ))
        .child(text(
            if s.progress.item.is_empty() {
                "Nothing is downloading.".to_owned()
            } else {
                s.progress.item.clone()
            },
            12.5,
            p.text,
        ))
        .child(progress(percent, p))
        .child(dim(
            format!(
                "{:.1} / {:.1} MiB · {:.1} MiB/s · {} / {} files",
                s.progress.bytes as f64 / 1_048_576.,
                s.progress.total as f64 / 1_048_576.,
                s.progress.bytes_per_second as f64 / 1_048_576.,
                s.progress.files_done,
                s.progress.files_total
            ),
            11.,
            p,
        ))
        .child(
            rect()
                .horizontal()
                .spacing(8.)
                .child(action("Cancel", p).enabled(s.busy).on_press({
                    let cancel = cancel.clone();
                    move |_| cancel.cancel()
                }))
                .child(
                    primary("Retry · Repair", p)
                        .enabled(!s.busy && chosen.is_some())
                        .on_press({
                            let retry = retry.clone();
                            let id = id.clone();
                            move |_| retry.send(Command::Install(id.clone()))
                        }),
                ),
        );

    let log = panel(p).width(Size::fill()).child(section(
        p,
        "Session log",
        "Verified steps from this session. Nothing is stored remotely.",
    ));
    let entries = if s.log.is_empty() {
        empty(p, "No activity yet in this session.").into_element()
    } else {
        rect()
            .width(Size::fill())
            .spacing(3.)
            .children(
                s.log
                    .iter()
                    .rev()
                    .take(24)
                    .map(|line| dim(line.clone(), 11., p)),
            )
            .into_element()
    };

    scroll_column()
        .child(page_body().child(current).child(log.child(entries)))
        .into_element()
}

// ---------------------------------------------------------------- settings

pub fn settings(ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Element {
    let save = b.clone();
    let appearance = ui.appearance;
    let mut theme = ui.theme;
    let memory = ui.memory;
    let java = ui.java;
    let low = ui.low;
    let reduced = ui.reduced;
    let show_news = ui.show_news;

    let appearance_row = rect().horizontal().spacing(8.).children(
        ["Aero Light", "Aero Dark", "Aero Lite"]
            .into_iter()
            .map(|value| {
                let chosen = *appearance.read() == value;
                let mut appearance = appearance;
                Button::new()
                    .theme_layout(
                        ButtonLayoutThemePartial::default()
                            .corner_radius(5.)
                            .padding((0., 14.))
                            .height(Size::px(30.)),
                    )
                    .theme_colors(
                        ButtonColorsThemePartial::default()
                            .color(if chosen { p.accent_text } else { p.text })
                            .background(if chosen { p.accent } else { p.surface })
                            .hover_background(if chosen { p.accent } else { p.accent_soft })
                            .border_fill(p.border)
                            .focus_border_fill(p.accent),
                    )
                    .on_press(move |_| {
                        appearance.set(value.to_owned());
                        theme.set(design_theme(value));
                    })
                    .child(text(
                        value.to_owned(),
                        12.,
                        if chosen { p.accent_text } else { p.text },
                    ))
            }),
    );

    let visuals = panel(p)
        .width(Size::flex(1.))
        .child(section(
            p,
            "Appearance",
            "Aero Light and Dark use the static landscape; Lite is a flat low-cost preset.",
        ))
        .child(appearance_row)
        .child(
            rect()
                .horizontal()
                .spacing(10.)
                .cross_align(Alignment::Center)
                .child(Switch::new(show_news))
                .child(text("Show cached news on the home screen", 12., p.text)),
        )
        .child(
            rect()
                .horizontal()
                .spacing(10.)
                .cross_align(Alignment::Center)
                .child(Switch::new(low))
                .child(text(
                    "Low-performance mode (static character preview)",
                    12.,
                    p.text,
                )),
        )
        .child(
            rect()
                .horizontal()
                .spacing(10.)
                .cross_align(Alignment::Center)
                .child(Switch::new(reduced))
                .child(text("Reduced motion", 12., p.text)),
        );

    let runtime = panel(p)
        .width(Size::flex(1.))
        .child(section(
            p,
            "Minecraft and Java",
            "Game Java is detected or downloaded from official metadata; nothing is bundled.",
        ))
        .child(text(
            format!("Default game memory: {} MiB", *memory.read()),
            12.,
            p.text,
        ))
        .child(
            Slider::new({
                let mut memory = memory;
                move |percent: f64| memory.set((1024. + percent / 100. * 31744.).round() as u32)
            })
            .value(((*memory.read() as f64 - 1024.) / 31744. * 100.).clamp(0., 100.))
            .size(Size::px(360.))
            .step(1.),
        )
        .child(dim(
            "Overrides are available per instance in Library → Manage.",
            10.5,
            p,
        ))
        .child(
            rect()
                .width(Size::px(420.))
                .child(Input::new(java).placeholder("Optional compatible java.exe path")),
        );

    let storage = panel(p)
        .width(Size::fill())
        .child(section(
            p,
            "Storage",
            "Shared verified assets live once, outside instances.",
        ))
        .child(kv(p, "Data folder", s.storage.clone()))
        .child(kv(
            p,
            "Metadata cache",
            "6 hours, refreshed on demand".into(),
        ))
        .child(dim(
            "Removing an instance moves it to trash; Aero never silently deletes user files.",
            10.5,
            p,
        ));

    let accounts = panel(p)
        .width(Size::fill())
        .child(section(
            p,
            "Accounts and updates",
            "No Aero account, telemetry, advertising or paid tier exists.",
        ))
        .child(text(
            "Not signed in. Microsoft sign-in, entitlement checks and launch arrive in M3.",
            12.,
            p.text,
        ))
        .child(dim(
            "Updates are manual: github.com/karmangamerz-coder/Aero-client",
            10.5,
            p,
        ));

    scroll_column()
        .child(
            page_body()
                .child(section(
                    p,
                    "Settings",
                    "Preferences are stored on this device only and never uploaded.",
                ))
                .child(
                    rect()
                        .horizontal()
                        .width(Size::fill())
                        .spacing(12.)
                        .child(visuals)
                        .child(runtime),
                )
                .child(storage)
                .child(accounts)
                .child(primary("Save preferences", p).enabled(!s.busy).on_press({
                    let save = save.clone();
                    move |_| {
                        save.send(Command::Save(aero_core::storage::Settings {
                            appearance: appearance.read().clone(),
                            memory_mib: *memory.read(),
                            custom_java: java.read().clone(),
                            low_performance: *low.read(),
                            reduced_motion: *reduced.read(),
                            show_news: *show_news.read(),
                        }))
                    }
                })),
        )
        .into_element()
}

fn design_theme(name: &str) -> Theme {
    crate::design::theme(name)
}
