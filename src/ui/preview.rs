use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use egui::{
    Align2, Color32, CornerRadius, FontId, Label, Rect, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, TextWrapMode, TextureHandle, TextureOptions, Ui, pos2, vec2,
};

use crate::preview::{
    ArchivePreview, AudioPreview, Bitmap, FontPreview, ImagePreview, PreviewContent, PreviewResult,
    ShellPreview, VideoPreview,
};
use crate::theme;
use crate::util;

use super::icons;
use super::widgets;

#[derive(Default)]
pub struct PreviewState {
    pub request_id: Option<u64>,
    pub path: Option<PathBuf>,
    pub result: Option<Arc<PreviewResult>>,
    pub textures: PreviewTextures,
    pub error: Option<String>,
    pub loading: bool,
    pub started_at: Option<Instant>,
}

#[derive(Default)]
pub struct PreviewTextures {
    image: Option<TextureHandle>,
    poster: Option<TextureHandle>,
    filmstrip: Vec<TextureHandle>,
    cover: Option<TextureHandle>,
    waveform: Option<TextureHandle>,
    specimen: Option<TextureHandle>,
    shell: Option<TextureHandle>,
}

impl PreviewState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn begin(&mut self, request_id: u64, path: PathBuf) {
        self.clear();
        self.request_id = Some(request_id);
        self.path = Some(path);
        self.loading = true;
        self.started_at = Some(Instant::now());
    }

    pub fn accept(&mut self, ctx: &egui::Context, result: Arc<PreviewResult>) {
        let request_id = self.request_id.unwrap_or(0);
        self.textures = PreviewTextures::build(ctx, &result, request_id);
        self.result = Some(result);
        self.loading = false;
        self.error = None;
    }

    pub fn fail(&mut self, error: String) {
        self.loading = false;
        self.result = None;
        self.error = Some(error);
    }

    pub fn show(&self, ui: &mut Ui) {
        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());
        if self.loading {
            self.show_loading(ui);
            return;
        }
        if let Some(error) = &self.error {
            self.show_error(ui, error);
            return;
        }
        let Some(result) = &self.result else {
            widgets::empty_state(ui, "No preview", "Select a file to preview it.");
            return;
        };

        match &result.content {
            PreviewContent::Image(image) => self.show_image(ui, image),
            PreviewContent::Text(text) => show_text(ui, text),
            PreviewContent::Video(video) => self.show_video(ui, video, &result.path),
            PreviewContent::Audio(audio) => self.show_audio(ui, audio, &result.path),
            PreviewContent::Archive(archive) => show_archive(ui, archive),
            PreviewContent::Font(font) => self.show_font(ui, font),
            PreviewContent::Shell(shell) => self.show_shell(ui, shell, &result.path),
            PreviewContent::Unsupported(unsupported) => {
                show_unsupported(ui, &unsupported.reason, Some(&result.path))
            }
        }
    }

    fn show_loading(&self, ui: &mut Ui) {
        let elapsed = self
            .started_at
            .map(|started| started.elapsed().as_millis())
            .unwrap_or(0);
        let rect = ui.available_rect_before_wrap();
        ui.allocate_rect(rect, Sense::hover());
        let painter = ui.painter();
        let spinner_center = pos2(rect.center().x, rect.center().y - 18.0);
        let spinner_rect = Rect::from_center_size(spinner_center, vec2(26.0, 26.0));
        let start = -std::f32::consts::FRAC_PI_2
            + (ui.input(|input| input.time) as f32 * 2.4) % (std::f32::consts::TAU);
        let points: Vec<egui::Pos2> = (0..26)
            .map(|index| {
                let angle = start + index as f32 * 0.16;
                spinner_center + vec2(angle.cos(), angle.sin()) * 12.0
            })
            .collect();
        painter.add(egui::Shape::line(points, Stroke::new(2.0, theme::ACCENT)));
        let _ = spinner_rect;
        painter.text(
            pos2(rect.center().x, rect.center().y + 18.0),
            Align2::CENTER_CENTER,
            if elapsed > 900 {
                "Preparing preview…"
            } else {
                "Loading preview…"
            },
            theme::semibold(13.5),
            theme::MUTED,
        );
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(40));
    }

    fn show_error(&self, ui: &mut Ui, error: &str) {
        let rect = ui.available_rect_before_wrap();
        ui.allocate_rect(rect, Sense::hover());
        let painter = ui.painter();
        icons::warning(
            painter,
            Rect::from_center_size(
                pos2(rect.center().x, rect.center().y - 24.0),
                vec2(34.0, 34.0),
            ),
            theme::MANUAL,
            2.0,
        );
        painter.text(
            pos2(rect.center().x, rect.center().y + 20.0),
            Align2::CENTER_CENTER,
            "Preview unavailable",
            theme::semibold(16.0),
            theme::TEXT,
        );
        painter.text(
            pos2(rect.center().x, rect.center().y + 44.0),
            Align2::CENTER_CENTER,
            error,
            FontId::proportional(13.0),
            theme::MUTED,
        );
    }

    fn show_image(&self, ui: &mut Ui, image: &ImagePreview) {
        let Some(texture) = &self.textures.image else {
            show_unsupported(
                ui,
                "The decoded image could not be uploaded.",
                self.path.as_deref(),
            );
            return;
        };
        let rect = ui.available_rect_before_wrap();
        ui.allocate_rect(rect, Sense::hover());
        paint_checkerboard(ui.painter(), rect, 16.0);
        draw_texture_fit(ui.painter(), rect, texture, 4.0);
        let caption = format!(
            "{} × {}  ·  {}",
            image.original_width, image.original_height, image.format
        );
        draw_caption(ui.painter(), rect, &caption);
        if image.animated {
            draw_tag(ui.painter(), rect, "Animated", theme::PURPLE);
        }
    }

    fn show_video(&self, ui: &mut Ui, video: &VideoPreview, path: &PathBuf) {
        let rect = ui.available_rect_before_wrap();
        ui.allocate_rect(rect, Sense::hover());
        let strip_height = if video.filmstrip.is_empty() {
            0.0
        } else {
            62.0
        };
        let main_rect = Rect::from_min_max(
            rect.min,
            pos2(
                rect.right(),
                rect.bottom() - strip_height - if strip_height > 0.0 { 12.0 } else { 0.0 },
            ),
        );
        paint_checkerboard(ui.painter(), main_rect, 16.0);

        if let Some(poster) = &self.textures.poster {
            draw_texture_fit(ui.painter(), main_rect, poster, 3.0);
            let dims = match (video.meta.width, video.meta.height) {
                (Some(width), Some(height)) => format!("{width} × {height}"),
                _ => "Video".to_owned(),
            };
            let duration = video
                .meta
                .duration_secs
                .map(util::format_seconds)
                .unwrap_or_else(|| "Unknown length".to_owned());
            draw_caption(ui.painter(), main_rect, &format!("{dims}  ·  {duration}"));
            let play_center = main_rect.center() + vec2(0.0, -2.0);
            let circle = Rect::from_center_size(play_center, vec2(58.0, 58.0));
            ui.painter().circle_filled(
                play_center,
                29.0,
                Color32::from_rgba_unmultiplied(0x0B, 0x0F, 0x14, 190),
            );
            ui.painter().circle_stroke(
                play_center,
                29.0,
                Stroke::new(1.5, theme::alpha(theme::ACCENT, 150)),
            );
            icons::play(ui.painter(), circle.shrink(18.0), theme::ACCENT, 1.0);
            let _ = path;
        } else {
            let painter = ui.painter();
            icons::play(
                painter,
                Rect::from_center_size(main_rect.center() - vec2(0.0, 14.0), vec2(50.0, 50.0)),
                theme::alpha(theme::DELETE, 170),
                1.0,
            );
            painter.text(
                main_rect.center() + vec2(0.0, 34.0),
                Align2::CENTER_CENTER,
                "No frame preview",
                theme::semibold(14.0),
                theme::MUTED,
            );
        }

        if !self.textures.filmstrip.is_empty() {
            let mut x = main_rect.left();
            let y = main_rect.bottom() + 8.0;
            let thumb_width = 108.0;
            let thumb_height = strip_height - 10.0;
            for texture in &self.textures.filmstrip {
                let thumb_rect = Rect::from_min_size(pos2(x, y), vec2(thumb_width, thumb_height));
                draw_texture_fit_rounded(ui.painter(), thumb_rect, texture, 4.0, 7);
                x += thumb_width + 8.0;
                if x > main_rect.right() {
                    break;
                }
            }
        }
    }

    fn show_audio(&self, ui: &mut Ui, audio: &AudioPreview, path: &Path) {
        let rect = ui.available_rect_before_wrap();
        ui.allocate_rect(rect, Sense::hover());
        let painter = ui.painter();
        let cover_size = (rect.height() * 0.5)
            .clamp(150.0, 260.0)
            .min(rect.width() * 0.5);
        let cover_rect = Rect::from_center_size(
            pos2(
                rect.left() + cover_size * 0.5 + 8.0,
                rect.top() + cover_size * 0.5 + 8.0,
            ),
            vec2(cover_size, cover_size),
        );

        if let Some(cover) = &self.textures.cover {
            draw_texture_fit_rounded(painter, cover_rect, cover, 2.0, 14);
        } else {
            painter.rect_filled(
                cover_rect,
                CornerRadius::same(14),
                theme::alpha(theme::MANUAL, 16),
            );
            painter.rect_stroke(
                cover_rect,
                CornerRadius::same(14),
                Stroke::new(1.0, theme::alpha(theme::MANUAL, 45)),
                StrokeKind::Inside,
            );
            icons::note(
                painter,
                cover_rect.shrink(cover_size * 0.28),
                theme::alpha(theme::MANUAL, 190),
                2.0,
            );
        }

        let text_left = cover_rect.right() + 26.0;
        if text_left < rect.right() {
            let title = audio.meta.title.clone().unwrap_or_else(|| {
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Audio".to_owned())
            });
            painter.text(
                pos2(text_left, cover_rect.top() + 26.0),
                Align2::LEFT_CENTER,
                util::truncate_middle(&title, 40),
                theme::semibold(22.0),
                theme::TEXT,
            );
            if let Some(artist) = &audio.meta.artist {
                painter.text(
                    pos2(text_left, cover_rect.top() + 58.0),
                    Align2::LEFT_CENTER,
                    util::truncate_middle(artist, 44),
                    FontId::proportional(14.0),
                    theme::MUTED,
                );
            }
            if let Some(album) = &audio.meta.album {
                painter.text(
                    pos2(text_left, cover_rect.top() + 82.0),
                    Align2::LEFT_CENTER,
                    util::truncate_middle(album, 44),
                    FontId::proportional(13.0),
                    theme::FAINT,
                );
            }
        }

        let wave_rect = Rect::from_min_max(
            pos2(rect.left() + 8.0, cover_rect.bottom() + 26.0),
            pos2(rect.right() - 8.0, rect.bottom() - 8.0),
        );
        if wave_rect.height() > 40.0 {
            if let Some(waveform) = &self.textures.waveform {
                painter.rect_filled(
                    wave_rect,
                    CornerRadius::same(10),
                    Color32::from_rgb(0x0C, 0x13, 0x1B),
                );
                draw_texture_fit_rounded(painter, wave_rect.shrink(8.0), waveform, 1.0, 8);
            } else {
                paint_fake_waveform(painter, wave_rect, theme::alpha(theme::MANUAL, 150));
            }
        }
    }

    fn show_font(&self, ui: &mut Ui, font: &FontPreview) {
        let Some(texture) = &self.textures.specimen else {
            show_unsupported(ui, "The font could not be rendered.", self.path.as_deref());
            return;
        };
        let rect = ui.available_rect_before_wrap();
        ui.allocate_rect(rect, Sense::hover());
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(10),
            Color32::from_rgb(0x0C, 0x13, 0x1B),
        );
        draw_texture_fit(ui.painter(), rect.shrink(10.0), texture, 2.0);
        draw_caption(ui.painter(), rect, &format!("{} font", font.format));
    }

    fn show_shell(&self, ui: &mut Ui, shell: &ShellPreview, path: &Path) {
        let Some(texture) = &self.textures.shell else {
            show_unsupported(
                ui,
                shell
                    .note
                    .as_deref()
                    .unwrap_or("No system preview is available."),
                Some(path),
            );
            return;
        };
        let rect = ui.available_rect_before_wrap();
        ui.allocate_rect(rect, Sense::hover());
        paint_checkerboard(ui.painter(), rect, 16.0);
        let image_rect = draw_texture_fit(ui.painter(), rect, texture, 3.0);
        if shell.is_icon {
            draw_tag(ui.painter(), image_rect, "System icon", theme::MUTED);
        }
        if let Some(note) = &shell.note {
            let painter = ui.painter();
            let note_rect = Rect::from_min_max(
                pos2(rect.left(), rect.bottom() - 36.0),
                pos2(rect.right(), rect.bottom() - 8.0),
            );
            painter.rect_filled(
                note_rect,
                CornerRadius::same(8),
                Color32::from_rgba_unmultiplied(0x0B, 0x0F, 0x14, 220),
            );
            painter.text(
                note_rect.center(),
                Align2::CENTER_CENTER,
                util::truncate_middle(note, 110),
                FontId::proportional(12.5),
                theme::MUTED,
            );
        }
    }
}

