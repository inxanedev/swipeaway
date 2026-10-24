use std::path::PathBuf;

use egui::{
    Align2, Color32, CornerRadius, FontId, Rect, RichText, ScrollArea, Sense, Stroke, StrokeKind,
    Ui, pos2, vec2,
};

use crate::app::{SwipeAwayApp, ToastKind};
use crate::theme;
use crate::util;

use super::icons;
use super::widgets;

impl SwipeAwayApp {
    pub(crate) fn screen_welcome(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ScrollArea::vertical()
            .id_salt("welcome-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                widgets::page(ui, 900.0, |ui| {
                    ui.add_space(8.0);
                    self.welcome_header(ui);
                    ui.add_space(34.0);
                    self.welcome_hero(ui, ctx);
                    ui.add_space(26.0);
                    if let Some(error) = self.startup_error.clone() {
                        self.welcome_error(ui, &error);
                        ui.add_space(14.0);
                    }
                    self.welcome_safety(ui);
                    ui.add_space(26.0);
                    self.welcome_recent(ui, ctx);
                    ui.add_space(36.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "SwipeAway {}  ·  Rust + egui",
                                env!("CARGO_PKG_VERSION")
                            ))
                            .font(FontId::proportional(11.0))
                            .color(theme::FAINT),
                        );
                    });
                    ui.add_space(18.0);
                });
            });

        let hovering = ctx.input(|input| !input.raw.hovered_files.is_empty());
        if hovering {
            let rect = ctx.content_rect();
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::unique("drop-overlay"),
            ));
            painter.rect_filled(
                rect,
                0,
                Color32::from_rgba_unmultiplied(0x0B, 0x0F, 0x14, 220),
            );
            let inner = rect.shrink(30.0);
            painter.rect_stroke(
                inner,
                CornerRadius::same(24),
                Stroke::new(2.0, theme::alpha(theme::ACCENT, 150)),
                StrokeKind::Inside,
            );
            icons::folder(
                &painter,
                Rect::from_center_size(
                    pos2(inner.center().x, inner.center().y - 30.0),
                    vec2(64.0, 64.0),
                ),
                theme::ACCENT,
                2.5,
            );
            painter.text(
                pos2(inner.center().x, inner.center().y + 36.0),
                Align2::CENTER_CENTER,
                "Drop a folder to start cleaning",
                theme::semibold(22.0),
                theme::TEXT,
            );
        }

        let dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect()
        });
        if let Some(path) = dropped.into_iter().next() {
            let target = if path.is_dir() {
                path
            } else {
                path.parent().map(PathBuf::from).unwrap_or(path)
            };
            self.try_start_session(target, ctx);
        }
    }

    fn welcome_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            widgets::logo_badge(ui, 34.0);
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("SwipeAway")
                        .font(theme::semibold(19.0))
                        .color(theme::TEXT),
                );
                ui.label(
                    RichText::new("a calmer way to clean up")
                        .font(FontId::proportional(10.5))
                        .color(theme::FAINT),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                widgets::plain_chip(ui, "Nothing is ever deleted", theme::KEEP);
            });
        });
    }

    fn welcome_hero(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(vec2(width, 250.0), Sense::hover());
        let painter = ui.painter();

        painter.text(
            pos2(rect.left() + 2.0, rect.top() + 4.0),
            Align2::LEFT_TOP,
            "Keep what matters.",
            theme::semibold(46.0),
            theme::TEXT,
        );
        painter.text(
            pos2(rect.left() + 2.0, rect.top() + 58.0),
            Align2::LEFT_TOP,
            "Clear the rest.",
            theme::semibold(46.0),
            theme::ACCENT,
        );
        painter.text(
            pos2(rect.left() + 3.0, rect.top() + 126.0),
            Align2::LEFT_TOP,
            "One file at a time, with a real preview. Keep it, move it to the\njunk folder, or set it aside for a second look.",
            FontId::proportional(15.5),
            theme::MUTED,
        );

        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(
                    pos2(rect.left() + 2.0, rect.top() + 186.0),
                    vec2(width - 4.0, 64.0),
                ))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let response =
            widgets::primary_button(&mut child, "Choose a folder", Some(icons::folder), 230.0);
        if response.clicked() {
            self.open_picker(ctx);
        }
        child.add_space(14.0);
        child.label(
            RichText::new("or drop a folder anywhere in this window")
                .font(FontId::proportional(12.5))
                .color(theme::FAINT),
        );
    }

    fn welcome_error(&mut self, ui: &mut Ui, message: &str) {
        egui::Frame::NONE
            .fill(theme::alpha(theme::DELETE, 14))
            .stroke(Stroke::new(1.0, theme::alpha(theme::DELETE, 60)))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(egui::Margin::same(14))
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                    icons::warning(ui.painter(), rect, theme::DELETE, 1.8);
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            RichText::new("That folder can't be cleaned")
                                .font(theme::semibold(13.5))
                                .color(theme::DELETE),
                        );
                        ui.add(
                            egui::Label::new(
                                RichText::new(message)
                                    .font(FontId::proportional(12.5))
                                    .color(theme::TEXT_SOFT),
                            )
                            .wrap(),
                        );
                        ui.add_space(4.0);
                        ui.allocate_ui_with_layout(
                            vec2(ui.available_width(), 28.0),
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if widgets::ghost_button(ui, "Dismiss", theme::MUTED).clicked() {
                                    self.startup_error = None;
                                }
                            },
                        );
                    });
                });
            });
    }

    fn welcome_safety(&mut self, ui: &mut Ui) {
        egui::Frame::NONE
            .fill(theme::PANEL_RAISED)
            .stroke(Stroke::new(1.0, theme::STROKE_SOFT))
            .corner_radius(CornerRadius::same(15))
            .inner_margin(egui::Margin::same(18))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::hover());
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(10),
                        theme::alpha(theme::KEEP, 24),
                    );
                    icons::check(ui.painter(), rect.shrink(9.0), theme::KEEP, 2.2);
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Nothing is deleted, ever")
                                .font(theme::semibold(15.0))
                                .color(theme::TEXT),
                        );
                        ui.label(
                            RichText::new(
                                "Delete moves an item into a `swipeaway-junk` folder in the same \
                                 directory. Manual moves it to `swipeaway-manual`. You clear them \
                                 out yourself, whenever you're ready.",
                            )
                            .font(FontId::proportional(12.5))
                            .color(theme::MUTED),
                        );
                    });
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    widgets::plain_chip(ui, "Hidden & system items included", theme::MANUAL);
                    widgets::plain_chip(ui, "System folders blocked", theme::BLUE);
                });
            });
    }

    fn welcome_recent(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Recent folders")
                    .font(theme::semibold(15.0))
                    .color(theme::TEXT_SOFT),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !self.config.recent_folders.is_empty() {
                    widgets::plain_chip(
                        ui,
                        &format!("{}", self.config.recent_folders.len()),
                        theme::MUTED,
                    );
                }
            });
        });
        ui.add_space(8.0);

        if self.config.recent_folders.is_empty() {
            widgets::inset_frame().show(ui, |ui| {
                ui.set_min_height(64.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Folders you open will show up here for quick access.")
                            .font(FontId::proportional(12.5))
                            .color(theme::FAINT),
                    );
                });
            });
            return;
        }

        let recents = self.config.recent_folders.clone();
        let mut pick: Option<PathBuf> = None;
        let mut forget: Option<PathBuf> = None;
        for path in recents {
            if recent_row(ui, &path).clicked() {
                if path.is_dir() {
                    pick = Some(path);
                } else {
                    forget = Some(path);
                }
            }
            ui.add_space(6.0);
        }

        if let Some(path) = pick {
            self.try_start_session(path, ctx);
        }
        if let Some(path) = forget {
            self.config.forget(&path);
            self.set_toast(
                "That folder no longer exists, so it was removed from recents.".to_owned(),
                ToastKind::Warning,
            );
        }
    }
}

