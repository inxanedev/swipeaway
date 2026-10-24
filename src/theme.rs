use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke,
    TextStyle, Visuals,
};

pub const BG: Color32 = Color32::from_rgb(0x0B, 0x0F, 0x14);
pub const PANEL: Color32 = Color32::from_rgb(0x10, 0x17, 0x20);
pub const PANEL_RAISED: Color32 = Color32::from_rgb(0x14, 0x1C, 0x26);
pub const CARD: Color32 = Color32::from_rgb(0x15, 0x1E, 0x29);
pub const CARD_HOVER: Color32 = Color32::from_rgb(0x1B, 0x26, 0x34);
pub const STROKE: Color32 = Color32::from_rgb(0x27, 0x34, 0x49);
pub const STROKE_SOFT: Color32 = Color32::from_rgb(0x1C, 0x27, 0x36);
pub const TEXT: Color32 = Color32::from_rgb(0xE6, 0xED, 0xF7);
pub const TEXT_SOFT: Color32 = Color32::from_rgb(0xC3, 0xCD, 0xDD);
pub const MUTED: Color32 = Color32::from_rgb(0x8B, 0x9A, 0xAF);
pub const FAINT: Color32 = Color32::from_rgb(0x5D, 0x6C, 0x80);
pub const ACCENT: Color32 = Color32::from_rgb(0x5E, 0xEA, 0xD4);
pub const ACCENT_DARK: Color32 = Color32::from_rgb(0x2D, 0xD4, 0xBF);
pub const KEEP: Color32 = Color32::from_rgb(0x34, 0xD3, 0x99);
pub const DELETE: Color32 = Color32::from_rgb(0xF8, 0x71, 0x71);
pub const MANUAL: Color32 = Color32::from_rgb(0xFB, 0xBF, 0x24);
pub const BLUE: Color32 = Color32::from_rgb(0x60, 0xA5, 0xFA);
pub const PURPLE: Color32 = Color32::from_rgb(0xA7, 0x8B, 0xFA);

pub const SEMIBOLD: &str = "inter-semibold";
pub const BOLD: &str = "inter-bold";

pub fn apply(ctx: &egui::Context) {
    install_fonts(ctx);
    install_style(ctx);
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "inter-regular".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "inter-medium".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-Medium.ttf"
        ))),
    );
    fonts.font_data.insert(
        SEMIBOLD.to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-SemiBold.ttf"
        ))),
    );
    fonts.font_data.insert(
        BOLD.to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-Bold.ttf"
        ))),
    );
    fonts.font_data.insert(
        "jetbrains-mono".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/JetBrainsMono-Variable.ttf"
        ))),
    );

    let proportional_fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_else(|| vec!["Ubuntu-Light".to_owned()]);
    let monospace_fallback = fonts
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();

    let mut proportional = vec!["inter-regular".to_owned(), "inter-medium".to_owned()];
    proportional.extend(proportional_fallback.clone());
    fonts
        .families
        .insert(FontFamily::Proportional, proportional);

    let mut semibold = vec![SEMIBOLD.to_owned(), "inter-medium".to_owned()];
    semibold.extend(proportional_fallback.clone());
    fonts
        .families
        .insert(FontFamily::Name(SEMIBOLD.into()), semibold);

    let mut bold = vec![BOLD.to_owned(), SEMIBOLD.to_owned()];
    bold.extend(proportional_fallback);
    fonts.families.insert(FontFamily::Name(BOLD.into()), bold);

    let mut monospace = vec!["jetbrains-mono".to_owned()];
    monospace.extend(monospace_fallback);
    fonts.families.insert(FontFamily::Monospace, monospace);

    ctx.set_fonts(fonts);
}

fn install_style(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();

    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = Color32::from_rgb(0x0D, 0x13, 0x1B);
    visuals.faint_bg_color = PANEL_RAISED;
    visuals.override_text_color = Some(TEXT);
    visuals.window_corner_radius = CornerRadius::same(16);
    visuals.window_stroke = Stroke::new(1.0, STROKE_SOFT);
    visuals.menu_corner_radius = CornerRadius::same(12);
    visuals.hyperlink_color = ACCENT;
    visuals.warn_fg_color = MANUAL;
    visuals.error_fg_color = DELETE;
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0x5E, 0xEA, 0xD4, 48);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.striped = false;
    visuals.button_frame = true;
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

    visuals.widgets.noninteractive.bg_fill = PANEL_RAISED;
    visuals.widgets.noninteractive.weak_bg_fill = PANEL_RAISED;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, STROKE_SOFT);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(10);

    visuals.widgets.inactive.bg_fill = PANEL_RAISED;
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x18, 0x21, 0x2D);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, STROKE_SOFT);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SOFT);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(10);

    visuals.widgets.hovered.bg_fill = CARD_HOVER;
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x1E, 0x2A, 0x3A);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, STROKE);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(10);
    visuals.widgets.hovered.expansion = 1.0;

    visuals.widgets.active.bg_fill = Color32::from_rgb(0x22, 0x2F, 0x41);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(0x22, 0x2F, 0x41);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_DARK);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.active.corner_radius = CornerRadius::same(10);

    visuals.widgets.open.bg_fill = CARD_HOVER;
    visuals.widgets.open.weak_bg_fill = CARD_HOVER;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, STROKE);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.open.corner_radius = CornerRadius::same(10);

    visuals.text_options.font_hinting = false;
    visuals.text_options.subpixel_binning = true;
    visuals.text_options.color_transfer_function =
        egui::epaint::FontColorTransferFunction::Gamma(0.75);

    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.window_margin = Margin::same(0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.interact_size = egui::vec2(40.0, 38.0);
    style.spacing.scroll = egui::style::ScrollStyle::floating();
    style.spacing.scroll.bar_width = 10.0;
    style.spacing.scroll.handle_min_length = 28.0;
    style.spacing.scroll.floating_allocated_width = 0.0;
    style.spacing.scroll.active_handle_opacity = 0.55;
    style.spacing.scroll.interact_handle_opacity = 0.85;
    style.spacing.scroll.active_background_opacity = 0.15;
    style.spacing.scroll.interact_background_opacity = 0.25;
    style.animation_time = 0.14;

    style.text_styles = [
        (TextStyle::Small, FontId::proportional(12.5)),
        (TextStyle::Body, FontId::proportional(15.5)),
        (TextStyle::Button, FontId::proportional(15.5)),
        (TextStyle::Monospace, FontId::monospace(14.0)),
        (
            TextStyle::Heading,
            FontId::new(26.0, FontFamily::Name(SEMIBOLD.into())),
        ),
    ]
    .into();
    style.spacing.extra_text_line_spacing = 0.5;

    ctx.set_style_of(egui::Theme::Dark, style.clone());
    ctx.set_style_of(egui::Theme::Light, style);
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

pub fn alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 * (1.0 - t) + b.r() as f32 * t) as u8,
        (a.g() as f32 * (1.0 - t) + b.g() as f32 * t) as u8,
        (a.b() as f32 * (1.0 - t) + b.b() as f32 * t) as u8,
    )
}
