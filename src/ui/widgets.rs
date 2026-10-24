use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Margin, Painter, Rect, Response, Sense, Shape,
    Stroke, StrokeKind, Ui, pos2, vec2,
};

use crate::theme;

use super::icons;

pub type IconDraw = fn(&Painter, Rect, Color32, f32);

pub fn inset_frame() -> Frame {
    Frame::NONE
        .fill(Color32::from_rgb(0x0D, 0x14, 0x1D))
        .stroke(Stroke::new(1.0, theme::STROKE_SOFT))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(14))
}

#[allow(clippy::too_many_arguments)]
pub fn action_button(
    ui: &mut Ui,
    width: f32,
    label: &str,
    shortcut: &str,
    color: Color32,
    icon: IconDraw,
    enabled: bool,
    tooltip: &str,
) -> Response {
    let height = 58.0;
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), sense);
    let hover = if enabled {
        ui.ctx()
            .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.12)
    } else {
        0.0
    };
    let pressed = enabled && response.is_pointer_button_down_on();
    let painter = ui.painter();

    let fill_alpha = if !enabled {
        10
    } else if pressed {
        38
    } else {
        (16.0 + hover * 20.0) as u8
    };
    let fill = theme::alpha(color, fill_alpha);
    let border = theme::alpha(
        color,
        if enabled {
            55 + (hover * 55.0) as u8
        } else {
            25
        },
    );
    painter.rect_filled(
        rect,
        CornerRadius::same(16),
        if enabled {
            theme::mix(theme::CARD, fill, 0.6)
        } else {
            Color32::from_rgb(0x12, 0x19, 0x22)
        },
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(16),
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );

    if enabled && hover > 0.0 {
        let glow = Rect::from_min_max(
            rect.left_top() + vec2(0.0, 1.0),
            pos2(rect.right(), rect.top() + 2.0 + hover * 2.0),
        );
        painter.rect_filled(
            glow,
            CornerRadius::same(2),
            theme::alpha(color, (hover * 45.0) as u8),
        );
    }

    let icon_rect =
        Rect::from_center_size(pos2(rect.left() + 38.0, rect.center().y), vec2(36.0, 36.0));
    painter.rect_filled(
        icon_rect,
        CornerRadius::same(11),
        theme::alpha(
            color,
            if enabled {
                30 + (hover * 20.0) as u8
            } else {
                12
            },
        ),
    );
    icon(
        painter,
        icon_rect.shrink(9.0),
        if enabled { color } else { theme::FAINT },
        2.0,
    );

    painter.text(
        pos2(rect.left() + 68.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        theme::semibold(17.0),
        if enabled { theme::TEXT } else { theme::FAINT },
    );

    if !shortcut.is_empty() {
        let key_width = 24.0 + (shortcut.chars().count() as f32) * 3.0;
        let key_rect = Rect::from_center_size(
            pos2(rect.right() - 18.0 - key_width * 0.5, rect.center().y),
            vec2(key_width, 22.0),
        );
        painter.rect_filled(
            key_rect,
            CornerRadius::same(6),
            if enabled {
                theme::alpha(theme::TEXT, 12)
            } else {
                Color32::from_rgb(0x16, 0x1D, 0x26)
            },
        );
        painter.rect_stroke(
            key_rect,
            CornerRadius::same(6),
            Stroke::new(
                1.0,
                if enabled {
                    theme::alpha(theme::TEXT, 20)
                } else {
                    theme::STROKE_SOFT
                },
            ),
            StrokeKind::Inside,
        );
        painter.text(
            key_rect.center(),
            Align2::CENTER_CENTER,
            shortcut,
            theme::mono(11.5),
            if enabled { theme::MUTED } else { theme::FAINT },
        );
    }

    if enabled && response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.0),
            CornerRadius::same(18),
            Stroke::new(1.0, theme::alpha(color, 120)),
            StrokeKind::Inside,
        );
    }

    if response.hovered() && !tooltip.is_empty() {
        response.on_hover_text(tooltip)
    } else {
        response
    }
}