fn recent_row(ui: &mut Ui, path: &std::path::Path) -> egui::Response {
    let exists = path.is_dir();
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 56.0), Sense::click());
    let hover =
        ui.ctx()
            .animate_bool_with_time(response.id.with("recent-hover"), response.hovered(), 0.1);
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        CornerRadius::same(12),
        if exists {
            theme::mix(theme::PANEL_RAISED, theme::CARD_HOVER, hover)
        } else {
            theme::PANEL_RAISED
        },
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(1.0, theme::mix(theme::STROKE_SOFT, theme::STROKE, hover)),
        StrokeKind::Inside,
    );

    let icon_rect =
        Rect::from_center_size(pos2(rect.left() + 28.0, rect.center().y), vec2(24.0, 24.0));
    icons::folder(
        painter,
        icon_rect,
        if exists { theme::ACCENT } else { theme::FAINT },
        1.8,
    );

    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    painter.text(
        pos2(rect.left() + 50.0, rect.center().y - 8.0),
        Align2::LEFT_CENTER,
        if exists {
            name
        } else {
            format!("{name}  (missing)")
        },
        theme::semibold(13.5),
        if exists { theme::TEXT } else { theme::MUTED },
    );
    painter.text(
        pos2(rect.left() + 50.0, rect.center().y + 11.0),
        Align2::LEFT_CENTER,
        util::truncate_middle(&path.display().to_string(), 88),
        FontId::proportional(10.5),
        theme::FAINT,
    );

    painter.text(
        pos2(rect.right() - 18.0, rect.center().y),
        Align2::RIGHT_CENTER,
        if exists { "Open  →" } else { "Remove" },
        theme::semibold(11.5),
        if exists { theme::MUTED } else { theme::MANUAL },
    );

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
