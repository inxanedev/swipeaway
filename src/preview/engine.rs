use std::collections::{HashMap, VecDeque};
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;

use super::{
    MediaKind, PreviewContent, PreviewResult, ShellPreview, UnsupportedPreview, detect, media,
    shell, text,
};

const CACHE_CAPACITY: usize = 18;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PreviewKey {
    path: PathBuf,
    len: u64,
    modified_nanos: u128,
}

#[derive(Debug)]
struct Job {
    id: u64,
    path: PathBuf,
}

#[derive(Debug)]
pub enum PreviewEvent {
    Loaded {
        request_id: u64,
        result: Arc<PreviewResult>,
    },
    Failed {
        request_id: u64,
        path: PathBuf,
        error: String,
    },
}

pub struct PreviewEngine {
    tx: Sender<Job>,
    rx: Receiver<PreviewEvent>,
    next_id: u64,
    latest: Arc<AtomicU64>,
}

impl PreviewEngine {
    pub fn new(ctx: egui::Context) -> Self {
        let (job_tx, job_rx) = channel::<Job>();
        let (event_tx, event_rx) = channel::<PreviewEvent>();
        let latest = Arc::new(AtomicU64::new(0));
        let worker_latest = Arc::clone(&latest);

        std::thread::Builder::new()
            .name("swipeaway-preview".to_owned())
            .spawn(move || {
                let mut worker = Worker {
                    rx: job_rx,
                    tx: event_tx,
                    ctx,
                    latest: worker_latest,
                    cache: HashMap::new(),
                    order: VecDeque::new(),
                };
                worker.run();
            })
            .expect("failed to start preview loader");

        Self {
            tx: job_tx,
            rx: event_rx,
            next_id: 1,
            latest,
        }
    }

    pub fn request(&mut self, path: PathBuf) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.latest.store(id, Ordering::SeqCst);
        let _ = self.tx.send(Job { id, path });
        id
    }

    pub fn poll(&mut self) -> Vec<PreviewEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            events.push(event);
        }
        events
    }
}

struct Worker {
    rx: Receiver<Job>,
    tx: Sender<PreviewEvent>,
    ctx: egui::Context,
    latest: Arc<AtomicU64>,
    cache: HashMap<PreviewKey, Arc<PreviewResult>>,
    order: VecDeque<PreviewKey>,
}

impl Worker {
    fn run(&mut self) {
        while let Ok(job) = self.rx.recv() {
            if job.id != self.latest.load(Ordering::SeqCst) {
                continue;
            }

            let key = key_for(&job.path);
            if let Some(cached) = self.cache.get(&key).cloned() {
                self.touch(&key);
                if job.id == self.latest.load(Ordering::SeqCst) {
                    let _ = self.tx.send(PreviewEvent::Loaded {
                        request_id: job.id,
                        result: cached,
                    });
                    self.ctx.request_repaint();
                }
                continue;
            }

            let started = Instant::now();
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| analyze(&job.path)));
            match result {
                Ok(mut preview) => {
                    preview.elapsed_ms = started.elapsed().as_millis();
                    let preview = Arc::new(preview);
                    self.insert(key, Arc::clone(&preview));
                    if job.id == self.latest.load(Ordering::SeqCst) {
                        let _ = self.tx.send(PreviewEvent::Loaded {
                            request_id: job.id,
                            result: preview,
                        });
                        self.ctx.request_repaint();
                    }
                }
                Err(_) => {
                    if job.id == self.latest.load(Ordering::SeqCst) {
                        let _ = self.tx.send(PreviewEvent::Failed {
                            request_id: job.id,
                            path: job.path,
                            error: "The preview crashed while loading this file.".to_owned(),
                        });
                        self.ctx.request_repaint();
                    }
                }
            }
        }
    }

    fn touch(&mut self, key: &PreviewKey) {
        if let Some(position) = self.order.iter().position(|candidate| candidate == key) {
            self.order.remove(position);
            self.order.push_back(key.clone());
        }
    }

    fn insert(&mut self, key: PreviewKey, preview: Arc<PreviewResult>) {
        self.cache.insert(key.clone(), preview);
        self.order.push_back(key);
        while self.order.len() > CACHE_CAPACITY {
            if let Some(oldest) = self.order.pop_front() {
                self.cache.remove(&oldest);
            }
        }
    }
}

