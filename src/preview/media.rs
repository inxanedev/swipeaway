use std::path::Path;
use std::process::Stdio;
use std::sync::OnceLock;

use fontdue::{Font, FontSettings};
use image::{DynamicImage, ImageDecoder, ImageReader};
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::Accessor;

use crate::util::command_hidden;

use super::{
    ArchiveEntry, ArchivePreview, AudioMeta, AudioPreview, Bitmap, FontPreview, ImagePreview,
    VideoMeta, VideoPreview,
};

pub fn load_image(path: &Path) -> Result<ImagePreview, String> {
    let reader = ImageReader::open(path)
        .map_err(|err| format!("Could not open image: {err}"))?
        .with_guessed_format()
        .map_err(|err| format!("Could not detect image format: {err}"))?;

    let format = reader
        .format()
        .map(|format| format!("{format:?}").to_uppercase())
        .unwrap_or_else(|| {
            path.extension()
                .map(|ext| ext.to_string_lossy().to_uppercase())
                .unwrap_or_else(|| "IMAGE".to_owned())
        });

    let mut decoder = reader
        .into_decoder()
        .map_err(|err| format!("Could not decode image: {err}"))?;

    let mut orientation_applied = false;
    let orientation = decoder.orientation().ok();
    if orientation.is_some() {
        orientation_applied = true;
    }

    let mut image = DynamicImage::from_decoder(decoder)
        .map_err(|err| format!("Could not decode image: {err}"))?;

    if let Some(orientation) = orientation {
        image.apply_orientation(orientation);
    }

    let (original_width, original_height) = (image.width(), image.height());
    let bitmap = dynamic_to_bitmap(image, 2400);
    let ext = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    Ok(ImagePreview {
        bitmap,
        original_width,
        original_height,
        format,
        animated: ext == "gif" || ext == "webp",
        orientation_applied,
    })
}

pub fn load_video(path: &Path) -> Result<VideoPreview, String> {
    let meta = probe_video(path);
    let mut poster = None;
    let mut filmstrip = Vec::new();
    let mut source = "Windows".to_owned();

    if ffmpeg_available() {
        let duration = meta.duration_secs;
        let poster_seek = duration
            .filter(|duration| *duration > 0.5)
            .map(|duration| (duration * 0.12).clamp(0.1, 8.0))
            .unwrap_or(0.5);

        poster = extract_video_frame(path, poster_seek, 1600);
        if poster.is_none() && poster_seek > 0.0 {
            poster = extract_video_frame(path, 0.0, 1600);
        }

        let fractions = [0.32, 0.56, 0.82];
        for (index, fraction) in fractions.iter().enumerate() {
            let seek = duration
                .filter(|duration| *duration > 1.0)
                .map(|duration| (duration * fraction).clamp(0.2, 3600.0))
                .unwrap_or(1.5 + index as f64 * 2.0);
            if let Some(frame) = extract_video_frame(path, seek, 420) {
                filmstrip.push(frame);
            }
        }

        if poster.is_some() || !filmstrip.is_empty() {
            source = "FFmpeg".to_owned();
        }
    }

    Ok(VideoPreview {
        poster,
        filmstrip,
        meta,
        source,
    })
}

pub fn load_audio(path: &Path) -> AudioPreview {
    let mut meta = AudioMeta {
        format: path
            .extension()
            .map(|ext| ext.to_string_lossy().to_uppercase())
            .unwrap_or_else(|| "AUDIO".to_owned()),
        ..AudioMeta::default()
    };
    let mut cover = None;

    if let Ok(tagged) = lofty::read_from_path(path) {
        let properties = tagged.properties();
        let duration = properties.duration();
        if !duration.is_zero() {
            meta.duration_secs = Some(duration.as_secs_f64());
        }
        meta.bitrate = properties
            .audio_bitrate()
            .or_else(|| properties.overall_bitrate());
        meta.sample_rate = properties.sample_rate();
        meta.channels = properties.channels();

        if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
            meta.title = tag.title().map(|value| value.into_owned());
            meta.artist = tag.artist().map(|value| value.into_owned());
            meta.album = tag.album().map(|value| value.into_owned());

            for picture in tag.pictures() {
                if let Ok(image) = image::load_from_memory(picture.data()) {
                    cover = Some(dynamic_to_bitmap(image, 900));
                    break;
                }
            }
        }
    }

    let waveform = if ffmpeg_available() {
        make_waveform(path)
    } else {
        None
    };

    AudioPreview {
        cover,
        waveform,
        meta,
    }
}