impl PreviewTextures {
    fn build(ctx: &egui::Context, result: &PreviewResult, request_id: u64) -> Self {
        let mut textures = Self::default();
        match &result.content {
            PreviewContent::Image(image) => {
                textures.image = load_bitmap(ctx, request_id, "image", &image.bitmap);
            }
            PreviewContent::Video(video) => {
                if let Some(poster) = &video.poster {
                    textures.poster = load_bitmap(ctx, request_id, "poster", poster);
                }
                for (index, frame) in video.filmstrip.iter().enumerate() {
                    if let Some(texture) =
                        load_bitmap(ctx, request_id * 100 + index as u64, "frame", frame)
                    {
                        textures.filmstrip.push(texture);
                    }
                }
            }
            PreviewContent::Audio(audio) => {
                if let Some(cover) = &audio.cover {
                    textures.cover = load_bitmap(ctx, request_id, "cover", cover);
                }
                if let Some(waveform) = &audio.waveform {
                    textures.waveform = load_bitmap(ctx, request_id, "waveform", waveform);
                }
            }
            PreviewContent::Font(font) => {
                textures.specimen = load_bitmap(ctx, request_id, "specimen", &font.specimen);
            }
            PreviewContent::Shell(shell) => {
                if let Some(image) = &shell.image {
                    textures.shell = load_bitmap(ctx, request_id, "shell", image);
                }
            }
            _ => {}
        }
        textures
    }
}

