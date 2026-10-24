use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2, pos2};

fn point(rect: Rect, x: f32, y: f32) -> Pos2 {
    pos2(
        rect.left() + rect.width() * x,
        rect.top() + rect.height() * y,
    )
}

fn poly(painter: &Painter, rect: Rect, points: &[(f32, f32)], color: Color32, width: f32) {
    let points: Vec<Pos2> = points.iter().map(|&(x, y)| point(rect, x, y)).collect();
    painter.add(Shape::line(points, Stroke::new(width, color)));
}

pub fn check(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    poly(
        painter,
        rect,
        &[(0.16, 0.52), (0.41, 0.76), (0.84, 0.24)],
        color,
        width,
    );
}

pub fn trash(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let stroke = Stroke::new(width, color);
    painter.line_segment([point(rect, 0.18, 0.28), point(rect, 0.82, 0.28)], stroke);
    painter.line_segment([point(rect, 0.4, 0.28), point(rect, 0.4, 0.17)], stroke);
    painter.line_segment([point(rect, 0.6, 0.28), point(rect, 0.6, 0.17)], stroke);
    painter.line_segment([point(rect, 0.4, 0.17), point(rect, 0.6, 0.17)], stroke);
    let body = Rect::from_min_max(point(rect, 0.26, 0.34), point(rect, 0.74, 0.85));
    painter.rect_stroke(
        body,
        (rect.width() * 0.08).round() as u8,
        stroke,
        StrokeKind::Inside,
    );
    painter.line_segment([point(rect, 0.41, 0.46), point(rect, 0.41, 0.73)], stroke);
    painter.line_segment([point(rect, 0.59, 0.46), point(rect, 0.59, 0.73)], stroke);
}

pub fn tray(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let stroke = Stroke::new(width, color);
    poly(
        painter,
        rect,
        &[(0.18, 0.5), (0.18, 0.78), (0.82, 0.78), (0.82, 0.5)],
        color,
        width,
    );
    painter.line_segment([point(rect, 0.5, 0.18), point(rect, 0.5, 0.57)], stroke);
    poly(
        painter,
        rect,
        &[(0.36, 0.45), (0.5, 0.59), (0.64, 0.45)],
        color,
        width,
    );
}

pub fn undo(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let stroke = Stroke::new(width, color);
    poly(
        painter,
        rect,
        &[
            (0.78, 0.72),
            (0.48, 0.72),
            (0.28, 0.56),
            (0.28, 0.38),
            (0.44, 0.25),
            (0.66, 0.25),
        ],
        color,
        width,
    );
    painter.line_segment([point(rect, 0.44, 0.25), point(rect, 0.57, 0.13)], stroke);
    painter.line_segment([point(rect, 0.44, 0.25), point(rect, 0.57, 0.38)], stroke);
}

pub fn folder(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let stroke = Stroke::new(width, color);
    let tab = vec![
        point(rect, 0.14, 0.36),
        point(rect, 0.14, 0.76),
        point(rect, 0.86, 0.76),
        point(rect, 0.86, 0.3),
        point(rect, 0.48, 0.3),
        point(rect, 0.4, 0.2),
        point(rect, 0.14, 0.2),
        point(rect, 0.14, 0.36),
    ];
    painter.add(Shape::line(tab, stroke));
}

pub fn open_external(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let stroke = Stroke::new(width, color);
    poly(
        painter,
        rect,
        &[(0.5, 0.2), (0.82, 0.2), (0.82, 0.52)],
        color,
        width,
    );
    painter.line_segment([point(rect, 0.8, 0.2), point(rect, 0.36, 0.64)], stroke);
    poly(
        painter,
        rect,
        &[(0.24, 0.42), (0.24, 0.8), (0.62, 0.8), (0.62, 0.6)],
        color,
        width,
    );
}

pub fn play(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let points = vec![
        point(rect, 0.34, 0.18),
        point(rect, 0.84, 0.5),
        point(rect, 0.34, 0.82),
    ];
    painter.add(Shape::convex_polygon(
        points,
        color,
        Stroke::new(width * 0.5, color),
    ));
}

pub fn warning(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let stroke = Stroke::new(width, color);
    poly(
        painter,
        rect,
        &[(0.5, 0.12), (0.9, 0.84), (0.1, 0.84), (0.5, 0.12)],
        color,
        width,
    );
    painter.line_segment([point(rect, 0.5, 0.38), point(rect, 0.5, 0.6)], stroke);
    painter.circle_filled(point(rect, 0.5, 0.71), width * 1.1, color);
}

pub fn clock(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    painter.circle_stroke(
        rect.center(),
        rect.width() * 0.36,
        Stroke::new(width, color),
    );
    painter.line_segment(
        [rect.center(), point(rect, 0.5, 0.28)],
        Stroke::new(width, color),
    );
    painter.line_segment(
        [rect.center(), point(rect, 0.68, 0.56)],
        Stroke::new(width, color),
    );
}

pub fn note(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    let stroke = Stroke::new(width, color);
    painter.circle_stroke(point(rect, 0.38, 0.72), rect.width() * 0.13, stroke);
    painter.circle_stroke(point(rect, 0.74, 0.62), rect.width() * 0.13, stroke);
    painter.line_segment([point(rect, 0.49, 0.72), point(rect, 0.49, 0.24)], stroke);
    painter.line_segment([point(rect, 0.85, 0.62), point(rect, 0.85, 0.18)], stroke);
    painter.line_segment([point(rect, 0.49, 0.24), point(rect, 0.85, 0.18)], stroke);
}

pub fn document(painter: &Painter, rect: Rect, color: Color32, width: f32) {
    poly(
        painter,
        rect,
        &[
            (0.24, 0.12),
            (0.62, 0.12),
            (0.78, 0.3),
            (0.78, 0.88),
            (0.24, 0.88),
            (0.24, 0.12),
        ],
        color,
        width,
    );
    poly(
        painter,
        rect,
        &[(0.62, 0.12), (0.62, 0.3), (0.78, 0.3)],
        color,
        width,
    );
}

pub fn logo(painter: &Painter, rect: Rect) {
    let radius = (rect.width() * 0.28).round() as u8;
    painter.rect_filled(rect, radius, crate::theme::ACCENT);
    let inner = rect.shrink(rect.width() * 0.2);
    painter.rect_stroke(
        inner.translate(Vec2::new(-rect.width() * 0.06, rect.height() * 0.06)),
        (inner.width() * 0.22).round() as u8,
        Stroke::new(rect.width() * 0.055, Color32::from_rgb(0x0B, 0x0F, 0x14)),
        StrokeKind::Inside,
    );
    painter.rect_stroke(
        inner.translate(Vec2::new(rect.width() * 0.06, -rect.height() * 0.06)),
        (inner.width() * 0.22).round() as u8,
        Stroke::new(rect.width() * 0.055, Color32::from_rgb(0x0B, 0x0F, 0x14)),
        StrokeKind::Inside,
    );
}