pub fn load_archive(path: &Path) -> Result<ArchivePreview, String> {
    let file = std::fs::File::open(path).map_err(|err| format!("Could not open archive: {err}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|err| format!("Not a readable ZIP archive: {err}"))?;

    let total_entries = archive.len();
    const MAX_ENTRIES: usize = 500;
    let shown = total_entries.min(MAX_ENTRIES);
    let mut entries = Vec::with_capacity(shown);
    let mut shown_size = 0u64;

    for index in 0..shown {
        let Ok(entry) = archive.by_index(index) else {
            continue;
        };
        let Ok(name) = entry.name() else {
            continue;
        };
        let entry = ArchiveEntry {
            name: name.into_owned(),
            size: entry.size(),
            is_dir: entry.is_dir(),
        };
        if !entry.is_dir {
            shown_size = shown_size.saturating_add(entry.size);
        }
        entries.push(entry);
    }

    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));

    Ok(ArchivePreview {
        entries,
        total_entries,
        shown_size,
        truncated: total_entries > shown,
    })
}

pub fn load_font(path: &Path) -> Result<FontPreview, String> {
    let data = std::fs::read(path).map_err(|err| format!("Could not read font file: {err}"))?;
    let font = Font::from_bytes(
        data,
        FontSettings {
            collection_index: 0,
            ..FontSettings::default()
        },
    )
    .map_err(|err| format!("Unsupported font file: {err}"))?;

    let width = 1400usize;
    let height = 520usize;
    let mut bitmap = Bitmap::empty(width as u32, height as u32);

    let lines = [
        ("SwipeAway", 104.0f32, [94u8, 234, 212, 255]),
        (
            "Aa Bb Cc Dd Ee Ff Gg Hh Ii Jj",
            54.0,
            [230u8, 237, 247, 255],
        ),
        ("0123456789  !@#$%&*()", 54.0, [150u8, 165, 185, 255]),
    ];

    let mut y = 40.0f32;
    for (text, size, color) in lines {
        let metrics = font
            .horizontal_line_metrics(size)
            .map(|metrics| metrics.ascent)
            .unwrap_or(size);
        let baseline = y + metrics;
        let mut x = 46.0f32;
        for character in text.chars() {
            let (glyph, coverage) = font.rasterize(character, size);
            blit_coverage(
                &mut bitmap,
                &coverage,
                glyph.width,
                glyph.height,
                x + glyph.xmin as f32,
                baseline - (glyph.height as f32 + glyph.ymin as f32),
                color,
            );
            x += glyph.advance_width;
        }
        let line_metrics = font
            .horizontal_line_metrics(size)
            .map(|metrics| metrics.new_line_size)
            .unwrap_or(size * 1.3);
        y += line_metrics;
    }

    Ok(FontPreview {
        specimen: bitmap,
        format: path
            .extension()
            .map(|ext| ext.to_string_lossy().to_uppercase())
            .unwrap_or_else(|| "FONT".to_owned()),
    })
}

pub fn ffmpeg_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| binary_available("ffmpeg"))
}

fn ffprobe_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| binary_available("ffprobe"))
}