fn load_bitmap(
    ctx: &egui::Context,
    request_id: u64,
    name: &str,
    bitmap: &Bitmap,
) -> Option<TextureHandle> {
    if bitmap.width == 0 || bitmap.height == 0 {
        return None;
    }
    let expected = bitmap.width as usize * bitmap.height as usize * 4;
    if bitmap.rgba.len() != expected {
        return None;
    }
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [bitmap.width as usize, bitmap.height as usize],
        &bitmap.rgba,
    );
    Some(ctx.load_texture(
        format!("swipeaway-{request_id}-{name}"),
        image,
        TextureOptions::LINEAR,
    ))
}

fn paint_checkerboard(painter: &egui::Painter, rect: Rect, cell: f32) {
    painter.rect_filled(
        rect,
        CornerRadius::same(10),
        Color32::from_rgb(0x0A, 0x10, 0x17),
    );
    let light = Color32::from_rgb(0x0E, 0x15, 0x1E);
    let mut y = rect.top();
    let mut row = 0;
    while y < rect.bottom() {
        let mut x = rect.left();
        let mut column = 0;
        while x < rect.right() {
            if (row + column) % 2 == 0 {
                let square = Rect::from_min_max(
                    pos2(x, y),
                    pos2((x + cell).min(rect.right()), (y + cell).min(rect.bottom())),
                );
                painter.rect_filled(square, 0, light);
            }
            x += cell;
            column += 1;
        }
        y += cell;
        row += 1;
    }
}

