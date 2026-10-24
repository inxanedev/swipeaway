use std::path::Path;

use egui::{
    Align, CornerRadius, FontId, Label, Layout, RichText, ScrollArea, Sense, Stroke, Ui, vec2,
};

use crate::app::{PendingAction, Screen, SwipeAwayApp};
use crate::core::entry::Entry;
use crate::core::session::Decision;
use crate::preview::{MediaKind, PreviewContent, PreviewResult, detect};
use crate::theme;
use crate::util;

use super::icons;
use super::widgets;

impl SwipeAwayApp {
    pub(crate) fn screen_decision(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let Some(session) = &self.session else {
            self.screen = Screen::Welcome;
            return;
        };
        let root = session.root.clone();
        let total = session.total();
        let index = session.index;
        let kept = session.kept;
        let deleted = session.deleted;
        let manual = session.manual;
        let can_undo = session.can_undo();
        let warning_count = session.warnings.len();
        let entry = session.current().cloned();

        widgets::page(ui, 1180.0, |ui| {
            self.decision_topbar(
                ui,
                ctx,
                &root,
                index,
                total,
                kept,
                deleted,
                manual,
                can_undo,
                warning_count,
            );
            ui.add_space(14.0);

            if let Some(entry) = &entry {
                self.decision_card(ui, ctx, entry);
                ui.add_space(12.0);
                let widths = (ui.available_width() - 24.0) / 3.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    if widgets::action_button(
                        ui,
                        widths,
                        "Delete",
                        "D",
                        theme::DELETE,
                        icons::trash,
                        true,
                        "Move this item into the swipeaway-junk folder",
                    )
                    .clicked()
                    {
                        self.pending = Some(PendingAction::Decide(Decision::Junk));
                    }
                    if widgets::action_button(
                        ui,
                        widths,
                        "Manual",
                        "M",
                        theme::MANUAL,
                        icons::tray,
                        true,
                        "Move this item into swipeaway-manual for a second look",
                    )
                    .clicked()
                    {
                        self.pending = Some(PendingAction::Decide(Decision::Manual));
                    }
                    if widgets::action_button(
                        ui,
                        widths,
                        "Keep",
                        "K",
                        theme::KEEP,
                        icons::check,
                        true,
                        "Leave this item exactly where it is",
                    )
                    .clicked()
                    {
                        self.pending = Some(PendingAction::Decide(Decision::Keep));
                    }
                });
                ui.add_space(4.0);
                shortcut_hint(ui);
            } else {
                self.empty_decision_state(ui, ctx, &root);
            }
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn decision_topbar(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        root: &Path,
        index: usize,
        total: usize,
        kept: u64,
        deleted: u64,
        manual: u64,
        can_undo: bool,
        warning_count: usize,
    ) {
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
                    RichText::new(util::short_path(root, 76))
                        .font(theme::mono(10.5))
                        .color(theme::FAINT),
                )
                .on_hover_text(root.display().to_string());
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::ghost_button(ui, "Change folder", theme::MUTED).clicked() {
                    self.request_leave(ctx, false);
                }
                if widgets::ghost_button(ui, "Shortcuts", theme::MUTED).clicked() {
                    self.show_shortcuts = true;
                }
                if can_undo {
                    if widgets::secondary_button(
                        ui,
                        "Undo",
                        Some(icons::undo),
                        theme::BLUE,
                        "Undo the last decision (Ctrl+Z)",
                    )
                    .clicked()
                    {
                        self.pending = Some(PendingAction::Undo);
                    }
                } else {
                    widgets::plain_chip(ui, "Undo", theme::FAINT);
                }
            });
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{index} of {total} decided"))
                    .font(theme::semibold(18.0))
                    .color(theme::TEXT),
            );
            ui.label(
                RichText::new(format!("{} remaining", total.saturating_sub(index)))
                    .font(FontId::proportional(12.5))
                    .color(theme::MUTED),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                widgets::chip(ui, &format!("{manual} manual"), theme::MANUAL);
                widgets::chip(ui, &format!("{deleted} deleted"), theme::DELETE);
                widgets::chip(ui, &format!("{kept} kept"), theme::KEEP);
                if warning_count > 0 {
                    widgets::chip(ui, &format!("{warning_count} skipped"), theme::MUTED)
                        .on_hover_text("Some entries could not be read and were skipped.");
                }
            });
        });
        ui.add_space(6.0);
        let progress = if total == 0 {
            1.0
        } else {
            index as f32 / total as f32
        };
        widgets::progress_bar(ui, progress, 7.0);
    }

    fn decision_card(&mut self, ui: &mut Ui, ctx: &egui::Context, entry: &Entry) {
        let card_height = (ui.available_height() - 130.0).max(280.0);
        let card_rect =
            egui::Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), card_height));
        let mut card_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(card_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        egui::Frame::NONE
            .fill(theme::CARD)
            .stroke(Stroke::new(1.0, theme::STROKE_SOFT))
            .corner_radius(CornerRadius::same(18))
            .inner_margin(egui::Margin::same(18))
            .show(&mut card_ui, |ui| {
                ui.set_width(ui.available_width());
                self.card_header(ui, ctx, entry);
                ui.add_space(10.0);

                let body_height = ui.available_height();
                let gap = 14.0;
                let sidebar_width = if ui.available_width() > 900.0 {
                    286.0
                } else {
                    250.0
                };
                let left_width = (ui.available_width() - sidebar_width - gap).max(320.0);

                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    ui.allocate_ui_with_layout(
                        vec2(left_width, body_height),
                        Layout::top_down(Align::Min),
                        |ui| {
                            if entry.is_dir {
                                self.ui_folder_explorer(ui);
                            } else {
                                widgets::inset_frame().show(ui, |ui| {
                                    ui.set_min_size(vec2(
                                        left_width - 28.0,
                                        (body_height - 28.0).max(120.0),
                                    ));
                                    self.preview.show(ui);
                                });
                            }
                        },
                    );
                    ui.allocate_ui_with_layout(
                        vec2(sidebar_width, body_height),
                        Layout::top_down(Align::Min),
                        |ui| {
                            self.detail_sidebar(ui, entry);
                        },
                    );
                });
            });
        ui.advance_cursor_after_rect(card_rect);
    }

    fn card_header(&mut self, ui: &mut Ui, ctx: &egui::Context, entry: &Entry) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.add(
                    Label::new(
                        RichText::new(&entry.name)
                            .font(theme::semibold(23.0))
                            .color(theme::TEXT),
                    )
                    .truncate(),
                )
                .on_hover_text(&entry.name);
                ui.add(
                    Label::new(
                        RichText::new(util::short_path(&entry.path, 96))
                            .font(theme::mono(10.5))
                            .color(theme::FAINT),
                    )
                    .truncate(),
                )
                .on_hover_text(entry.path.display().to_string());

                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    if entry.is_dir {
                        widgets::plain_chip(ui, "Folder", theme::MANUAL);
                        if let Some(listing) = &self.folder.listing {
                            widgets::plain_chip(ui, &util::item_count(listing.total), theme::MUTED);
                        }
                    } else {
                        let kind = detect::classify(&entry.path);
                        widgets::plain_chip(ui, kind.label(), super::kind_color(kind));
                        widgets::plain_chip(ui, &util::format_size(entry.size), theme::MUTED);
                    }
                    if entry.hidden {
                        widgets::plain_chip(ui, "Hidden", theme::MANUAL);
                    }
                    if entry.system {
                        widgets::plain_chip(ui, "System", theme::DELETE);
                    }
                    if entry.is_symlink {
                        widgets::plain_chip(ui, "Shortcut", theme::BLUE);
                    }
                    if entry.readonly {
                        widgets::plain_chip(ui, "Read-only", theme::MUTED);
                    }
                });
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::secondary_button(
                    ui,
                    "Show in folder",
                    Some(icons::folder),
                    theme::TEXT_SOFT,
                    "Open Explorer with this item selected",
                )
                .clicked()
                {
                    self.show_in_folder(&entry.path);
                }
                if widgets::secondary_button(
                    ui,
                    "Open",
                    Some(icons::open_external),
                    theme::TEXT_SOFT,
                    "Open this item with the default app",
                )
                .clicked()
                {
                    self.open_with_default(&entry.path);
                }
                if !entry.is_dir {
                    let kind = detect::classify(&entry.path);
                    if matches!(kind, MediaKind::Video | MediaKind::Audio)
                        && widgets::secondary_button(
                            ui,
                            "Play",
                            Some(icons::play),
                            super::kind_color(kind),
                            "Play this file in the default media player",
                        )
                        .clicked()
                    {
                        self.open_with_default(&entry.path);
                    }
                }
                let _ = ctx;
            });
        });
    }

    fn detail_sidebar(&self, ui: &mut Ui, entry: &Entry) {
        let preview = self
            .preview
            .result
            .as_deref()
            .filter(|result| result.path == entry.path);

        widgets::inset_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height((ui.available_height()).max(120.0));
            ScrollArea::vertical()
                .id_salt("detail-sidebar")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("Details")
                            .font(theme::semibold(14.0))
                            .color(theme::TEXT_SOFT),
                    );
                    ui.add_space(6.0);

                    if entry.hidden || entry.system {
                        warning_card(
                            ui,
                            if entry.hidden && entry.system {
                                "Hidden system item"
                            } else if entry.hidden {
                                "Hidden item"
                            } else {
                                "System item"
                            },
                            "This item is normally hidden by Windows. Make sure you know what it is before moving or keeping it.",
                        );
                        ui.add_space(8.0);
                    }

                    let kind = if entry.is_dir {
                        MediaKind::Unknown
                    } else {
                        detect::classify(&entry.path)
                    };
                    detail_row(ui, "Type", if entry.is_dir { "Folder".to_owned() } else { kind.label().to_owned() });
                    if entry.is_dir {
                        let items = self
                            .folder
                            .listing
                            .as_ref()
                            .map(|listing| listing.total.to_string())
                            .unwrap_or_else(|| "—".to_owned());
                        detail_row(ui, "Items", items);
                    } else {
                        detail_row(ui, "Size", util::format_size(entry.size));
                        detail_row_mb(ui, entry.size);
                    }
                    detail_row_time(ui, "Created", entry.created);
                    detail_row_time(ui, "Modified", entry.modified);
                    detail_row_time(ui, "Last opened", entry.accessed);
                    detail_row_path(ui, &entry.path);

                    if let Some(result) = preview {
                        ui.add_space(8.0);
                        widgets::divider(ui);
                        ui.add_space(8.0);
                        preview_details(ui, result);
                    }

                    if let Some(listing) = &self.folder.listing
                        && entry.is_dir && !listing.warnings.is_empty() {
                            ui.add_space(8.0);
                            warning_card(
                                ui,
                                "Some items were skipped",
                                &format!(
                                    "{} item(s) in this folder could not be read.",
                                    listing.warnings.len()
                                ),
                            );
                        }
                });
        });
    }

    fn empty_decision_state(&mut self, ui: &mut Ui, ctx: &egui::Context, root: &Path) {
        let height = (ui.available_height() - 30.0).max(300.0);
        egui::Frame::NONE
            .fill(theme::CARD)
            .stroke(Stroke::new(1.0, theme::STROKE_SOFT))
            .corner_radius(CornerRadius::same(18))
            .inner_margin(egui::Margin::same(18))
            .show(ui, |ui| {
                ui.set_min_height(height);
                ui.vertical_centered(|ui| {
                    ui.add_space(height * 0.18);
                    let (rect, _) = ui.allocate_exact_size(vec2(64.0, 64.0), Sense::hover());
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(20),
                        theme::alpha(theme::KEEP, 22),
                    );
                    icons::check(ui.painter(), rect.shrink(18.0), theme::KEEP, 3.0);
                    ui.add_space(18.0);
                    ui.label(
                        RichText::new("Nothing to clean here")
                            .font(theme::semibold(26.0))
                            .color(theme::TEXT),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(format!(
                            "`{}` doesn't contain any items SwipeAway can review.",
                            util::short_path(root, 80)
                        ))
                        .font(FontId::proportional(13.5))
                        .color(theme::MUTED),
                    );
                    ui.add_space(22.0);
                    ui.horizontal(|ui| {
                        let width = 220.0;
                        let total = width * 2.0 + 12.0;
                        ui.add_space((ui.available_width() - total) * 0.5);
                        if widgets::secondary_button(ui, "Rescan", None, theme::TEXT_SOFT, "")
                            .clicked()
                        {
                            let root = root.to_path_buf();
                            self.try_start_session(root, ctx);
                        }
                        ui.add_space(12.0);
                        if widgets::primary_button(ui, "Choose another folder", None, width)
                            .clicked()
                        {
                            self.request_leave(ctx, true);
                        }
                    });
                });
            });
    }
}

