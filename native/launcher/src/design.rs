// SPDX-License-Identifier: MIT
use freya::prelude::*;

pub fn theme(name: &str) -> Theme {
    let dark = name == "Aero Dark";
    let mut t = if dark { dark_theme() } else { light_theme() };
    t.name = if dark {
        "Aero Dark"
    } else if name == "Aero Lite" {
        "Aero Lite"
    } else {
        "Aero Light"
    };
    let c = &mut t.colors;
    c.primary = Color::from_rgb(8, 126, 186);
    c.secondary = Color::from_rgb(141, 216, 248);
    c.tertiary = Color::from_rgb(20, 153, 190);
    c.background = if dark {
        Color::from_rgb(12, 37, 56)
    } else {
        Color::from_rgb(234, 248, 255)
    };
    c.surface_primary = if dark {
        Color::from_rgb(22, 62, 80)
    } else {
        Color::from_rgb(205, 237, 248)
    };
    c.surface_secondary = if dark {
        Color::from_rgb(30, 79, 99)
    } else {
        Color::from_rgb(183, 228, 244)
    };
    c.surface_tertiary = if dark {
        Color::from_rgb(18, 51, 72)
    } else {
        Color::from_rgb(239, 250, 255)
    };
    c.text_primary = if dark {
        Color::from_rgb(234, 248, 255)
    } else {
        Color::from_rgb(18, 60, 86)
    };
    c.text_secondary = if dark {
        Color::from_rgb(172, 215, 231)
    } else {
        Color::from_rgb(55, 92, 108)
    };
    c.text_placeholder = c.text_secondary;
    c.border = if dark {
        Color::from_rgb(58, 111, 136)
    } else {
        Color::from_rgb(160, 207, 223)
    };
    c.border_focus = Color::from_rgb(8, 126, 186);
    c.focus = Color::from_rgb(37, 184, 223);
    c.active = Color::from_rgb(65, 157, 183);
    c.success = Color::from_rgb(49, 149, 55);
    t
}
pub fn glass(dark: bool) -> Color {
    if dark {
        Color::from_argb(238, 16, 47, 66)
    } else {
        Color::from_argb(235, 234, 248, 255)
    }
}
pub fn panel(dark: bool) -> Rect {
    rect()
        .padding(18.)
        .spacing(12.)
        .corner_radius(10.)
        .background(glass(dark))
        .border(Border::new().width(1.).fill(if dark {
            (58, 111, 136)
        } else {
            (180, 224, 240)
        }))
}
pub fn action(text: impl Into<String>) -> Button {
    Button::new()
        .theme_layout(
            ButtonLayoutThemePartial::default()
                .corner_radius(7.)
                .padding((9., 15.)),
        )
        .child(label().text(text.into()).font_size(14.))
}
pub fn primary(text: impl Into<String>) -> Button {
    action(text).theme_colors(
        ButtonColorsThemePartial::default()
            .background((49, 149, 55))
            .hover_background((66, 170, 63))
            .color(Color::WHITE)
            .border_fill((140, 210, 121))
            .focus_border_fill((8, 126, 186)),
    )
}
pub fn heading(title: &str, subtitle: &str) -> Rect {
    rect()
        .spacing(6.)
        .child(label().text(title.to_owned()).font_size(25.))
        .child(label().text(subtitle.to_owned()).font_size(13.))
}
pub fn choice(mut value: State<String>, values: Vec<String>) -> Select {
    Select::new()
        .selected_item(value.read().clone())
        .children(values.into_iter().map(move |v| {
            let selected = *value.read() == v;
            MenuItem::new()
                .selected(selected)
                .on_press({
                    let v = v.clone();
                    move |_| value.set(v.clone())
                })
                .child(v)
        }))
}
pub fn progress(bytes: u64, total: u64) -> Rect {
    let percent = if total > 0 {
        (bytes as f32 / total as f32 * 100.).min(100.)
    } else {
        0.
    };
    rect()
        .width(Size::fill())
        .height(Size::px(8.))
        .corner_radius(4.)
        .background((175, 213, 224))
        .a11y_role(AccessibilityRole::ProgressIndicator)
        .a11y_alt(format!("{percent:.0}%"))
        .child(
            rect()
                .width(Size::percent(percent))
                .height(Size::fill())
                .corner_radius(4.)
                .background((72, 172, 65)),
        )
}
pub fn icon(page: usize) -> SvgViewer {
    const ICONS: [&[u8]; 6] = [
        br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="m8 5 11 7-11 7z" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round"/></svg>"#,
        br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M4 4h6v6H4zm10 0h6v6h-6zM4 14h6v6H4zm10 0h6v6h-6z" fill="none" stroke="currentColor" stroke-width="1.6"/></svg>"#,
        br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="10" cy="10" r="6" fill="none" stroke="currentColor" stroke-width="1.6"/><path d="m15 15 6 6" stroke="currentColor" stroke-width="1.6"/></svg>"#,
        br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="m8 4-5 3 2 5 3-1v10h8V11l3 1 2-5-5-3c0 4-8 4-8 0z" fill="none" stroke="currentColor" stroke-width="1.6"/></svg>"#,
        br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5" fill="none" stroke="currentColor" stroke-width="1.6"/></svg>"#,
        br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M5 3v18M12 3v18M19 3v18M2 8h6m1 8h6m1-9h6" fill="none" stroke="currentColor" stroke-width="1.6"/></svg>"#,
    ];
    SvgViewer::new(ICONS[page])
        .width(Size::px(23.))
        .height(Size::px(23.))
        .color((18, 85, 112))
}
pub const CHARACTERS: [&[u8]; 9] = [
    include_bytes!("../assets/character-0.svg"),
    include_bytes!("../assets/character-1.svg"),
    include_bytes!("../assets/character-2.svg"),
    include_bytes!("../assets/character-3.svg"),
    include_bytes!("../assets/character-4.svg"),
    include_bytes!("../assets/character-5.svg"),
    include_bytes!("../assets/character-6.svg"),
    include_bytes!("../assets/character-7.svg"),
    include_bytes!("../assets/character-8.svg"),
];