fn draw_texture_fit(
    painter: &egui::Painter,
    bounds: Rect,
    texture: &TextureHandle,
    max_scale: f32,
) -> Rect {
    draw_texture_fit_rounded(painter, bounds, texture, max_scale, 0)
}

fn draw_texture_fit_rounded(
    painter: &egui::Painter,
    bounds: Rect,
    texture: &TextureHandle,
    max_scale: f32,
    radius: u8,
) -> Rect {
    let texture_size = texture.size_vec2();
    if texture_size.x <= 0.0 || texture_size.y <= 0.0 {
        return Rect::ZERO;
    }
    let available = bounds.shrink(10.0);
    let scale = (available.width() / texture_size.x)
        .min(available.height() / texture_size.y)
        .min(max_scale)
        .max(0.0);
    let size = texture_size * scale;
    let rect = Rect::from_center_size(available.center(), size);
    painter.rect_filled(
        rect.expand(6.0),
        CornerRadius::same(radius.saturating_add(6)),
        Color32::from_rgba_unmultiplied(0, 0, 0, 70),
    );
    painter.image(
        texture.id(),
        rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    if radius > 0 {
        painter.rect_stroke(
            rect,
            CornerRadius::same(radius),
            Stroke::new(1.0, theme::alpha(Color32::WHITE, 22)),
            StrokeKind::Inside,
        );
    }
    rect
}

fn draw_caption(painter: &egui::Painter, rect: Rect, text: &str) {
    let font = theme::mono(10.5);
    let galley = painter.layout_no_wrap(text.to_owned(), font.clone(), theme::MUTED);
    let pad = vec2(10.0, 5.0);
    let tag_rect = Rect::from_min_size(
        pos2(
            rect.right() - galley.size().x - pad.x * 2.0 - 8.0,
            rect.bottom() - galley.size().y - pad.y * 2.0 - 8.0,
        ),
        galley.size() + pad * 2.0,
    );
    painter.rect_filled(
        tag_rect,
        CornerRadius::same(6),
        Color32::from_rgba_unmultiplied(0x0B, 0x0F, 0x14, 220),
    );
    painter.text(
        tag_rect.center(),
        Align2::CENTER_CENTER,
        text,
        font,
        theme::MUTED,
    );
}

fn draw_tag(painter: &egui::Painter, rect: Rect, text: &str, color: Color32) {
    let font = theme::semibold(10.5);
    let galley = painter.layout_no_wrap(text.to_owned(), font.clone(), color);
    let tag_rect = Rect::from_min_size(
        pos2(rect.left() + 8.0, rect.top() + 8.0),
        galley.size() + vec2(16.0, 8.0),
    );
    painter.rect_filled(tag_rect, CornerRadius::same(6), theme::alpha(color, 30));
    painter.text(tag_rect.center(), Align2::CENTER_CENTER, text, font, color);
}

fn show_text(ui: &mut Ui, text: &crate::preview::text::TextPreview) {
    ui.horizontal_wrapped(|ui| {
        widgets::plain_chip(ui, text.language, theme::KEEP);
        widgets::plain_chip(ui, &text.encoding, theme::BLUE);
        widgets::plain_chip(ui, &format!("{} lines", text.lines_shown), theme::MUTED);
        if text.file_truncated || text.display_truncated {
            widgets::plain_chip(ui, "Truncated", theme::MANUAL);
        }
    });
    ui.add_space(6.0);
    let available = ui.available_size();
    egui::Frame::NONE
        .fill(Color32::from_rgb(0x0A, 0x10, 0x17))
        .stroke(Stroke::new(1.0, theme::STROKE_SOFT))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_min_size(vec2(available.x - 22.0, available.y - 22.0));
            ScrollArea::both()
                .id_salt("text_preview_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if text.text.is_empty() {
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("This file is empty.")
                                .color(theme::MUTED)
                                .font(FontId::proportional(13.0)),
                        );
                        return;
                    }
                    let width = text.lines_shown.max(1).to_string().len();
                    let mut rendered =
                        String::with_capacity(text.text.len() + text.lines_shown * (width + 3));
                    for (index, line) in text.text.split('\n').enumerate() {
                        rendered.push_str(&format!(
                            "{:>width$}  {line}\n",
                            index + 1,
                            line = line,
                            width = width
                        ));
                    }
                    ui.add(
                        Label::new(
                            RichText::new(rendered)
                                .font(theme::mono(12.8))
                                .color(theme::TEXT_SOFT),
                        )
                        .selectable(true)
                        .wrap_mode(TextWrapMode::Extend),
                    );
                });
        });
}

