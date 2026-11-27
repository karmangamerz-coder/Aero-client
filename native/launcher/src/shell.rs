// SPDX-License-Identifier: GPL-3.0-only
//! Window chrome: narrow icon-only rail, compact top bar with the Aero wordmark and
//! the Microsoft account selector, and a status strip that appears only while work
//! is running.
use crate::{
    backend::{Backend, Command, Snapshot},
    design::*,
    ui::Ui,
};
use freya::prelude::*;

pub fn rail(ui: &Ui, p: Palette, b: &Backend) -> Rect {
    let mut page = ui.page;
    let mut detail = ui.detail;
    let mut rail = rect()
        .width(Size::px(RAIL_WIDTH))
        .height(Size::fill())
        .padding((8., 8.))
        .spacing(6.)
        .cross_align(Alignment::Center)
        .background(p.rail)
        .border(Border::new().width(1.).fill(p.border_soft));
    rail = rail.child(
        rect()
            .width(Size::px(40.))
            .height(Size::px(40.))
            .center()
            .child(logo(26.)),
    );
    rail = rail.child(hairline(p));
    for (index, (label, _)) in NAV.iter().enumerate() {
        let selected = page.read().eq(&index);
        let backend = b.clone();
        let ui = *ui;
        rail = rail.child(
            Button::new()
                .theme_layout(
                    ButtonLayoutThemePartial::default()
                        .width(Size::px(RAIL_WIDTH - 18.))
                        .height(Size::px(38.))
                        .corner_radius(6.)
                        .padding((0., 10.)),
                )
                .theme_colors(
                    ButtonColorsThemePartial::default()
                        .color(p.text)
                        .background(if selected {
                            p.accent_soft
                        } else {
                            Color::TRANSPARENT
                        })
                        .hover_background(p.surface)
                        .border_fill(if selected {
                            p.border
                        } else {
                            Color::TRANSPARENT
                        })
                        .focus_border_fill(p.accent),
                )
                .child(
                    rect()
                        .horizontal()
                        .spacing(10.)
                        .cross_align(Alignment::Center)
                        .child(icon(index, p, selected))
                        .child(text(*label, 13., p.text)),
                )
                .on_press(move |_| {
                    page.set(index);
                    detail.set(String::new());
                    if index == 2 {
                        backend.send(Command::Discover(
                            ui.query.read().clone(),
                            ui.kind.read().clone(),
                            ui.mod_version.read().clone(),
                            ui.mod_loader.read().clone(),
                            ui.mod_category.read().clone(),
                            ui.mod_sort.read().clone(),
                        ));
                    }
                }),
        );
    }
    rail
}

pub fn top_bar(_ui: &Ui, s: &Snapshot, p: Palette, b: &Backend) -> Rect {
    let mut items: Vec<MenuItem> = vec![];
    for account in &s.accounts.accounts {
        let id = account.uuid.clone();
        let backend = b.clone();
        items.push(
            MenuItem::new()
                .selected(s.accounts.active.as_deref() == Some(account.uuid.as_str()))
                .on_press(move |_| backend.send(Command::SwitchAccount(id.clone())))
                .child(account.name.clone()),
        );
    }
    let add = b.clone();
    items.push(
        MenuItem::new()
            .on_press(move |_| add.send(Command::SignIn))
            .child("Add account"),
    );
    let sign_out = b.clone();
    items.push(
        MenuItem::new()
            .on_press(move |_| sign_out.send(Command::SignOut))
            .child("Sign out"),
    );
    let label = s
        .accounts
        .active_account()
        .map(|account| account.name.clone())
        .unwrap_or_else(|| "No Account".to_owned());

    rect()
        .horizontal()
        .content(Content::Flex)
        .width(Size::fill())
        .height(Size::px(TOP_BAR_HEIGHT))
        .padding((0., 14.))
        .spacing(10.)
        .cross_align(Alignment::Center)
        .background(p.glass_soft)
        .border(Border::new().width(1.).fill(p.border_soft))
        .child(logo(24.))
        .child(strong("Aero Client", 15.5, p.art_text))
        .child(rect().width(Size::flex(1.)))
        .child(character(&s.head, 28., 28.))
        .child(
            rect()
                .width(Size::px(190.))
                .height(Size::px(34.))
                .child(Select::new().selected_item(label).children(items)),
        )
        .child(
            rect()
                .width(Size::px(1.))
                .height(Size::px(20.))
                .background(p.border_soft),
        )
        .child(dim(
            match s.accounts.active_account() {
                Some(account) if account.owns_minecraft => "Minecraft",
                Some(_) => "No ownership",
                None => "Microsoft",
            },
            10.5,
            p,
        ))
}

pub fn status_strip(
    p: Palette,
    message: &str,
    activity: &aero_core::Progress,
    busy: bool,
    launching: bool,
) -> Rect {
    let percent = if activity.total > 0 {
        activity.bytes as f32 / activity.total as f32 * 100.
    } else if activity.files_total > 0 {
        activity.files_done as f32 / activity.files_total as f32 * 100.
    } else {
        0.
    };
    let mut bar = rect()
        .horizontal()
        .content(Content::Flex)
        .width(Size::fill())
        .height(Size::px(STATUS_HEIGHT))
        .padding((0., 14.))
        .spacing(12.)
        .cross_align(Alignment::Center)
        .background(p.glass_soft)
        .child(dim(message.to_owned(), 11., p))
        .child(rect().width(Size::flex(1.)));
    if busy && percent > 0. {
        bar = bar
            .child(rect().width(Size::px(140.)).child(progress(percent, p)))
            .child(dim(format!("{percent:.0}%"), 10.5, p));
    }
    bar.child(dim(
        if launching {
            "Minecraft running"
        } else if busy {
            "Working…"
        } else {
            "Ready"
        },
        10.5,
        p,
    ))
}

pub fn auth_notice(s: &Snapshot, p: Palette) -> Rect {
    rect().width(Size::fill()).height(Size::px(112.))
        .padding((9., 16.)).spacing(5.)
        .background(p.surface).border(Border::new().width(1.).fill(p.border))
        .child(strong(if s.authenticating { "Completing Microsoft sign-in…" } else { "Sign-in could not finish" }, 13., if s.authenticating { p.accent } else { p.danger }))
        .child(text(s.signin_state.clone(), 12., p.text))
        .child(dim(if s.authenticating {
            "Keep Aero open. Your account appears only after Minecraft verification succeeds."
        } else {
            "Retry from the account menu. Registration approval errors require approval for Aero's own client ID; browser authorization alone is insufficient."
        }, 11., p))
}
