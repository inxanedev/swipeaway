use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Layout, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, Ui, pos2, vec2,
};

use crate::app::{PendingAction, SwipeAwayApp};
use crate::core::entry::Entry;
use crate::core::session::Decision;
use crate::theme;
use crate::util;

use super::icons;
use super::widgets;

impl SwipeAwayApp {
    pub(crate) fn screen_summary(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let Some(session) = &self.session else {
            self.screen = crate::app::Screen::Welcome;
            return;
        };

        let root = session.root.clone();
        let total = session.total();
        let kept = session.kept;
        let deleted = session.deleted;
        let manual = session.manual;
        let deleted_bytes = session.deleted_bytes;
        let manual_bytes = session.manual_bytes;
        let can_undo = session.can_undo();
        let elapsed = session.started_at.elapsed();
        let warned = session.warnings.len();
        let deleted_entries: Vec<Entry> = session.entries_for(Decision::Junk).cloned().collect();
        let manual_entries: Vec<Entry> = session.entries_for(Decision::Manual).cloned().collect();
        let kept_entries: Vec<Entry> = session.entries_for(Decision::Keep).cloned().collect();
        let junk_dir = session.junk_dir();
        let manual_dir = session.manual_dir();
        let junk_exists = junk_dir.exists();
        let manual_exists = manual_dir.exists();

        ScrollArea::vertical()
            .id_salt("summary-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                widgets::page(ui, 1000.0, |ui| {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        widgets::logo_badge(ui, 30.0);
                        ui.add_space(2.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("SwipeAway")
                                    .font(theme::semibold(16.5))
                                    .color(theme::TEXT),
                            );
                            ui.label(
                                RichText::new(util::short_path(&root, 72))
                                    .font(theme::mono(10.5))
                                    .color(theme::FAINT),
                            )
                            .on_hover_text(root.display().to_string());
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if widgets::ghost_button(ui, "Close", theme::MUTED).clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            if widgets::ghost_button(ui, "Clean another folder", theme::MUTED)
                                .clicked()
                            {
                                self.leave_session(ctx, true);
                            }
                        });
                    });

                    ui.add_space(34.0);
                    let hero_height = 170.0;
                    let (hero_rect, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), hero_height), Sense::hover());
                    let painter = ui.painter();
                    let badge = egui::Rect::from_center_size(
                        pos2(hero_rect.center().x, hero_rect.top() + 48.0),
                        vec2(76.0, 76.0),
                    );
                    painter.circle_filled(badge.center(), 38.0, theme::alpha(theme::KEEP, 22));
                    painter.circle_stroke(
                        badge.center(),
                        37.0,
                        Stroke::new(1.0, theme::alpha(theme::KEEP, 65)),
                    );
                    icons::check(painter, badge.shrink(22.0), theme::KEEP, 3.2);
                    painter.text(
                        pos2(hero_rect.center().x, hero_rect.top() + 112.0),
                        Align2::CENTER_CENTER,
                        if total == 0 {
                            "This folder was already clear"
                        } else {
                            "That's the whole folder"
                        },
                        theme::semibold(31.0),
                        theme::TEXT,
                    );
                    painter.text(
                        pos2(hero_rect.center().x, hero_rect.top() + 143.0),
                        Align2::CENTER_CENTER,
                        if total == 0 {
                            "Nothing was moved. Pick another folder whenever you're ready.".to_owned()
                        } else {
                            format!(
                                "You reviewed {total} item{} in {}.",
                                if total == 1 { "" } else { "s" },
                                format_elapsed(elapsed)
                            )
                        },
                        FontId::proportional(13.5),
                        theme::MUTED,
                    );

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let gap = 12.0;
                        let width = (ui.available_width() - gap * 2.0) / 3.0;
                        ui.spacing_mut().item_spacing.x = gap;
                        widgets::stat_block(ui, "Kept", &format!("{kept}"), theme::KEEP, width);
                        widgets::stat_block(ui, "Deleted", &format!("{deleted}"), theme::DELETE, width);
                        widgets::stat_block(ui, "Manual", &format!("{manual}"), theme::MANUAL, width);
                    });
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        let gap = 12.0;
                        let width = (ui.available_width() - gap) / 2.0;
                        ui.spacing_mut().item_spacing.x = gap;
                        size_card(
                            ui,
                            "Moved to swipeaway-junk",
                            &util::format_size(deleted_bytes),
                            "Folder contents aren't included in this total.",
                            theme::DELETE,
                            width,
                        );
                        size_card(
                            ui,
                            "Set aside in swipeaway-manual",
                            &util::format_size(manual_bytes),
                            "Review these files later at your own pace.",
                            theme::MANUAL,
                            width,
                        );
                    });

                    ui.add_space(18.0);
                    ui.horizontal(|ui| {
                        if widgets::secondary_button(
                            ui,
                            "Open junk folder",
                            Some(icons::folder),
                            theme::DELETE,
                            "Open swipeaway-junk in Explorer",
                        )
                        .clicked()
                        {
                            if junk_exists {
                                self.open_folder(&junk_dir);
                            } else {
                                self.set_toast(
                                    "There is no junk folder yet.".to_owned(),
                                    crate::app::ToastKind::Warning,
                                );
                            }
                        }
                        if widgets::secondary_button(
                            ui,
                            "Open manual folder",
                            Some(icons::folder),
                            theme::MANUAL,
                            "Open swipeaway-manual in Explorer",
                        )
                        .clicked()
                        {
                            if manual_exists {
                                self.open_folder(&manual_dir);
                            } else {
                                self.set_toast(
                                    "There is no manual folder yet.".to_owned(),
                                    crate::app::ToastKind::Warning,
                                );
                            }
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if widgets::primary_button(ui, "Clean another folder", None, 210.0)
                                .clicked()
                            {
                                self.leave_session(ctx, true);
                            }
                            if can_undo
                                && widgets::secondary_button(
                                    ui,
                                    "Undo last",
                                    Some(icons::undo),
                                    theme::BLUE,
                                    "Restore the most recent decision and go back",
                                )
                                .clicked()
                            {
                                self.pending = Some(PendingAction::Undo);
                            }
                        });
                    });

                    ui.add_space(14.0);
                    let notice = if deleted == 0 && manual == 0 {
                        "Nothing was moved. Every item was kept in place.".to_owned()
                    } else {
                        "Everything is still on disk. Empty the junk and manual folders whenever you're ready."
                            .to_owned()
                    };
                    egui::Frame::NONE
                        .fill(theme::PANEL_RAISED)
                        .stroke(Stroke::new(1.0, theme::STROKE_SOFT))
                        .corner_radius(CornerRadius::same(12))
                        .inner_margin(egui::Margin::same(13))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (rect, _) =
                                    ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                                icons::clock(ui.painter(), rect, theme::MUTED, 1.4);
                                ui.label(
                                    RichText::new(notice)
                                        .font(FontId::proportional(12.0))
                                        .color(theme::MUTED),
                                );
                            });
                        });

                    if warned > 0 {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!(
                                "{warned} entry/entries could not be inspected and were left untouched."
                            ))
                            .font(FontId::proportional(11.5))
                            .color(theme::MANUAL),
                        );
                    }

                    if !deleted_entries.is_empty() {
                        ui.add_space(18.0);
                        entry_section(ui, "Deleted items", &deleted_entries, theme::DELETE);
                    }
                    if !manual_entries.is_empty() {
                        ui.add_space(10.0);
                        entry_section(ui, "Set-aside items", &manual_entries, theme::MANUAL);
                    }
                    if !kept_entries.is_empty() {
                        ui.add_space(10.0);
                        entry_section(ui, "Kept items", &kept_entries, theme::KEEP);
                    }

                    ui.add_space(24.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Nothing leaves your PC until you empty those folders yourself.")
                                .font(FontId::proportional(11.0))
                                .color(theme::FAINT),
                        );
                    });
                    ui.add_space(18.0);
                });
            });
    }
}