fn show_archive(ui: &mut Ui, archive: &ArchivePreview) {
    ui.horizontal_wrapped(|ui| {
        widgets::plain_chip(
            ui,
            &format!("{} entries", archive.total_entries),
            theme::MANUAL,
        );
        widgets::plain_chip(ui, &util::format_size(archive.shown_size), theme::BLUE);
        if archive.truncated {
            widgets::plain_chip(ui, "First 500 shown", theme::MUTED);
        }
    });
    ui.add_space(6.0);
    let available = ui.available_size();
    egui::Frame::NONE
        .fill(Color32::from_rgb(0x0A, 0x10, 0x17))
        .stroke(Stroke::new(1.0, theme::STROKE_SOFT))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_size(vec2(available.x - 18.0, available.y - 18.0));
            ScrollArea::vertical()
                .id_salt("archive_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for entry in &archive.entries {
                        let (rect, _) = ui
                            .allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
                        let painter = ui.painter();
                        let color = if entry.is_dir {
                            theme::MANUAL
                        } else {
                            theme::MUTED
                        };
                        let icon_rect = Rect::from_center_size(
                            pos2(rect.left() + 13.0, rect.center().y),
                            vec2(14.0, 14.0),
                        );
                        if entry.is_dir {
                            icons::folder(painter, icon_rect, color, 1.4);
                        } else {
                            icons::document(painter, icon_rect, color, 1.2);
                        }
                        painter.text(
                            pos2(rect.left() + 28.0, rect.center().y),
                            Align2::LEFT_CENTER,
                            util::truncate_middle(&entry.name, 92),
                            theme::mono(11.8),
                            theme::TEXT_SOFT,
                        );
                        if !entry.is_dir {
                            painter.text(
                                pos2(rect.right() - 4.0, rect.center().y),
                                Align2::RIGHT_CENTER,
                                util::format_size_short(entry.size),
                                theme::mono(10.5),
                                theme::FAINT,
                            );
                        }
                    }
                });
        });
}