fn binary_available(name: &str) -> bool {
    command_hidden(name)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn probe_video(path: &Path) -> VideoMeta {
    let mut meta = VideoMeta::default();
    if !ffprobe_available() {
        return meta;
    }

    let output = command_hidden("ffprobe")
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .stderr(Stdio::null())
        .output();

    let Ok(output) = output else {
        return meta;
    };
    if !output.status.success() {
        return meta;
    }
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) else {
        return meta;
    };

    if let Some(duration) = json
        .get("format")
        .and_then(|format| format.get("duration"))
        .and_then(|value| value.as_str())
        .and_then(|value| value.parse::<f64>().ok())
        && duration.is_finite()
        && duration > 0.0
    {
        meta.duration_secs = Some(duration);
    }
    if let Some(bit_rate) = json
        .get("format")
        .and_then(|format| format.get("bit_rate"))
        .and_then(|value| value.as_str())
        .and_then(|value| value.parse::<u64>().ok())
    {
        meta.bit_rate = Some(bit_rate);
    }

    if let Some(streams) = json.get("streams").and_then(|streams| streams.as_array()) {
        for stream in streams {
            match stream.get("codec_type").and_then(|value| value.as_str()) {
                Some("video") if meta.width.is_none() => {
                    meta.width = stream
                        .get("width")
                        .and_then(|value| value.as_u64())
                        .map(|value| value as u32);
                    meta.height = stream
                        .get("height")
                        .and_then(|value| value.as_u64())
                        .map(|value| value as u32);
                    meta.codec = stream
                        .get("codec_name")
                        .and_then(|value| value.as_str())
                        .map(str::to_owned);
                    if let Some(rate) = stream
                        .get("avg_frame_rate")
                        .or_else(|| stream.get("r_frame_rate"))
                        .and_then(|value| value.as_str())
                    {
                        meta.frame_rate = parse_rate(rate);
                    }
                }
                Some("audio") => meta.has_audio = true,
                _ => {}
            }
        }
    }

    meta
}

fn parse_rate(rate: &str) -> Option<f64> {
    let (numerator, denominator) = rate.split_once('/')?;
    let numerator = numerator.parse::<f64>().ok()?;
    let denominator = denominator.parse::<f64>().ok()?;
    if denominator <= 0.0 {
        return None;
    }
    let value = numerator / denominator;
    if value.is_finite() && value > 0.0 {
        Some(value)
    } else {
        None
    }
}

fn extract_video_frame(path: &Path, seek: f64, max_size: u32) -> Option<Bitmap> {
    let output = command_hidden("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-ss",
            &format!("{seek:.3}"),
            "-i",
        ])
        .arg(path)
        .args([
            "-an",
            "-sn",
            "-frames:v",
            "1",
            "-vf",
            &format!("scale={max_size}:{max_size}:force_original_aspect_ratio=decrease"),
            "-f",
            "image2pipe",
            "-vcodec",
            "png",
            "pipe:1",
        ])
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }
    decode_memory_bitmap(&output.stdout, max_size)
}

fn make_waveform(path: &Path) -> Option<Bitmap> {
    let output = command_hidden("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-i"])
        .arg(path)
        .args([
            "-t",
            "900",
            "-ac",
            "1",
            "-ar",
            "4000",
            "-f",
            "s16le",
            "-acodec",
            "pcm_s16le",
            "pipe:1",
        ])
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() || output.stdout.len() < 2 {
        return None;
    }

    let samples: Vec<f32> = output
        .stdout
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32)
        .collect();
    if samples.is_empty() {
        return None;
    }

    const WIDTH: u32 = 1200;
    const HEIGHT: u32 = 240;
    const BARS: usize = 160;
    let mut bitmap = Bitmap::empty(WIDTH, HEIGHT);
    let center_y = HEIGHT as f32 * 0.5;
    let half_height = HEIGHT as f32 * 0.5 - 18.0;
    let left = 20.0f32;
    let usable = WIDTH as f32 - 40.0;
    let slot = usable / BARS as f32;
    let bar_width = (slot * 0.62).max(1.0);

    for index in 0..BARS {
        let start = index * samples.len() / BARS;
        let end = ((index + 1) * samples.len() / BARS).min(samples.len());
        if start >= end {
            continue;
        }
        let peak = samples[start..end]
            .iter()
            .fold(0.0f32, |peak, sample| peak.max(sample.abs()));
        let amplitude = (peak / 32768.0).sqrt().clamp(0.0, 1.0);
        let height = (amplitude * half_height).max(1.0);
        let x = left + index as f32 * slot;
        let top = center_y - height;
        let bottom = center_y + height;
        draw_rect_rgba(
            &mut bitmap,
            x,
            top,
            bar_width.max(1.0),
            bottom - top,
            [94, 234, 212, 210],
        );
        draw_rect_rgba(
            &mut bitmap,
            x + bar_width * 0.5 - 1.0,
            top,
            2.0,
            3.0,
            [190, 255, 244, 230],
        );
    }

    draw_rect_rgba(
        &mut bitmap,
        16.0,
        center_y - 0.5,
        WIDTH as f32 - 32.0,
        1.0,
        [94, 234, 212, 38],
    );

    Some(bitmap)
}