fn size_card(ui: &mut Ui, title: &str, value: &str, subtitle: &str, color: Color32, width: f32) {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 92.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(14), theme::alpha(color, 8));
    painter.rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0, theme::alpha(color, 30)),
        StrokeKind::Inside,
    );
    painter.text(
        pos2(rect.left() + 16.0, rect.top() + 20.0),
        Align2::LEFT_CENTER,
        title,
        theme::semibold(11.5),
        theme::MUTED,
    );
    painter.text(
        pos2(rect.left() + 16.0, rect.top() + 48.0),
        Align2::LEFT_CENTER,
        value,
        theme::semibold(22.0),
        color,
    );
    painter.text(
        pos2(rect.left() + 16.0, rect.top() + 72.0),
        Align2::LEFT_CENTER,
        util::truncate_middle(subtitle, 56),
        FontId::proportional(10.0),
        theme::FAINT,
    );
    response.on_hover_text(subtitle);
}

fn entry_section(ui: &mut Ui, title: &str, entries: &[Entry], color: Color32) {
    egui::Frame::NONE
        .fill(theme::PANEL_RAISED)
        .stroke(Stroke::new(1.0, theme::STROKE_SOFT))
        .corner_radius(CornerRadius::same(13))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::CollapsingHeader::new(
                RichText::new(format!("{title}  ·  {}", entries.len()))
                    .font(theme::semibold(13.0))
                    .color(color),
            )
            .id_salt(format!("section-{title}"))
            .default_open(false)
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .id_salt(format!("list-{title}"))
                    .max_height(220.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for entry in entries {
                            let (rect, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), 26.0),
                                Sense::hover(),
                            );
                            let painter = ui.painter();
                            let icon_rect = egui::Rect::from_center_size(
                                pos2(rect.left() + 10.0, rect.center().y),
                                vec2(13.0, 13.0),
                            );
                            if entry.is_dir {
                                icons::folder(painter, icon_rect, color, 1.3);
                            } else {
                                icons::document(painter, icon_rect, theme::MUTED, 1.1);
                            }
                            painter.text(
                                pos2(rect.left() + 24.0, rect.center().y),
                                Align2::LEFT_CENTER,
                                util::truncate_middle(&entry.name, 100),
                                FontId::proportional(11.8),
                                theme::TEXT_SOFT,
                            );
                            painter.text(
                                pos2(rect.right() - 2.0, rect.center().y),
                                Align2::RIGHT_CENTER,
                                if entry.is_dir {
                                    "Folder".to_owned()
                                } else {
                                    util::format_size_short(entry.size)
                                },
                                theme::mono(10.0),
                                theme::FAINT,
                            );
                        }
                    });
            });
        });
}

fn format_elapsed(elapsed: std::time::Duration) -> String {
    let seconds = elapsed.as_secs();
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3_600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3_600, (seconds % 3_600) / 60)
    }
}
