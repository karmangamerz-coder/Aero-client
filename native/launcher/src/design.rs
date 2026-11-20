// SPDX-License-Identifier: MIT
//! Aero design system: tokens plus the small set of custom controls used by every
//! screen. Surfaces are translucent blue glass with one hairline rim and a single
//! pixel of gloss; nothing animates continuously, so the window only repaints on
//! input. Typography is Segoe UI throughout and icons are outline SVGs drawn here.
use bytes::Bytes;
use freya::prelude::*;

pub const RAIL_WIDTH: f32 = 58.;
pub const TOP_BAR_HEIGHT: f32 = 52.;
pub const STATUS_HEIGHT: f32 = 24.;
pub const FONT: &str = "Segoe UI";

#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub photo: bool,
    pub bg: Color,
    pub glass: Color,
    pub glass_soft: Color,
    pub rail: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub text: Color,
    pub text_dim: Color,
    pub art_text: Color,
    pub art_dim: Color,
    pub border: Color,
    pub border_soft: Color,
    pub accent: Color,
    pub accent_soft: Color,
    pub accent_text: Color,
    pub green: Color,
    pub green_hover: Color,
    pub green_border: Color,
    pub green_deep: Color,
    pub danger: Color,
}

pub fn palette(name: &str) -> Palette {
    match name {
        "Aero Dark" => Palette {
            dark: true,
            photo: true,
            bg: Color::from_rgb(10, 32, 50),
            glass: Color::from_argb(205, 18, 48, 68),
            glass_soft: Color::from_argb(96, 40, 90, 120),
            rail: Color::from_argb(150, 14, 42, 62),
            surface: Color::from_argb(196, 26, 64, 86),
            surface_alt: Color::from_argb(110, 44, 90, 116),
            text: Color::from_rgb(232, 246, 253),
            text_dim: Color::from_rgb(158, 198, 216),
            art_text: Color::from_rgb(240, 250, 255),
            art_dim: Color::from_rgb(205, 230, 242),
            border: Color::from_rgb(56, 106, 132),
            border_soft: Color::from_rgb(38, 80, 102),
            accent: Color::from_rgb(126, 212, 241),
            accent_soft: Color::from_argb(200, 24, 104, 142),
            accent_text: Color::from_rgb(8, 32, 48),
            green: Color::from_rgb(58, 150, 62),
            green_hover: Color::from_rgb(76, 176, 78),
            green_border: Color::from_rgb(140, 210, 132),
            green_deep: Color::from_rgb(38, 108, 44),
            danger: Color::from_rgb(224, 126, 118),
        },
        "Aero Lite" => Palette {
            dark: false,
            photo: false,
            bg: Color::from_rgb(243, 246, 248),
            glass: Color::from_rgb(255, 255, 255),
            glass_soft: Color::from_rgb(250, 252, 253),
            rail: Color::from_rgb(248, 250, 251),
            surface: Color::from_rgb(255, 255, 255),
            surface_alt: Color::from_rgb(236, 241, 244),
            text: Color::from_rgb(29, 41, 50),
            text_dim: Color::from_rgb(88, 101, 110),
            art_text: Color::from_rgb(29, 41, 50),
            art_dim: Color::from_rgb(88, 101, 110),
            border: Color::from_rgb(199, 207, 213),
            border_soft: Color::from_rgb(222, 228, 232),
            accent: Color::from_rgb(10, 111, 160),
            accent_soft: Color::from_rgb(214, 232, 242),
            accent_text: Color::WHITE,
            green: Color::from_rgb(43, 122, 51),
            green_hover: Color::from_rgb(52, 140, 60),
            green_border: Color::from_rgb(158, 190, 158),
            green_deep: Color::from_rgb(32, 96, 40),
            danger: Color::from_rgb(170, 44, 44),
        },
        _ => Palette {
            dark: false,
            photo: true,
            bg: Color::from_rgb(235, 248, 255),
            glass: Color::from_argb(228, 250, 253, 255),
            glass_soft: Color::from_argb(120, 255, 255, 255),
            rail: Color::from_argb(120, 232, 248, 254),
            surface: Color::from_argb(198, 255, 255, 255),
            surface_alt: Color::from_argb(120, 214, 238, 250),
            text: Color::from_rgb(16, 58, 84),
            text_dim: Color::from_rgb(70, 104, 124),
            art_text: Color::from_rgb(14, 52, 78),
            art_dim: Color::from_rgb(34, 76, 98),
            border: Color::from_rgb(150, 204, 224),
            border_soft: Color::from_rgb(196, 226, 238),
            accent: Color::from_rgb(8, 126, 186),
            accent_soft: Color::from_argb(96, 122, 210, 240),
            accent_text: Color::WHITE,
            green: Color::from_rgb(70, 168, 54),
            green_hover: Color::from_rgb(88, 190, 64),
            green_border: Color::from_rgb(158, 224, 132),
            green_deep: Color::from_rgb(46, 132, 36),
            danger: Color::from_rgb(176, 48, 48),
        },
    }
}