pub fn secondary_button(
    ui: &mut Ui,
    label: &str,
    icon: Option<IconDraw>,
    color: Color32,
    tooltip: &str,
) -> Response {
    let font = theme::semibold(13.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), color);
    let icon_space = if icon.is_some() { 24.0 } else { 0.0 };
    let width = galley.size().x + icon_space + 30.0;
    let (rect, response) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.1);
    let painter = ui.painter();

    painter.rect_filled(
        rect,
        CornerRadius::same(10),
        theme::mix(theme::PANEL_RAISED, theme::CARD_HOVER, hover),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0, theme::mix(theme::STROKE_SOFT, theme::STROKE, hover)),
        StrokeKind::Inside,
    );

    let mut text_x = rect.left() + 15.0;
    if let Some(icon) = icon {
        icon(
            painter,
            Rect::from_center_size(pos2(text_x + 7.0, rect.center().y), vec2(16.0, 16.0)),
            color,
            1.6,
        );
        text_x += 24.0;
    }
    painter.text(
        pos2(text_x, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        color,
    );

    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    if tooltip.is_empty() {
        response
    } else {
        response.on_hover_text(tooltip)
    }
}

pub fn primary_button(
    ui: &mut Ui,
    label: &str,
    icon: Option<IconDraw>,
    min_width: f32,
) -> Response {
    let font = theme::semibold(15.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), theme::BG);
    let icon_space = if icon.is_some() { 26.0 } else { 0.0 };
    let width = (galley.size().x + icon_space + 42.0).max(min_width);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 48.0), Sense::click());
    let hover =
        ui.ctx()
            .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.12);
    let pressed = response.is_pointer_button_down_on();
    let fill = if pressed {
        theme::mix(theme::ACCENT, theme::KEEP, 0.4)
    } else {
        theme::mix(theme::ACCENT, Color32::WHITE, hover * 0.12)
    };
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(14), fill);
    if hover > 0.0 {
        painter.rect_stroke(
            rect.expand(3.0 * hover),
            CornerRadius::same(17),
            Stroke::new(1.5, theme::alpha(theme::ACCENT, (70.0 * hover) as u8)),
            StrokeKind::Inside,
        );
    }

    let mut text_x = rect.center().x - galley.size().x * 0.5 + icon_space * 0.5;
    if let Some(icon) = icon {
        icon(
            painter,
            Rect::from_center_size(pos2(text_x - 14.0, rect.center().y), vec2(18.0, 18.0)),
            theme::BG,
            2.0,
        );
        text_x += 6.0;
    }
    painter.text(
        pos2(text_x, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        theme::BG,
    );

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn ghost_button(ui: &mut Ui, label: &str, color: Color32) -> Response {
    let font = theme::semibold(13.0);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), color);
    let (rect, response) =
        ui.allocate_exact_size(vec2(galley.size().x + 16.0, 30.0), Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.1);
    let painter = ui.painter();
    if hover > 0.0 {
        painter.rect_filled(
            rect,
            CornerRadius::same(8),
            theme::alpha(color, (18.0 * hover) as u8),
        );
    }
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        font,
        theme::mix(color, theme::TEXT, hover * 0.4),
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn chip(ui: &mut Ui, label: &str, color: Color32) -> Response {
    let font = theme::semibold(11.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), color);
    let width = galley.size().x + 26.0;
    let (rect, response) = ui.allocate_exact_size(vec2(width, 24.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(12), theme::alpha(color, 20));
    painter.rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(1.0, theme::alpha(color, 45)),
        StrokeKind::Inside,
    );
    painter.circle_filled(pos2(rect.left() + 11.0, rect.center().y), 2.5, color);
    painter.text(
        pos2(rect.left() + 18.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font,
        theme::mix(color, theme::TEXT, 0.35),
    );
    response
}

pub fn plain_chip(ui: &mut Ui, label: &str, color: Color32) -> Response {
    let font = theme::semibold(11.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), color);
    let (rect, response) =
        ui.allocate_exact_size(vec2(galley.size().x + 20.0, 24.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(8), theme::alpha(color, 16));
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, color);
    response
}

