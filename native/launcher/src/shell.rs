// SPDX-License-Identifier: MIT
//! Window chrome: the 64 px icon rail, compact top bar and status strip.
use crate::{design::*, ui::Ui};
use aero_core::{Instance, storage::Settings};
use freya::prelude::*;

pub fn rail(ui: &Ui, p: Palette) -> Rect {
    let mut page = ui.page;
    let mut detail = ui.detail;
    let mut rail = rect()
        .width(Size::px(RAIL_WIDTH))
        .height(Size::fill())
        .padding((10., 8.))
        .spacing(6.)
        .cross_align(Alignment::Center)
        .background(p.glass)
        .border(Border::new().width(1.).fill(p.border_soft));
    rail = rail.child(
        rect()
            .width(Size::px(44.))
            .height(Size::px(42.))
            .center()
            .child(logo(34.)),
    );
    rail = rail.child(
        rect()
            .width(Size::px(28.))
            .height(Size::px(1.))
            .background(p.border_soft),
    );
    for (index, (label, _)) in NAV.iter().enumerate() {
        let selected = page.read().eq(&index);
        rail = rail.child(TooltipContainer::new(Tooltip::new_text(*label)).child(
            rail_button(index, selected, p).on_press(move |_| {
                page.set(index);
                detail.set(String::new());
            }),
        ));
    }
    rail
}

pub fn top_bar(
    ui: &Ui,
    p: Palette,
    settings: &Settings,
    instances: &[Instance],
    chosen: Option<&Instance>,
    backend: &crate::backend::Backend,
    busy: bool,
) -> Rect {
    let mut selected = ui.selected;
    let page = ui.page;
    let mut detail = ui.detail;
    let mut instance_menu = None;
    if !instances.is_empty() {
        instance_menu = Some(
            Select::new()
                .selected_item(
                    chosen
                        .map(|i| format!("{}  ·  {}", i.name, i.version))
                        .unwrap_or_else(|| "Select instance".into()),
                )
                .children(instances.iter().map(|i| {
                    let id = i.id.clone();
                    let current = chosen.is_some_and(|c| c.id == i.id);
                    MenuItem::new()
                        .selected(current)
                        .on_press(move |_| {
                            selected.set(id.clone());
                            detail.set(String::new());
                        })
                        .child(format!("{}  ·  {} {}", i.name, i.version, i.loader))
                })),
        );
    }
    let mut bar = rect()
        .horizontal()
        .width(Size::fill())
        .height(Size::px(TOP_BAR_HEIGHT))
        .padding((0., 14.))
        .spacing(10.)
        .cross_align(Alignment::Center)
        .background(p.glass)
        .border(Border::new().width(1.).fill(p.border_soft))
        .child(
            rect()
                .spacing(1.)
                .child(strong("AERO CLIENT", 12., p.text))
                .child(dim("Launcher", 9.5, p)),
        )
        .child(
            rect()
                .width(Size::px(1.))
                .height(Size::px(24.))
                .background(p.border_soft),
        );
    bar = bar.maybe(instance_menu.is_some(), |r| match instance_menu {
        Some(menu) => r.child(menu),
        None => r,
    });
    if let Some(chosen) = chosen {
        bar = bar
            .child(chip(chosen.version.clone(), p, false))
            .child(chip(format!("{}", chosen.loader), p, false))
            .child(status_chip(&chosen.status, p));
    }
    bar = bar.child(rect().width(Size::flex(1.)));
    if !settings.show_news {
        bar = bar.child(ghost("News", p).on_press({
            let mut page = page;
            move |_| page.set(0)
        }));
    }
    bar = bar
        .child(action("Guest", p).on_press({
            let mut page = page;
            move |_| page.set(4)
        }))
        .child(
            TooltipContainer::new(Tooltip::new_text("Preferences")).child(
                rail_button(4, false, p).on_press({
                    let mut page = page;
                    move |_| page.set(4)
                }),
            ),
        );
    let _ = (backend, busy);
    bar
}

pub fn status_bar(p: Palette, message: &str, activity: &aero_core::Progress, busy: bool) -> Rect {
    let percent = if activity.total > 0 {
        activity.bytes as f32 / activity.total as f32 * 100.
    } else if activity.files_total > 0 {
        activity.files_done as f32 / activity.files_total as f32 * 100.
    } else {
        0.
    };
    let mut bar = rect()
        .horizontal()
        .width(Size::fill())
        .height(Size::px(STATUS_HEIGHT))
        .padding((0., 14.))
        .spacing(12.)
        .cross_align(Alignment::Center)
        .background(p.glass)
        .border(Border::new().width(1.).fill(p.border_soft))
        .child(dim(message.to_owned(), 11., p))
        .child(rect().width(Size::flex(1.)));
    if busy {
        bar = bar
            .child(rect().width(Size::px(120.)).child(progress(percent, p)))
            .child(dim(format!("{percent:.0}%"), 10.5, p));
    }
    bar.child(dim(if busy { "Working…" } else { "Idle" }, 10.5, p))
}
