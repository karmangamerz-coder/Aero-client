// SPDX-License-Identifier: MIT
//! Aero design tokens and the small custom control set shared by every page.
//!
//! Surfaces are flat translucent glass with a single hairline rim; gloss is kept to
//! a one-pixel highlight. Nothing here animates continuously, so idle rendering is
//! limited to actual input.
use freya::prelude::*;

pub const RAIL_WIDTH: f32 = 64.;
pub const TOP_BAR_HEIGHT: f32 = 56.;
pub const STATUS_HEIGHT: f32 = 26.;
pub const FONT: &str = "Segoe UI";

#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub photo: bool,
    pub bg: Color,
    pub glass: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub text: Color,
    pub text_dim: Color,
    pub border: Color,
    pub border_soft: Color,
    pub accent: Color,
    pub accent_soft: Color,
    pub accent_text: Color,
    pub green: Color,
    pub green_hover: Color,
    pub green_border: Color,
    pub danger: Color,
}

pub fn palette(name: &str) -> Palette {
    match name {
        "Aero Dark" => Palette {
            dark: true,
            photo: true,
            bg: Color::from_rgb(12, 37, 56),
            glass: Color::from_argb(238, 17, 46, 66),
            surface: Color::from_argb(190, 26, 64, 86),
            surface_alt: Color::from_argb(120, 46, 94, 118),
            text: Color::from_rgb(233, 246, 253),
            text_dim: Color::from_rgb(164, 203, 219),
            border: Color::from_rgb(58, 108, 133),
            border_soft: Color::from_rgb(42, 86, 108),
            accent: Color::from_rgb(126, 210, 241),
            accent_soft: Color::from_argb(220, 24, 104, 142),
            accent_text: Color::from_rgb(10, 34, 50),
            green: Color::from_rgb(60, 155, 66),
            green_hover: Color::from_rgb(76, 175, 78),
            green_border: Color::from_rgb(146, 214, 138),
            danger: Color::from_rgb(224, 126, 118),
        },
        "Aero Lite" => Palette {
            dark: false,
            photo: false,
            bg: Color::from_rgb(243, 245, 247),
            glass: Color::from_rgb(255, 255, 255),
            surface: Color::from_rgb(255, 255, 255),
            surface_alt: Color::from_rgb(236, 241, 244),
            text: Color::from_rgb(29, 41, 50),
            text_dim: Color::from_rgb(88, 101, 110),
            border: Color::from_rgb(199, 207, 213),
            border_soft: Color::from_rgb(222, 228, 232),
            accent: Color::from_rgb(10, 111, 160),
            accent_soft: Color::from_rgb(214, 232, 242),
            accent_text: Color::WHITE,
            green: Color::from_rgb(43, 122, 51),
            green_hover: Color::from_rgb(52, 140, 60),
            green_border: Color::from_rgb(158, 190, 158),
            danger: Color::from_rgb(170, 44, 44),
        },
        _ => Palette {
            dark: false,
            photo: true,
            bg: Color::from_rgb(234, 248, 255),
            glass: Color::from_argb(236, 246, 252, 255),
            surface: Color::from_argb(196, 255, 255, 255),
            surface_alt: Color::from_argb(140, 214, 238, 250),
            text: Color::from_rgb(18, 60, 86),
            text_dim: Color::from_rgb(73, 105, 122),
            border: Color::from_rgb(168, 215, 232),
            border_soft: Color::from_rgb(204, 231, 241),
            accent: Color::from_rgb(8, 126, 186),
            accent_soft: Color::from_rgb(141, 216, 248),
            accent_text: Color::WHITE,
            green: Color::from_rgb(46, 140, 52),
            green_hover: Color::from_rgb(62, 166, 64),
            green_border: Color::from_rgb(151, 204, 137),
            danger: Color::from_rgb(176, 48, 48),
        },
    }
}