fn shortcut_hint(ui: &mut Ui) {
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        widgets::key_hint(ui, "D");
        ui.label(
            RichText::new("Delete")
                .font(FontId::proportional(11.0))
                .color(theme::FAINT),
        );
        ui.add_space(6.0);
        widgets::key_hint(ui, "M");
        ui.label(
            RichText::new("Manual")
                .font(FontId::proportional(11.0))
                .color(theme::FAINT),
        );
        ui.add_space(6.0);
        widgets::key_hint(ui, "K");
        ui.label(
            RichText::new("Keep")
                .font(FontId::proportional(11.0))
                .color(theme::FAINT),
        );
        ui.add_space(6.0);
        widgets::key_hint(ui, "Ctrl+Z");
        ui.label(
            RichText::new("Undo")
                .font(FontId::proportional(11.0))
                .color(theme::FAINT),
        );
    });
}

fn detail_row(ui: &mut Ui, label: &str, value: String) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .font(FontId::proportional(11.5))
                .color(theme::FAINT),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(
                Label::new(
                    RichText::new(value)
                        .font(theme::semibold(11.5))
                        .color(theme::TEXT_SOFT),
                )
                .truncate(),
            );
        });
    });
    ui.add_space(1.0);
}

fn detail_row_mb(ui: &mut Ui, bytes: u64) {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Megabytes")
                .font(FontId::proportional(11.5))
                .color(theme::FAINT),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{mb:.3} MB"))
                    .font(theme::mono(11.0))
                    .color(theme::MUTED),
            );
        });
    });
    ui.add_space(1.0);
}