fn draw_rect_rgba(bitmap: &mut Bitmap, x: f32, y: f32, width: f32, height: f32, color: [u8; 4]) {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = (x + width).ceil() as i32;
    let y1 = (y + height).ceil() as i32;
    for py in y0..y1 {
        for px in x0..x1 {
            if px < 0 || py < 0 || px >= bitmap.width as i32 || py >= bitmap.height as i32 {
                continue;
            }
            let index = ((py as u32 * bitmap.width + px as u32) * 4) as usize;
            let dst = &mut bitmap.rgba[index..index + 4];
            let alpha = color[3] as u32;
            let inv = 255 - alpha;
            dst[0] = ((color[0] as u32 * alpha + dst[0] as u32 * inv) / 255) as u8;
            dst[1] = ((color[1] as u32 * alpha + dst[1] as u32 * inv) / 255) as u8;
            dst[2] = ((color[2] as u32 * alpha + dst[2] as u32 * inv) / 255) as u8;
            dst[3] = (alpha + (dst[3] as u32 * inv) / 255).min(255) as u8;
        }
    }
}

fn decode_memory_bitmap(bytes: &[u8], max_size: u32) -> Option<Bitmap> {
    let image = image::load_from_memory(bytes).ok()?;
    if image.width() == 0 || image.height() == 0 {
        return None;
    }
    Some(dynamic_to_bitmap(image, max_size))
}

pub fn dynamic_to_bitmap(image: DynamicImage, max_size: u32) -> Bitmap {
    let image = if image.width() > max_size || image.height() > max_size {
        image.thumbnail(max_size, max_size)
    } else {
        image
    };
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    Bitmap::new(width, height, rgba.into_raw())
}

fn blit_coverage(
    target: &mut Bitmap,
    coverage: &[u8],
    glyph_width: usize,
    glyph_height: usize,
    x: f32,
    y: f32,
    color: [u8; 4],
) {
    let origin_x = x.round() as i32;
    let origin_y = y.round() as i32;
    for row in 0..glyph_height {
        for column in 0..glyph_width {
            let source = row * glyph_width + column;
            let alpha = coverage.get(source).copied().unwrap_or(0) as u32;
            if alpha == 0 {
                continue;
            }
            let target_x = origin_x + column as i32;
            let target_y = origin_y + row as i32;
            if target_x < 0
                || target_y < 0
                || target_x >= target.width as i32
                || target_y >= target.height as i32
            {
                continue;
            }
            let index = ((target_y as u32 * target.width + target_x as u32) * 4) as usize;
            let dst = &mut target.rgba[index..index + 4];
            let src_alpha = (color[3] as u32 * alpha) / 255;
            let inv = 255 - src_alpha;
            dst[0] = ((color[0] as u32 * src_alpha + dst[0] as u32 * inv) / 255) as u8;
            dst[1] = ((color[1] as u32 * src_alpha + dst[1] as u32 * inv) / 255) as u8;
            dst[2] = ((color[2] as u32 * src_alpha + dst[2] as u32 * inv) / 255) as u8;
            dst[3] = (src_alpha + (dst[3] as u32 * inv) / 255).min(255) as u8;
        }
    }
}
