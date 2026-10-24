pub mod detect;
pub mod engine;
pub mod media;
pub mod shell;
pub mod text;

use std::path::PathBuf;

pub use detect::MediaKind;

#[derive(Clone, Debug)]
pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Bitmap {
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Self {
        Self {
            width,
            height,
            rgba,
        }
    }

    pub fn empty(width: u32, height: u32) -> Self {
        Self::new(width, height, vec![0; (width * height * 4) as usize])
    }
}

#[derive(Debug)]
pub struct PreviewResult {
    pub path: PathBuf,
    pub content: PreviewContent,
    pub warnings: Vec<String>,
    pub elapsed_ms: u128,
}

#[derive(Debug)]
pub enum PreviewContent {
    Image(ImagePreview),
    Text(text::TextPreview),
    Video(VideoPreview),
    Audio(AudioPreview),
    Archive(ArchivePreview),
    Font(FontPreview),
    Shell(ShellPreview),
    Unsupported(UnsupportedPreview),
}

#[derive(Debug)]
pub struct ImagePreview {
    pub bitmap: Bitmap,
    pub original_width: u32,
    pub original_height: u32,
    pub format: String,
    pub animated: bool,
    pub orientation_applied: bool,
}

#[derive(Debug)]
pub struct VideoPreview {
    pub poster: Option<Bitmap>,
    pub filmstrip: Vec<Bitmap>,
    pub meta: VideoMeta,
    pub source: String,
}

#[derive(Debug, Default)]
pub struct VideoMeta {
    pub duration_secs: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub codec: Option<String>,
    pub bit_rate: Option<u64>,
    pub frame_rate: Option<f64>,
    pub has_audio: bool,
}

#[derive(Debug)]
pub struct AudioPreview {
    pub cover: Option<Bitmap>,
    pub waveform: Option<Bitmap>,
    pub meta: AudioMeta,
}

#[derive(Debug, Default)]
pub struct AudioMeta {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_secs: Option<f64>,
    pub format: String,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
}

#[derive(Debug)]
pub struct ArchivePreview {
    pub entries: Vec<ArchiveEntry>,
    pub total_entries: usize,
    pub shown_size: u64,
    pub truncated: bool,
}

#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Debug)]
pub struct FontPreview {
    pub specimen: Bitmap,
    pub format: String,
}

#[derive(Debug)]
pub struct ShellPreview {
    pub image: Option<Bitmap>,
    pub is_icon: bool,
    pub note: Option<String>,
}

#[derive(Debug)]
pub struct UnsupportedPreview {
    pub reason: String,
}