fn show_unsupported(ui: &mut Ui, reason: &str, path: Option<&std::path::Path>) {
    let rect = ui.available_rect_before_wrap();
    ui.allocate_rect(rect, Sense::hover());
    let painter = ui.painter();
    let badge_size = 84.0;
    let badge_rect = Rect::from_center_size(
        pos2(rect.center().x, rect.center().y - 36.0),
        vec2(badge_size, badge_size),
    );
    let extension = path
        .map(util::extension_upper)
        .unwrap_or_else(|| "FILE".to_owned());
    painter.rect_filled(
        badge_rect,
        CornerRadius::same(20),
        theme::alpha(theme::MUTED, 18),
    );
    painter.rect_stroke(
        badge_rect,
        CornerRadius::same(20),
        Stroke::new(1.0, theme::alpha(theme::MUTED, 55)),
        StrokeKind::Inside,
    );
    painter.text(
        badge_rect.center(),
        Align2::CENTER_CENTER,
        extension,
        theme::semibold(20.0),
        theme::TEXT_SOFT,
    );
    painter.text(
        pos2(rect.center().x, rect.center().y + 26.0),
        Align2::CENTER_CENTER,
        "No rich preview",
        theme::semibold(16.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.center().x, rect.center().y + 49.0),
        Align2::CENTER_CENTER,
        util::truncate_middle(reason, 100),
        FontId::proportional(12.5),
        theme::MUTED,
    );
}

fn paint_fake_waveform(painter: &egui::Painter, rect: Rect, color: Color32) {
    painter.rect_filled(
        rect,
        CornerRadius::same(10),
        Color32::from_rgb(0x0C, 0x13, 0x1B),
    );
    let bars = 96usize;
    let gap = 3.0;
    let bar_width = ((rect.width() - 16.0) / bars as f32 - gap).max(1.0);
    let center_y = rect.center().y;
    for index in 0..bars {
        let phase = index as f32 * 0.61;
        let envelope = ((index as f32 / bars as f32) * std::f32::consts::PI).sin();
        let amplitude = (phase.sin().abs() * 0.55 + (phase * 2.7).cos().abs() * 0.3 + 0.15)
            * envelope
            * rect.height()
            * 0.36;
        let x = rect.left() + 8.0 + index as f32 * (bar_width + gap);
        let bar = Rect::from_min_max(
            pos2(x, center_y - amplitude),
            pos2(x + bar_width, center_y + amplitude),
        );
        painter.rect_filled(bar, 1, theme::alpha(color, 150));
    }
}
