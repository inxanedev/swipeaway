use std::path::{Path, PathBuf};

use egui::{
    Align2, Color32, CornerRadius, FontId, Rect, RichText, ScrollArea, Sense, Stroke, Ui, pos2,
    vec2,
};

use crate::app::{SwipeAwayApp, ToastKind};
use crate::core::entry::Entry;
use crate::core::scan;
use crate::theme;
use crate::util;

use super::icons;
use super::widgets;

#[derive(Default)]
pub struct FolderState {
    pub rel: PathBuf,
    pub listing: Option<FolderListing>,
    pub selected: Option<PathBuf>,
}

#[derive(Clone, Default)]
pub struct FolderListing {
    pub entries: Vec<Entry>,
    pub total: usize,
    pub warnings: Vec<String>,
    pub error: Option<String>,
}

impl FolderState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn load(&mut self, root: &Path) {
        let dir = if self.rel.as_os_str().is_empty() {
            root.to_path_buf()
        } else {
            root.join(&self.rel)
        };

        match scan::scan(&dir) {
            Ok(report) => {
                let total = report.entries.len();
                let mut entries = report.entries;
                entries.truncate(1200);
                self.listing = Some(FolderListing {
                    entries,
                    total,
                    warnings: report.warnings,
                    error: None,
                });
            }
            Err(error) => {
                self.listing = Some(FolderListing {
                    error: Some(format!("Could not read this folder: {error}")),
                    ..FolderListing::default()
                });
            }
        }
    }

    pub fn navigate(&mut self, root: &Path, rel: PathBuf) {
        self.rel = rel;
        self.selected = None;
        self.listing = None;
        self.load(root);
    }

    pub fn absolute(&self, root: &Path) -> PathBuf {
        if self.rel.as_os_str().is_empty() {
            root.to_path_buf()
        } else {
            root.join(&self.rel)
        }
    }
}

#[derive(Debug)]
enum FolderAction {
    Navigate(PathBuf),
    Select(PathBuf),
    OpenInstance(PathBuf),
    ShowInExplorer(PathBuf),
    Open(PathBuf),
}

