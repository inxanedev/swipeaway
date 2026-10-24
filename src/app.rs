use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::App;
use egui::{Align2, Color32, CornerRadius, Frame, Margin, Order, RichText, Stroke, Vec2};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};

use crate::config::Config;
use crate::core::scan;
use crate::core::session::{Decision, Session};
use crate::preview::engine::{PreviewEngine, PreviewEvent};
use crate::safety;
use crate::theme;
use crate::ui::folder_explorer::FolderState;
use crate::ui::icons;
use crate::ui::preview::PreviewState;
use crate::ui::widgets;
use crate::util;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Decision,
    Summary,
}

#[derive(Debug, Clone, Copy)]
pub enum PendingAction {
    Decide(Decision),
    Undo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    fn color(self) -> Color32 {
        match self {
            Self::Info => theme::BLUE,
            Self::Success => theme::KEEP,
            Self::Warning => theme::MANUAL,
            Self::Error => theme::DELETE,
        }
    }
}

struct Toast {
    message: String,
    kind: ToastKind,
    created: Instant,
    ttl: Duration,
}

pub struct SwipeAwayApp {
    pub screen: Screen,
    pub session: Option<Session>,
    pub engine: PreviewEngine,
    pub preview: PreviewState,
    pub folder: FolderState,
    pub config: Config,
    pub pending: Option<PendingAction>,
    pub auto_pick: bool,
    pub startup_error: Option<String>,
    pub confirm_leave: bool,
    pub leave_then_pick: bool,
    pub show_shortcuts: bool,
    parent_window: Option<ParentWindow>,
    startup_frames: u8,
    toast: Option<Toast>,
}

struct ParentWindow {
    window: RawWindowHandle,
    display: RawDisplayHandle,
}

impl HasWindowHandle for ParentWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        unsafe { Ok(WindowHandle::borrow_raw(self.window)) }
    }
}

impl HasDisplayHandle for ParentWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe { Ok(DisplayHandle::borrow_raw(self.display)) }
    }
}

impl SwipeAwayApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial: Option<PathBuf>) -> Self {
        theme::apply(&cc.egui_ctx);
        let config = Config::load();
        let engine = PreviewEngine::new(cc.egui_ctx.clone());
        let parent_window = {
            let window = cc.window_handle().ok().map(|handle| handle.as_raw());
            let display = cc.display_handle().ok().map(|handle| handle.as_raw());
            match (window, display) {
                (Some(window), Some(display)) => Some(ParentWindow { window, display }),
                _ => None,
            }
        };

        let mut app = Self {
            screen: Screen::Welcome,
            session: None,
            engine,
            preview: PreviewState::default(),
            folder: FolderState::default(),
            config,
            pending: None,
            auto_pick: initial.is_none(),
            startup_error: None,
            confirm_leave: false,
            leave_then_pick: false,
            show_shortcuts: false,
            parent_window,
            startup_frames: 0,
            toast: None,
        };

        if let Some(initial) = initial {
            let target = if initial.is_file() {
                initial.parent().map(Path::to_path_buf).unwrap_or(initial)
            } else {
                initial
            };
            if !app.try_start_session(target, &cc.egui_ctx) {
                app.auto_pick = false;
            }
        }