/// Global component theme. Only the tokens Freya's built-in widgets need are set;
/// every Aero surface paints its own colours from [`Palette`].
pub fn theme(name: &str) -> Theme {
    let p = palette(name);
    let mut t = if p.dark { dark_theme() } else { light_theme() };
    t.name = match name {
        "Aero Dark" => "Aero Dark",
        "Aero Lite" => "Aero Lite",
        _ => "Aero Light",
    };
    let c = &mut t.colors;
    c.primary = p.accent;
    c.secondary = p.accent_soft;
    c.tertiary = p.green;
    c.background = p.bg;
    c.surface_primary = p.surface;
    c.surface_secondary = p.surface_alt;
    c.surface_tertiary = p.glass;
    c.text_primary = p.text;
    c.text_secondary = p.text_dim;
    c.text_placeholder = p.text_dim;
    c.border = p.border;
    c.border_focus = p.accent;
    c.focus = p.accent;
    c.active = p.accent_soft;
    c.success = p.green;
    c.error = p.danger;
    t
}

pub fn text(value: impl Into<String>, size: f32, color: Color) -> impl IntoElement {
    label()
        .text(value.into())
        .font_size(size)
        .font_family(FONT)
        .color(color)
}

pub fn strong(value: impl Into<String>, size: f32, color: Color) -> impl IntoElement {
    label()
        .text(value.into())
        .font_size(size)
        .font_family(FONT)
        .font_weight(FontWeight::BOLD)
        .color(color)
}

pub fn dim(value: impl Into<String>, size: f32, p: Palette) -> impl IntoElement {
    text(value, size, p.text_dim)
}

pub fn rim(p: Palette) -> Border {
    Border::new().width(1.).fill(p.border)
}

pub fn hairline(p: Palette) -> Rect {
    rect()
        .width(Size::fill())
        .height(Size::px(1.))
        .background(p.border_soft)
}

/// Standard opaque-ish glass surface used for grouped content.
pub fn panel(p: Palette) -> Rect {
    rect()
        .padding((14., 16.))
        .spacing(10.)
        .corner_radius(8.)
        .background(p.glass)
        .border(rim(p))
}

pub fn section(p: Palette, title: &str, hint: &str) -> Rect {
    rect()
        .spacing(4.)
        .child(strong(title.to_owned(), 15., p.text))
        .child(dim(hint.to_owned(), 11.5, p))
}

pub fn chip(value: impl Into<String>, p: Palette, active: bool) -> Rect {
    rect()
        .padding((3., 8.))
        .corner_radius(4.)
        .background(if active { p.accent_soft } else { p.surface_alt })
        .border(
            Border::new()
                .width(1.)
                .fill(if active { p.accent } else { p.border_soft }),
        )
        .child(text(
            value.into(),
            10.5,
            if active { p.accent_text } else { p.text_dim },
        ))
}

pub fn status_chip(status: &str, p: Palette) -> Rect {
    let (label, ok) = match status {
        "READY" => ("Ready", true),
        "INSTALLING" => ("Installing", false),
        "INTERRUPTED" => ("Interrupted", false),
        "CANCELLED" => ("Cancelled", false),
        "FAILED" => ("Needs repair", false),
        _ => ("Not installed", false),
    };
    chip(label, p, ok)
}

fn base(label: impl Into<String>, p: Palette, fg: Color) -> Button {
    Button::new()
        .theme_layout(
            ButtonLayoutThemePartial::default()
                .corner_radius(5.)
                .padding((0., 14.))
                .height(Size::px(32.)),
        )
        .theme_colors(
            ButtonColorsThemePartial::default()
                .color(fg)
                .background(p.surface)
                .hover_background(p.accent_soft)
                .border_fill(p.border)
                .focus_border_fill(p.accent),
        )
        .child(text(label.into(), 12.5, fg))
}

/// Quiet secondary action: glass fill, hairline border.
pub fn action(label: impl Into<String>, p: Palette) -> Button {
    base(label, p, p.text)
}