/// Global component theme so built-in widgets (Select, Input, ScrollView) adopt
/// the Aero palette instead of Freya's defaults.
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

pub fn art(value: impl Into<String>, size: f32, p: Palette) -> impl IntoElement {
    strong(value, size, p.art_text)
}

pub fn art_dim(value: impl Into<String>, size: f32, p: Palette) -> impl IntoElement {
    text(value, size, p.art_dim)
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

pub fn panel(p: Palette) -> Rect {
    rect()
        .padding((14., 16.))
        .spacing(9.)
        .corner_radius(10.)
        .background(p.glass)
        .border(rim(p))
}

pub fn card(p: Palette) -> Rect {
    rect()
        .padding((12., 14.))
        .spacing(8.)
        .corner_radius(8.)
        .background(p.surface)
        .border(Border::new().width(1.).fill(p.border_soft))
}

pub fn section(p: Palette, title: &str, hint: &str) -> Rect {
    rect()
        .spacing(3.)
        .child(strong(title.to_owned(), 15., p.text))
        .child(dim(hint.to_owned(), 11.5, p))
}

pub fn chip(value: impl Into<String>, p: Palette, active: bool) -> Rect {
    rect()
        .padding((3., 8.))
        .corner_radius(5.)
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
                .corner_radius(6.)
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

pub fn action(label: impl Into<String>, p: Palette) -> Button {
    base(label, p, p.text)
}

pub fn ghost(label: impl Into<String>, p: Palette) -> Button {
    Button::new()
        .theme_layout(
            ButtonLayoutThemePartial::default()
                .corner_radius(6.)
                .padding((0., 9.))
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

/// The single glossy green call to action.
pub fn launch_button(p: Palette, label: &str, enabled: bool) -> Button {
    let content = rect()
        .horizontal()
        .spacing(10.)
        .cross_align(Alignment::Center)
        .child(
            rect()
                .width(Size::px(18.))
                .height(Size::px(18.))
                .center()
                .child(
                    SvgViewer::new(PLAY_ICON)
                        .width(Size::px(16.))
                        .height(Size::px(16.))
                        .color(Color::WHITE),
                ),
        )
        .child(strong(label.to_owned(), 17., Color::WHITE));
    Button::new()
        .theme_layout(
            ButtonLayoutThemePartial::default()
                .corner_radius(10.)
                .padding((0., 30.))
                .height(Size::px(54.))
                .width(Size::px(270.)),
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
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .center()
                .child(content)
                .child(
                    rect()
                        .position(Position::new_absolute())
                        .width(Size::fill())
                        .height(Size::px(1.))
                        .background(Color::from_argb(70, 255, 255, 255)),
                )
                .child(
                    rect()
                        .position(Position::new_absolute())
                        .width(Size::fill())
                        .height(Size::px(3.))
                        .background(p.green_deep),
                ),
        )
}

/// Export progress replaces the launch action while files are being verified.
pub fn launch_progress(p: Palette, percent: f32, label: &str) -> Rect {
    let percent = percent.clamp(0., 100.);
    rect()
        .width(Size::px(270.))
        .height(Size::px(54.))
        .corner_radius(10.)
        .background(p.surface_alt)
        .border(Border::new().width(1.).fill(p.green_border))
        .a11y_role(AccessibilityRole::ProgressIndicator)
        .a11y_alt(format!("{percent:.0} percent"))
        .child(
            rect()
                .position(Position::new_absolute())
                .width(Size::percent(percent))
                .height(Size::fill())
                .corner_radius(9.)
                .background(Color::from_argb(180, 70, 168, 54)),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .center()
                .child(strong(format!("{label} · {percent:.0}%"), 14., p.text)),
        )
}

/// Square glass button used for the instance settings action next to Launch.
pub fn gear_button(p: Palette) -> Button {
    Button::new()
        .theme_layout(
            ButtonLayoutThemePartial::default()
                .corner_radius(10.)
                .padding(0.)
                .width(Size::px(54.))
                .height(Size::px(54.)),
        )
        .theme_colors(
            ButtonColorsThemePartial::default()
                .background(p.glass)
                .hover_background(p.surface)
                .border_fill(p.border)
                .focus_border_fill(p.accent),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .center()
                .child(
                    SvgViewer::new(GEAR_ICON)
                        .width(Size::px(20.))
                        .height(Size::px(20.))
                        .color(p.text),
                ),
        )
}

pub fn nav_button(index: usize, selected: bool, p: Palette) -> Button {
    Button::new()
        .theme_layout(
            ButtonLayoutThemePartial::default()
                .corner_radius(9.)
                .padding(0.)
                .width(Size::px(42.))
                .height(Size::px(42.)),
        )
        .theme_colors(
            ButtonColorsThemePartial::default()
                .background(if selected {
                    p.accent_soft
                } else {
                    Color::TRANSPARENT
                })
                .hover_background(if selected {
                    p.accent_soft
                } else {
                    p.glass_soft
                })
                .border_fill(Color::TRANSPARENT)
                .focus_border_fill(p.accent),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .center()
                .child(icon(index, p, selected)),
        )
}

pub fn icon(index: usize, p: Palette, selected: bool) -> SvgViewer {
    SvgViewer::new(NAV_ICONS[index.min(NAV_ICONS.len() - 1)])
        .width(Size::px(19.))
        .height(Size::px(19.))
        .color(if selected { p.accent } else { p.text_dim })
}

pub const NAV: [(&str, &str); 5] = [
    ("Play", "Home"),
    ("Instances", "Library"),
    ("Discover", "Browse Modrinth"),
    ("Downloads", "Activity and logs"),
    ("Settings", "Preferences"),
];

const PLAY_ICON: &[u8] = br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="m8 5 11 7-11 7z" fill="currentColor"/></svg>"#;
const GEAR_ICON: &[u8] = br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="3.2" fill="none" stroke="currentColor" stroke-width="1.6"/><path d="M12 3.5v2.2M12 18.3v2.2M3.5 12h2.2M18.3 12h2.2M6 6l1.6 1.6M16.4 16.4 18 18M18 6l-1.6 1.6M7.6 16.4 6 18" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg>"#;

const NAV_ICONS: [&[u8]; 5] = [
    br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="m8 5 11 7-11 7z" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linejoin="round"/></svg>"#,
    br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M12 3.5 21 8l-9 4.5L3 8z" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/><path d="m3 12 9 4.5L21 12M3 16l9 4.5L21 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/></svg>"#,
    br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="8.5" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="m15.5 8.5-2.1 4.9-4.9 2.1 2.1-4.9z" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/></svg>"#,
    br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M12 3v11m-4.5-4.5L12 14l4.5-4.5M4 16v4.5h16V16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
    br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="3.2" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M12 3.4v2.3M12 18.3v2.3M3.4 12h2.3M18.3 12h2.3M5.9 5.9l1.6 1.6M16.5 16.5l1.6 1.6M18.1 5.9l-1.6 1.6M7.5 16.5l-1.6 1.6" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>"#,
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

/// Static original landscape: sky, hills and water, drawn once as an SVG and cached
/// by Skia. No live blur, no per-frame work.
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

/// Character preview: a composed PNG of the player's skin (or the Aero fallback).
/// The image is decoded once by Skia and never re-rendered while idle.
pub fn character(bytes: &std::sync::Arc<Vec<u8>>) -> ImageViewer {
    ImageViewer::new((bytes.len(), Bytes::from(bytes.as_ref().clone())))
        .width(Size::px(249.))
        .height(Size::px(408.))
        .a11y_alt("Minecraft character preview")
}