fn key_for(path: &Path) -> PreviewKey {
    let metadata = std::fs::metadata(path).ok();
    PreviewKey {
        path: path.to_path_buf(),
        len: metadata.as_ref().map(|m| m.len()).unwrap_or(0),
        modified_nanos: metadata
            .and_then(|m| m.modified().ok())
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos())
            .unwrap_or(0),
    }
}

fn analyze(path: &Path) -> PreviewResult {
    let mut kind = detect::classify(path);
    let mut warnings = Vec::new();

    if kind == MediaKind::Unknown && text::looks_like_text(path) {
        kind = MediaKind::Text;
    }

    if kind == MediaKind::Binary {
        warnings.push("This is an executable or binary file. Handle it with care.".to_owned());
    }

    let content = match kind {
        MediaKind::Image => match media::load_image(path) {
            Ok(image) => PreviewContent::Image(image),
            Err(error) => {
                warnings.push(error);
                shell_fallback(path, "No built-in preview could render this image.")
            }
        },
        MediaKind::Text => match text::load(path) {
            Ok(text) => PreviewContent::Text(text),
            Err(error) => {
                warnings.push(error);
                shell_fallback(path, "No text preview is available for this file.")
            }
        },
        MediaKind::Video => match media::load_video(path) {
            Ok(mut video) => {
                if video.poster.is_none()
                    && let Some(thumb) = shell::thumbnail(path, 1600)
                {
                    video.poster = Some(thumb.bitmap);
                    video.source = if thumb.is_icon {
                        "Windows icon".to_owned()
                    } else {
                        "Windows thumbnail".to_owned()
                    };
                }
                if video.poster.is_none() {
                    warnings.push(
                        "No frame preview was available. Use Play to open the video.".to_owned(),
                    );
                }
                if !media::ffmpeg_available() {
                    warnings.push(
                        "Install ffmpeg for extra frame previews. Windows thumbnails are used instead."
                            .to_owned(),
                    );
                }
                PreviewContent::Video(video)
            }
            Err(error) => {
                warnings.push(error);
                shell_fallback(path, "No video preview is available.")
            }
        },
        MediaKind::Audio => {
            let mut audio = media::load_audio(path);
            if audio.cover.is_none()
                && let Some(thumb) = shell::thumbnail(path, 900)
                && !thumb.is_icon
            {
                audio.cover = Some(thumb.bitmap);
            }
            PreviewContent::Audio(audio)
        }
        MediaKind::Pdf => shell_fallback(path, "No PDF preview is installed on this PC."),
        MediaKind::Office => {
            shell_fallback(path, "No document preview is available for this file.")
        }
        MediaKind::Archive => match media::load_archive(path) {
            Ok(archive) => PreviewContent::Archive(archive),
            Err(error) => {
                warnings.push(error);
                shell_fallback(path, "Archive contents can't be listed for this format.")
            }
        },
        MediaKind::Font => match media::load_font(path) {
            Ok(font) => PreviewContent::Font(font),
            Err(error) => {
                warnings.push(error);
                shell_fallback(path, "This font format can't be rendered natively.")
            }
        },
        MediaKind::Svg => shell_fallback(path, "This vector image has no system preview."),
        MediaKind::Binary | MediaKind::Unknown => {
            shell_fallback(path, "There is no built-in preview for this file type.")
        }
    };

    PreviewResult {
        path: path.to_path_buf(),
        content,
        warnings,
        elapsed_ms: 0,
    }
}

fn shell_fallback(path: &Path, note: &str) -> PreviewContent {
    match shell::thumbnail(path, 1024) {
        Some(thumb) => PreviewContent::Shell(ShellPreview {
            image: Some(thumb.bitmap),
            is_icon: thumb.is_icon,
            note: thumb
                .is_icon
                .then(|| format!("{note} Showing the system file icon instead.")),
        }),
        None => PreviewContent::Unsupported(UnsupportedPreview {
            reason: note.to_owned(),
        }),
    }
}