/// The single glossy green call to action.
pub fn primary(label: impl Into<String>, p: Palette) -> Button {
    Button::new()
        .theme_layout(
            ButtonLayoutThemePartial::default()
                .corner_radius(5.)
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
        .child(strong(label.into(), 14., Color::WHITE))
}

/// Text-only action for low-emphasis affordances.
pub fn ghost(label: impl Into<String>, p: Palette) -> Button {
    Button::new()
        .theme_layout(
            ButtonLayoutThemePartial::default()
                .corner_radius(5.)
                .padding((0., 10.))
                .height(Size::px(28.)),
        )
        .theme_colors(
            ButtonColorsThemePartial::default()
                .color(p.accent)
                .background(Color::TRANSPARENT)
                .hover_background(p.surface_alt)
                .border_fill(Color::TRANSPARENT)
                .focus_border_fill(p.accent),
        )
        .child(text(label.into(), 12., p.accent))
}

pub fn danger(label: impl Into<String>, p: Palette) -> Button {
    base(label, p, p.danger)
}

pub fn rail_button(index: usize, selected: bool, p: Palette) -> Button {
    Button::new()
        .theme_layout(
            ButtonLayoutThemePartial::default()
                .corner_radius(6.)
                .padding(10.)
                .width(Size::px(44.))
                .height(Size::px(38.)),
        )
        .theme_colors(
            ButtonColorsThemePartial::default()
                .background(if selected {
                    p.accent_soft
                } else {
                    Color::TRANSPARENT
                })
                .hover_background(if selected { p.accent_soft } else { p.surface })
                .border_fill(Color::TRANSPARENT)
                .focus_border_fill(p.accent),
        )
        .child(icon(index, p, selected))
}

pub fn icon(index: usize, p: Palette, selected: bool) -> SvgViewer {
    const ICONS: [&[u8]; 5] = [
        br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="m8 5 11 7-11 7z" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round"/></svg>"#,
        br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M4 4h6v6H4zm10 0h6v6h-6zM4 14h6v6H4zm10 0h6v6h-6z" fill="none" stroke="currentColor" stroke-width="1.6"/></svg>"#,
        br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="10" cy="10" r="6" fill="none" stroke="currentColor" stroke-width="1.6"/><path d="m15 15 6 6" stroke="currentColor" stroke-width="1.6"/></svg>"#,
        br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5" fill="none" stroke="currentColor" stroke-width="1.6"/></svg>"#,
        br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M5 3v18M12 3v18M19 3v18M2 8h6m1 8h6m1-9h6" fill="none" stroke="currentColor" stroke-width="1.6"/></svg>"#,
    ];
    SvgViewer::new(ICONS[index.min(ICONS.len() - 1)])
        .width(Size::px(19.))
        .height(Size::px(19.))
        .color(if selected { p.accent } else { p.text_dim })
}

pub const NAV: [(&str, &str); 5] = [
    ("Play", "Home"),
    ("Instances", "Library"),
    ("Discover", "Browse Modrinth"),
    ("Downloads", "Activity"),
    ("Settings", "Preferences"),
];

pub fn progress(percent: f32, p: Palette) -> Rect {
    let percent = percent.clamp(0., 100.);
    rect()
        .width(Size::fill())
        .height(Size::px(7.))
        .corner_radius(4.)
        .background(p.surface_alt)
        .border(Border::new().width(1.).fill(p.border_soft))
        .a11y_role(AccessibilityRole::ProgressIndicator)
        .a11y_alt(format!("{percent:.0}%"))
        .child(
            rect()
                .width(Size::percent(percent))
                .height(Size::fill())
                .corner_radius(3.)
                .background(p.green),
        )
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

pub fn landscape() -> SvgViewer {
    SvgViewer::new(include_bytes!("../assets/landscape.svg"))
        .position(Position::new_absolute())
        .width(Size::fill())
        .height(Size::fill())
        .aspect_ratio(AspectRatio::Max)
}

pub fn logo(size: f32) -> SvgViewer {
    SvgViewer::new(include_bytes!("../assets/logo.svg"))
        .width(Size::px(size))
        .height(Size::px(size))
}