        app
    }

    pub fn try_start_session(&mut self, path: PathBuf, ctx: &egui::Context) -> bool {
        match safety::validate_target(&path) {
            Ok(root) => match scan::scan(&root) {
                Ok(report) => {
                    let warnings = report.warnings.clone();
                    let count = report.entries.len();
                    self.startup_error = None;
                    self.config.remember(&root);
                    let session = Session::new(root, report);
                    self.session = Some(session);
                    self.screen = Screen::Decision;
                    self.auto_pick = false;
                    self.refresh_current(ctx);
                    if count == 0 {
                        self.set_toast(
                            "This folder has nothing to clean.".to_owned(),
                            ToastKind::Info,
                        );
                    } else {
                        self.set_toast(
                            format!("Loaded {count} item{}.", if count == 1 { "" } else { "s" }),
                            ToastKind::Info,
                        );
                    }
                    if !warnings.is_empty() {
                        self.set_toast(
                            format!(
                                "{} item(s) could not be read and were skipped.",
                                warnings.len()
                            ),
                            ToastKind::Warning,
                        );
                    }
                    true
                }
                Err(error) => {
                    self.startup_error = Some(format!("Could not read this folder: {error}"));
                    self.screen = Screen::Welcome;
                    false
                }
            },
            Err(error) => {
                self.startup_error = Some(error.message);
                self.screen = Screen::Welcome;
                false
            }
        }
    }

    pub fn leave_session(&mut self, ctx: &egui::Context, then_pick: bool) {
        self.session = None;
        self.screen = Screen::Welcome;
        self.preview.clear();
        self.folder.reset();
        self.pending = None;
        self.auto_pick = false;
        if then_pick {
            self.open_picker(ctx);
        }
    }

    pub fn request_leave(&mut self, ctx: &egui::Context, then_pick: bool) {
        let has_progress = self
            .session
            .as_ref()
            .map(|session| !session.history.is_empty())
            .unwrap_or(false);
        if has_progress {
            self.confirm_leave = true;
            self.leave_then_pick = then_pick;
        } else {
            self.leave_session(ctx, then_pick);
        }
    }

    pub fn open_picker(&mut self, ctx: &egui::Context) {
        let default_dir = self.config.last_root.clone().filter(|path| path.exists());
        let mut dialog = rfd::FileDialog::new().set_title("Choose a folder for SwipeAway");
        if let Some(dir) = default_dir {
            dialog = dialog.set_directory(dir);
        }
        if let Some(parent) = &self.parent_window {
            dialog = dialog.set_parent(parent);
        }
        if let Some(path) = dialog.pick_folder() {
            self.try_start_session(path, ctx);
        }
    }

    pub fn refresh_current(&mut self, ctx: &egui::Context) {
        self.folder.reset();
        self.preview.clear();
        let Some(session) = &self.session else {
            return;
        };
        let Some(entry) = session.current().cloned() else {
            return;
        };
        if entry.is_dir {
            self.folder.load(&entry.path);
        } else {
            self.request_preview(entry.path);
        }
        let _ = ctx;
    }

    pub fn request_preview(&mut self, path: PathBuf) {
        let request_id = self.engine.request(path.clone());
        self.preview.begin(request_id, path);
    }

    pub fn apply_decision(&mut self, decision: Decision, ctx: &egui::Context) {
        let Some(session) = &mut self.session else {
            return;
        };
        match session.decide(decision) {
            Ok(outcome) => {
                let message = match decision {
                    Decision::Keep => None,
                    Decision::Junk => {
                        Some(format!("Moved {} to swipeaway-junk.", outcome.entry.name))
                    }
                    Decision::Manual => Some(format!(
                        "Set {} aside in swipeaway-manual.",
                        outcome.entry.name
                    )),
                };
                let finished = session.is_finished();
                if let Some(message) = message {
                    self.set_toast(message, ToastKind::Success);
                }
                if finished {
                    self.screen = Screen::Summary;
                    self.preview.clear();
                    self.folder.reset();
                } else {
                    self.refresh_current(ctx);
                }
            }
            Err(error) => self.set_toast(error, ToastKind::Error),
        }
    }

    pub fn undo(&mut self, ctx: &egui::Context) {
        let Some(session) = &mut self.session else {
            return;
        };
        match session.undo_last() {
            Ok(outcome) => {
                let message = match outcome.decision {
                    Decision::Keep => format!("Undid keeping {}.", outcome.entry.name),
                    Decision::Junk => {
                        format!("Restored {} from swipeaway-junk.", outcome.entry.name)
                    }
                    Decision::Manual => {
                        format!("Restored {} from swipeaway-manual.", outcome.entry.name)
                    }
                };
                self.screen = Screen::Decision;
                self.set_toast(message, ToastKind::Info);
                self.refresh_current(ctx);
            }
            Err(error) => self.set_toast(error, ToastKind::Error),
        }
    }

    pub fn set_toast(&mut self, message: String, kind: ToastKind) {
        let ttl = match kind {
            ToastKind::Error => Duration::from_secs(5),
            ToastKind::Warning => Duration::from_secs(4),
            ToastKind::Info | ToastKind::Success => Duration::from_millis(2500),
        };
        self.toast = Some(Toast {
            message,
            kind,
            created: Instant::now(),
            ttl,
        });
    }

    pub fn show_in_folder(&mut self, path: &Path) {
        if let Err(error) = util::show_in_explorer(path) {
            self.set_toast(
                format!("Could not open Explorer: {error}"),
                ToastKind::Error,
            );
        }
    }

    pub fn open_with_default(&mut self, path: &Path) {
        if let Err(error) = util::open_path(path) {
            self.set_toast(
                format!("Could not open this file: {error}"),
                ToastKind::Error,
            );
        }
    }

    pub fn open_folder(&mut self, path: &Path) {
        if !path.exists() {
            self.set_toast(
                "That folder hasn't been created yet.".to_owned(),
                ToastKind::Warning,
            );
            return;
        }
        if let Err(error) = util::open_folder(path) {
            self.set_toast(
                format!("Could not open this folder: {error}"),
                ToastKind::Error,
            );
        }
    }

    fn poll_preview(&mut self, ctx: &egui::Context) {
        for event in self.engine.poll() {
            match event {
                PreviewEvent::Loaded { request_id, result } => {
                    if self.preview.request_id == Some(request_id) {
                        self.preview.accept(ctx, result);
                    }
                }
                PreviewEvent::Failed {
                    request_id,
                    path,
                    error,
                } => {
                    if self.preview.request_id == Some(request_id) {
                        let name = path
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "file".to_owned());
                        self.preview.fail(format!("{name}: {error}"));
                    }
                }
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if self.screen != Screen::Decision
            || self.confirm_leave
            || self.show_shortcuts
            || ctx.egui_wants_keyboard_input()
        {
            return;
        }

        if ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::Z)) {
            self.pending = Some(PendingAction::Undo);
            return;
        }

        if ctx.input(|input| input.key_pressed(egui::Key::Slash) && input.modifiers.shift) {
            self.show_shortcuts = true;
            return;
        }

        let plain = ctx.input(|input| {
            !input.modifiers.ctrl
                && !input.modifiers.alt
                && !input.modifiers.command
                && !input.modifiers.shift
        });
        if !plain {
            return;
        }

        if ctx.input(|input| {
            input.key_pressed(egui::Key::D) || input.key_pressed(egui::Key::ArrowLeft)
        }) {
            self.pending = Some(PendingAction::Decide(Decision::Junk));
        } else if ctx.input(|input| {
            input.key_pressed(egui::Key::K) || input.key_pressed(egui::Key::ArrowRight)
        }) {
            self.pending = Some(PendingAction::Decide(Decision::Keep));
        } else if ctx
            .input(|input| input.key_pressed(egui::Key::M) || input.key_pressed(egui::Key::ArrowUp))
        {
            self.pending = Some(PendingAction::Decide(Decision::Manual));
        }
    }

    fn process_pending(&mut self, ctx: &egui::Context) -> bool {
        let Some(pending) = self.pending.take() else {
            return false;
        };
        match pending {
            PendingAction::Decide(decision) => self.apply_decision(decision, ctx),
            PendingAction::Undo => self.undo(ctx),
        }
        true
    }

    fn draw_confirm_leave(&mut self, ctx: &egui::Context) {
        if !self.confirm_leave {
            return;
        }
        let mut close = false;
        let mut leave = false;
        egui::Window::new("leave-session")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .frame(
                Frame::NONE
                    .fill(theme::PANEL)
                    .stroke(Stroke::new(1.0, theme::STROKE))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(24)),
            )
            .show(ctx, |ui| {
                ui.set_width(380.0);
                ui.label(
                    RichText::new("Leave this session?")
                        .font(theme::semibold(21.0))
                        .color(theme::TEXT),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(
                        "Files you've already moved will stay in their holding folders. Files \
                         you haven't decided on will be left untouched.",
                    )
                    .font(egui::FontId::proportional(13.5))
                    .color(theme::MUTED),
                );
                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    if widgets::secondary_button(ui, "Stay here", None, theme::TEXT_SOFT, "")
                        .clicked()
                    {
                        close = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::primary_button(ui, "Leave", None, 110.0).clicked() {
                            leave = true;
                        }
                    });
                });
            });

        if close || ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.confirm_leave = false;
            self.leave_then_pick = false;
        }
        if leave {
            self.confirm_leave = false;
            let then_pick = self.leave_then_pick;
            self.leave_then_pick = false;
            self.leave_session(ctx, then_pick);
        }
    }

    fn draw_shortcuts(&mut self, ctx: &egui::Context) {
        if !self.show_shortcuts {
            return;
        }
        let mut close = false;
        egui::Window::new("shortcuts")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .frame(
                Frame::NONE
                    .fill(theme::PANEL)
                    .stroke(Stroke::new(1.0, theme::STROKE))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(24)),
            )
            .show(ctx, |ui| {
                ui.set_width(380.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Keyboard shortcuts")
                            .font(theme::semibold(21.0))
                            .color(theme::TEXT),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::ghost_button(ui, "Close", theme::MUTED).clicked() {
                            close = true;
                        }
                    });
                });
                ui.add_space(14.0);
                let rows = [
                    ("Delete", "D  or  ←"),
                    ("Set aside", "M  or  ↑"),
                    ("Keep", "K"),
                    ("Undo last decision", "Ctrl + Z"),
                    ("Show shortcuts", "Shift + /"),
                    ("Close dialog", "Esc"),
                ];
                for (label, keys) in rows {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(label)
                                .font(egui::FontId::proportional(13.5))
                                .color(theme::TEXT_SOFT),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            widgets::plain_chip(ui, keys, theme::MUTED);
                        });
                    });
                    ui.add_space(2.0);
                }
                ui.add_space(10.0);
            });

        if close || ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.show_shortcuts = false;
        }
    }

    fn draw_toast(&mut self, ctx: &egui::Context) {
        let Some(toast) = &self.toast else {
            return;
        };
        let age = toast.created.elapsed();
        if age > toast.ttl {
            self.toast = None;
            return;
        }
        let color = toast.kind.color();
        let offset = if self.screen == Screen::Decision {
            Vec2::new(0.0, -156.0)
        } else {
            Vec2::new(0.0, -22.0)
        };
        let icon = match toast.kind {
            ToastKind::Success => icons::check as widgets::IconDraw,
            ToastKind::Warning => icons::warning as widgets::IconDraw,
            ToastKind::Error => icons::warning as widgets::IconDraw,
            ToastKind::Info => icons::clock as widgets::IconDraw,
        };
        egui::Area::new(egui::Id::unique("swipeaway-toast"))
            .anchor(Align2::CENTER_BOTTOM, offset)
            .order(Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                widgets::toast_frame(color).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::hover());
                        icon(ui.painter(), rect, color, 1.6);
                        ui.label(
                            RichText::new(&toast.message)
                                .font(egui::FontId::proportional(13.0))
                                .color(theme::TEXT),
                        );
                    });
                });
            });
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

impl App for SwipeAwayApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_preview(&ctx);

        if self.auto_pick && self.screen == Screen::Welcome {
            if self.startup_frames >= 2 {
                self.auto_pick = false;
                self.open_picker(&ctx);
            } else {
                self.startup_frames = self.startup_frames.saturating_add(1);
                ctx.request_repaint();
            }
        }

        self.handle_shortcuts(&ctx);
        self.process_pending(&ctx);

        widgets::paint_background(&ctx);

        Frame::NONE
            .fill(Color32::TRANSPARENT)
            .inner_margin(Margin::symmetric(28, 22))
            .show(ui, |ui| match self.screen {
                Screen::Welcome => self.screen_welcome(ui, &ctx),
                Screen::Decision => self.screen_decision(ui, &ctx),
                Screen::Summary => self.screen_summary(ui, &ctx),
            });

        self.draw_confirm_leave(&ctx);
        self.draw_shortcuts(&ctx);
        self.draw_toast(&ctx);

        if self.process_pending(&ctx) {
            ctx.request_repaint();
        }
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.config.save();
    }
}