fn detail_row_time(ui: &mut Ui, label: &str, time: Option<std::time::SystemTime>) {
    let value = util::format_datetime(time);
    let age = util::format_age(time);
    let response = ui
        .horizontal(|ui| {
            ui.label(
                RichText::new(label)
                    .font(FontId::proportional(11.5))
                    .color(theme::FAINT),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(
                    Label::new(
                        RichText::new(value.clone())
                            .font(theme::mono(10.5))
                            .color(theme::TEXT_SOFT),
                    )
                    .truncate(),
                );
            });
        })
        .response;
    if !age.is_empty() {
        response.on_hover_text(age);
    }
}

fn detail_row_path(ui: &mut Ui, path: &Path) {
    ui.add_space(4.0);
    ui.label(
        RichText::new("Full path")
            .font(FontId::proportional(11.5))
            .color(theme::FAINT),
    );
    let response = ui.add(
        Label::new(
            RichText::new(path.display().to_string())
                .font(theme::mono(10.0))
                .color(theme::MUTED),
        )
        .wrap_mode(egui::TextWrapMode::Wrap),
    );
    response.on_hover_text("Click to copy").clicked().then(|| {
        ui.ctx().copy_text(path.display().to_string());
    });
}

fn warning_card(ui: &mut Ui, title: &str, message: &str) {
    egui::Frame::NONE
        .fill(theme::alpha(theme::MANUAL, 12))
        .stroke(Stroke::new(1.0, theme::alpha(theme::MANUAL, 45)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                icons::warning(ui.painter(), rect, theme::MANUAL, 1.5);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(title)
                            .font(theme::semibold(11.5))
                            .color(theme::MANUAL),
                    );
                    ui.label(
                        RichText::new(message)
                            .font(FontId::proportional(10.8))
                            .color(theme::MUTED),
                    );
                });
            });
        });
}