impl SwipeAwayApp {
    pub(crate) fn ui_folder_explorer(&mut self, ui: &mut Ui) {
        let Some(entry) = self
            .session
            .as_ref()
            .and_then(|session| session.current().cloned())
        else {
            return;
        };
        let base = entry.path.clone();

        if self.folder.listing.is_none() {
            self.folder.load(&base);
        }

        let listing = self.folder.listing.clone().unwrap_or_default();
        let rel = self.folder.rel.clone();
        let selected = self.folder.selected.clone();
        let selected_entry = selected.as_ref().and_then(|path| {
            listing
                .entries
                .iter()
                .find(|entry| entry.path == *path)
                .cloned()
        });

        let mut actions = Vec::new();

        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("↑  Up")
                            .font(theme::semibold(12.5))
                            .color(theme::TEXT_SOFT),
                    )
                    .fill(theme::PANEL_RAISED)
                    .stroke(Stroke::new(1.0, theme::STROKE_SOFT))
                    .corner_radius(CornerRadius::same(9))
                    .min_size(vec2(62.0, 30.0)),
                )
                .clicked()
            {
                let parent = rel.parent().map(Path::to_path_buf).unwrap_or_default();
                actions.push(FolderAction::Navigate(parent));
            }

            ui.spacing_mut().item_spacing.x = 2.0;
            let root_name = entry.name.clone();
            if widgets::ghost_button(ui, &root_name, theme::TEXT_SOFT).clicked() {
                actions.push(FolderAction::Navigate(PathBuf::new()));
            }
            let mut prefix = PathBuf::new();
            for component in rel.components() {
                prefix.push(component);
                ui.label(
                    RichText::new("/")
                        .color(theme::FAINT)
                        .font(FontId::proportional(12.0)),
                );
                let label = component.as_os_str().to_string_lossy().to_string();
                if widgets::ghost_button(ui, &label, theme::TEXT_SOFT).clicked() {
                    actions.push(FolderAction::Navigate(prefix.clone()));
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::secondary_button(
                    ui,
                    "Clean this folder",
                    Some(icons::open_external),
                    theme::ACCENT,
                    "Open another SwipeAway window pointed at this folder",
                )
                .clicked()
                {
                    actions.push(FolderAction::OpenInstance(self.folder.absolute(&base)));
                }
            });
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            widgets::plain_chip(ui, &util::item_count(listing.total), theme::MUTED);
            ui.label(
                RichText::new(
                    "Read-only preview · keep / delete / set aside applies to the whole folder",
                )
                .font(FontId::proportional(11.5))
                .color(theme::FAINT),
            );
        });
        ui.add_space(6.0);

        let body_height = ui.available_height();
        let gap = 12.0;
        let list_width = (ui.available_width() * 0.43).clamp(250.0, 430.0);
        let preview_width = ui.available_width() - list_width - gap;

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.allocate_ui_with_layout(
                vec2(list_width, body_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    widgets::inset_frame().show(ui, |ui| {
                        ui.set_min_size(vec2(list_width - 28.0, body_height - 28.0));
                        if let Some(error) = &listing.error {
                            ui.label(
                                RichText::new(error)
                                    .color(theme::DELETE)
                                    .font(FontId::proportional(12.5)),
                            );
                            return;
                        }
                        if listing.entries.is_empty() {
                            ui.add_space(20.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new("This folder is empty.")
                                        .color(theme::MUTED)
                                        .font(FontId::proportional(13.0)),
                                );
                            });
                            return;
                        }
                        let list_height = ui.available_height();
                        ScrollArea::vertical()
                            .id_salt("folder_children")
                            .auto_shrink([false, false])
                            .max_height(list_height)
                            .show(ui, |ui| {
                                for child in &listing.entries {
                                    let is_selected =
                                        selected.as_ref().is_some_and(|path| path == &child.path);
                                    if folder_row(ui, child, is_selected).clicked() {
                                        if child.is_dir {
                                            let next = rel.join(&child.name);
                                            actions.push(FolderAction::Navigate(next));
                                        } else {
                                            actions.push(FolderAction::Select(child.path.clone()));
                                        }
                                    }
                                }
                                if listing.total > listing.entries.len() {
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "Showing first {} of {} items",
                                            listing.entries.len(),
                                            listing.total
                                        ))
                                        .color(theme::FAINT)
                                        .font(FontId::proportional(11.0)),
                                    );
                                }
                            });
                    });
                },
            );

            ui.allocate_ui_with_layout(
                vec2(preview_width, body_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    widgets::inset_frame().show(ui, |ui| {
                        ui.set_min_size(vec2(preview_width - 28.0, body_height - 28.0));
                        if let Some(selected_entry) = &selected_entry {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(util::truncate_middle(&selected_entry.name, 48))
                                        .font(theme::semibold(14.5))
                                        .color(theme::TEXT),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if widgets::ghost_button(ui, "Show in folder", theme::MUTED)
                                            .clicked()
                                        {
                                            actions.push(FolderAction::ShowInExplorer(
                                                selected_entry.path.clone(),
                                            ));
                                        }
                                        if widgets::ghost_button(ui, "Open", theme::MUTED).clicked()
                                        {
                                            actions.push(FolderAction::Open(
                                                selected_entry.path.clone(),
                                            ));
                                        }
                                        ui.label(
                                            RichText::new(util::format_size(selected_entry.size))
                                                .font(theme::mono(10.5))
                                                .color(theme::FAINT),
                                        );
                                    },
                                );
                            });
                            ui.add_space(4.0);
                            let preview_height = ui.available_height();
                            ui.allocate_ui_with_layout(
                                vec2(ui.available_width(), preview_height),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    self.preview.show(ui);
                                },
                            );
                        } else {
                            let rect = ui.available_rect_before_wrap();
                            ui.allocate_rect(rect, Sense::hover());
                            let painter = ui.painter();
                            icons::folder(
                                painter,
                                Rect::from_center_size(
                                    pos2(rect.center().x, rect.center().y - 22.0),
                                    vec2(42.0, 42.0),
                                ),
                                theme::alpha(theme::ACCENT, 130),
                                1.8,
                            );
                            painter.text(
                                pos2(rect.center().x, rect.center().y + 22.0),
                                Align2::CENTER_CENTER,
                                "Pick a file to preview it",
                                theme::semibold(14.0),
                                theme::TEXT_SOFT,
                            );
                            painter.text(
                                pos2(rect.center().x, rect.center().y + 44.0),
                                Align2::CENTER_CENTER,
                                "Folders open in the list.",
                                FontId::proportional(12.0),
                                theme::MUTED,
                            );
                        }
                    });
                },
            );
        });

        for action in actions {
            match action {
                FolderAction::Navigate(rel) => {
                    self.folder.navigate(&base, rel);
                    self.preview.clear();
                }
                FolderAction::Select(path) => {
                    self.folder.selected = Some(path.clone());
                    self.request_preview(path);
                }
                FolderAction::OpenInstance(path) => match util::spawn_instance(&path) {
                    Ok(()) => self.set_toast(
                        "Opened this folder in a new SwipeAway window.".to_owned(),
                        ToastKind::Success,
                    ),
                    Err(error) => self.set_toast(
                        format!("Could not open a new window: {error}"),
                        ToastKind::Error,
                    ),
                },
                FolderAction::ShowInExplorer(path) => self.show_in_folder(&path),
                FolderAction::Open(path) => self.open_with_default(&path),
            }
        }
    }
}