pub fn progress_bar(ui: &mut Ui, value: f32, height: f32) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter();
    let radius = CornerRadius::same((height * 0.5) as u8);
    painter.rect_filled(rect, radius, Color32::from_rgb(0x1A, 0x24, 0x31));
    let value = value.clamp(0.0, 1.0);
    if value > 0.001 {
        let filled = Rect::from_min_size(
            rect.min,
            vec2((rect.width() * value).max(height), rect.height()),
        );
        painter.rect_filled(
            filled,
            radius,
            theme::mix(theme::ACCENT, theme::KEEP, (value * 0.5).min(0.5)),
        );
        let highlight = Rect::from_min_size(
            filled.min,
            vec2(filled.width(), (filled.height() * 0.45).max(1.0)),
        );
        painter.rect_filled(highlight, radius, theme::alpha(Color32::WHITE, 22));
    }
}

pub fn key_hint(ui: &mut Ui, key: &str) {
    let font = theme::mono(10.5);
    let width = 10.0 + key.chars().count() as f32 * 6.2;
    let (rect, _) = ui.allocate_exact_size(vec2(width, 18.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(5), theme::alpha(theme::TEXT, 10));
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        key,
        font,
        theme::MUTED,
    );
}

pub fn logo_badge(ui: &mut Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    icons::logo(ui.painter(), rect);
}

pub fn page<R>(ui: &mut Ui, max_width: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let available = ui.available_size();
    let width = available.x.min(max_width);
    let side = ((available.x - width) * 0.5).max(0.0);
    ui.horizontal_top(|ui| {
        ui.add_space(side);
        ui.allocate_ui_with_layout(
            vec2(width, available.y),
            egui::Layout::top_down(egui::Align::Min),
            add,
        )
        .inner
    })
    .inner
}

pub fn paint_background(ctx: &egui::Context) {
    let rect = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::background());
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), Color32::from_rgb(0x09, 0x0D, 0x12));
    mesh.colored_vertex(rect.right_top(), Color32::from_rgb(0x0B, 0x11, 0x19));
    mesh.colored_vertex(rect.right_bottom(), Color32::from_rgb(0x0C, 0x14, 0x1D));
    mesh.colored_vertex(rect.left_bottom(), Color32::from_rgb(0x0A, 0x0F, 0x16));
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(Shape::mesh(mesh));

    let glow_center = pos2(
        rect.right() - rect.width() * 0.08,
        rect.top() + rect.height() * 0.02,
    );
    painter.circle_filled(
        glow_center,
        rect.width() * 0.32,
        Color32::from_rgba_unmultiplied(0x2D, 0xD4, 0xBF, 6),
    );
}

pub fn toast_frame(color: Color32) -> Frame {
    Frame::NONE
        .fill(Color32::from_rgb(0x12, 0x1A, 0x24))
        .stroke(Stroke::new(1.0, theme::alpha(color, 80)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(14, 10))
}

pub fn stat_block(ui: &mut Ui, label: &str, value: &str, color: Color32, width: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 70.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(14), theme::alpha(color, 10));
    painter.rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0, theme::alpha(color, 34)),
        StrokeKind::Inside,
    );
    painter.circle_filled(pos2(rect.left() + 16.0, rect.top() + 20.0), 3.0, color);
    painter.text(
        pos2(rect.left() + 24.0, rect.top() + 20.0),
        Align2::LEFT_CENTER,
        label,
        theme::semibold(12.0),
        theme::MUTED,
    );
    painter.text(
        pos2(rect.left() + 16.0, rect.top() + 47.0),
        Align2::LEFT_CENTER,
        value,
        theme::semibold(20.0),
        theme::TEXT,
    );
    response
}

pub fn empty_state(ui: &mut Ui, title: &str, message: &str) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), 190.0), Sense::hover());
    let painter = ui.painter();
    painter.text(
        pos2(rect.center().x, rect.center().y - 16.0),
        Align2::CENTER_CENTER,
        title,
        theme::semibold(19.0),
        theme::TEXT_SOFT,
    );
    painter.text(
        pos2(rect.center().x, rect.center().y + 16.0),
        Align2::CENTER_CENTER,
        message,
        FontId::proportional(13.5),
        theme::MUTED,
    );
    response
}

pub fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0, theme::STROKE_SOFT);
}