fn preview_details(ui: &mut Ui, result: &PreviewResult) {
    match &result.content {
        PreviewContent::Image(image) => {
            detail_row(
                ui,
                "Dimensions",
                format!("{} × {}", image.original_width, image.original_height),
            );
            detail_row(ui, "Format", image.format.clone());
            if image.animated {
                detail_row(ui, "Animated", "Yes".to_owned());
            }
            if image.orientation_applied {
                detail_row(ui, "Orientation", "EXIF applied".to_owned());
            }
        }
        PreviewContent::Video(video) => {
            if let Some(duration) = video.meta.duration_secs {
                detail_row(ui, "Duration", util::format_seconds(duration));
            }
            if let (Some(width), Some(height)) = (video.meta.width, video.meta.height) {
                detail_row(ui, "Resolution", format!("{width} × {height}"));
            }
            if let Some(codec) = &video.meta.codec {
                detail_row(ui, "Codec", codec.clone());
            }
            if let Some(rate) = video.meta.frame_rate {
                detail_row(ui, "Frame rate", format!("{rate:.2} fps"));
            }
            if video.meta.has_audio {
                detail_row(ui, "Audio", "Yes".to_owned());
            }
            detail_row(ui, "Preview source", video.source.clone());
        }
        PreviewContent::Audio(audio) => {
            if let Some(title) = &audio.meta.title {
                detail_row(ui, "Title", title.clone());
            }
            if let Some(artist) = &audio.meta.artist {
                detail_row(ui, "Artist", artist.clone());
            }
            if let Some(album) = &audio.meta.album {
                detail_row(ui, "Album", album.clone());
            }
            if let Some(duration) = audio.meta.duration_secs {
                detail_row(ui, "Duration", util::format_seconds(duration));
            }
            if let Some(bitrate) = audio.meta.bitrate {
                detail_row(ui, "Bitrate", format!("{bitrate} kbps"));
            }
            if let Some(rate) = audio.meta.sample_rate {
                detail_row(ui, "Sample rate", format!("{rate} Hz"));
            }
            if let Some(channels) = audio.meta.channels {
                detail_row(ui, "Channels", channels.to_string());
            }
            detail_row(ui, "Format", audio.meta.format.clone());
        }
        PreviewContent::Text(text) => {
            detail_row(ui, "Language", text.language.to_owned());
            detail_row(ui, "Encoding", text.encoding.clone());
            detail_row(ui, "Lines", text.lines_shown.to_string());
            if text.file_truncated {
                detail_row(ui, "Note", "Preview truncated".to_owned());
            }
        }
        PreviewContent::Archive(archive) => {
            detail_row(ui, "Entries", archive.total_entries.to_string());
            detail_row(ui, "Shown size", util::format_size(archive.shown_size));
        }
        PreviewContent::Font(font) => {
            detail_row(ui, "Font format", font.format.clone());
            detail_row(ui, "Rendering", "Native".to_owned());
        }
        PreviewContent::Shell(shell) => {
            detail_row(
                ui,
                "Preview",
                if shell.is_icon {
                    "System icon".to_owned()
                } else {
                    "System thumbnail".to_owned()
                },
            );
        }
        PreviewContent::Unsupported(_) => {}
    }

    if !result.warnings.is_empty() {
        ui.add_space(8.0);
        for warning in &result.warnings {
            warning_card(ui, "Preview note", warning);
            ui.add_space(4.0);
        }
    }
}