fn folder_row(ui: &mut Ui, entry: &Entry, selected: bool) -> egui::Response {
    let height = 32.0;
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
    let hover =
        ui.ctx()
            .animate_bool_with_time(response.id.with("row-hover"), response.hovered(), 0.08);
    let painter = ui.painter();
    let fill = if selected {
        theme::alpha(theme::ACCENT, 26)
    } else if hover > 0.0 {
        theme::alpha(theme::TEXT, (10.0 * hover) as u8)
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, CornerRadius::same(7), fill);
    if selected {
        painter.rect_filled(
            Rect::from_min_size(rect.left_top(), vec2(2.0, rect.height())),
            1,
            theme::ACCENT,
        );
    }

    let color = if entry.is_dir {
        theme::MANUAL
    } else if entry.hidden || entry.system {
        theme::MUTED
    } else {
        theme::TEXT_SOFT
    };
    let icon_rect =
        Rect::from_center_size(pos2(rect.left() + 14.0, rect.center().y), vec2(15.0, 15.0));
    if entry.is_dir {
        icons::folder(painter, icon_rect, color, 1.5);
    } else {
        icons::document(painter, icon_rect, color, 1.2);
    }
    painter.text(
        pos2(rect.left() + 28.0, rect.center().y),
        Align2::LEFT_CENTER,
        util::truncate_middle(&entry.name, 52),
        FontId::proportional(12.5),
        if entry.is_dir {
            theme::TEXT
        } else {
            theme::TEXT_SOFT
        },
    );
    if entry.hidden || entry.system {
        painter.circle_filled(
            pos2(rect.right() - 58.0, rect.center().y),
            2.5,
            theme::MANUAL,
        );
    }
    painter.text(
        pos2(rect.right() - 6.0, rect.center().y),
        Align2::RIGHT_CENTER,
        if entry.is_dir {
            "Folder".to_owned()
        } else {
            util::format_size_short(entry.size)
        },
        theme::mono(10.0),
        theme::FAINT,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
