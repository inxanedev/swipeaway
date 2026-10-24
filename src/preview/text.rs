use std::io::Read;
use std::path::Path;

use super::detect::language_name;

const READ_LIMIT: u64 = 2 * 1024 * 1024;
const SNIFF_LIMIT: usize = 8192;
const MAX_DISPLAY_LINES: usize = 5000;
const MAX_DISPLAY_CHARS: usize = 500_000;
const MAX_LINE_CHARS: usize = 4000;

#[derive(Debug)]
pub struct TextPreview {
    pub text: String,
    pub encoding: String,
    pub language: &'static str,
    pub lines_shown: usize,
    pub file_truncated: bool,
    pub display_truncated: bool,
}

pub fn load(path: &Path) -> Result<TextPreview, String> {
    let metadata = std::fs::metadata(path).map_err(|err| format!("Could not read file: {err}"))?;
    let total_bytes = metadata.len();

    let mut file =
        std::fs::File::open(path).map_err(|err| format!("Could not open file: {err}"))?;
    let mut bytes = Vec::with_capacity(total_bytes.min(READ_LIMIT) as usize);
    file.by_ref()
        .take(READ_LIMIT)
        .read_to_end(&mut bytes)
        .map_err(|err| format!("Could not read file: {err}"))?;

    if !bytes_look_textual(&bytes) {
        return Err("This file appears to be binary data.".to_owned());
    }

    let (text, encoding) = decode_text(&bytes)?;
    let file_truncated = total_bytes > READ_LIMIT;

    let mut shown = String::with_capacity(text.len().min(MAX_DISPLAY_CHARS));
    let mut lines_shown = 0usize;
    let mut chars_shown = 0usize;
    let mut display_truncated = false;

    for line in text.lines() {
        if lines_shown >= MAX_DISPLAY_LINES || chars_shown >= MAX_DISPLAY_CHARS {
            display_truncated = true;
            break;
        }

        let mut line = line.replace('\t', "    ");
        if line.chars().count() > MAX_LINE_CHARS {
            line = line.chars().take(MAX_LINE_CHARS).collect::<String>();
            line.push('…');
            display_truncated = true;
        }

        if !shown.is_empty() {
            shown.push('\n');
            chars_shown += 1;
        }
        chars_shown += line.chars().count();
        shown.push_str(&line);
        lines_shown += 1;
    }

    if text.is_empty() {
        lines_shown = 0;
    }

    Ok(TextPreview {
        text: shown,
        encoding,
        language: language_name(path),
        lines_shown,
        file_truncated,
        display_truncated,
    })
}

pub fn looks_like_text(path: &Path) -> bool {
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut bytes = vec![0u8; SNIFF_LIMIT];
    let Ok(read) = file.read(&mut bytes) else {
        return false;
    };
    bytes.truncate(read);
    !bytes.is_empty() && bytes_look_textual(&bytes)
}

fn bytes_look_textual(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true;
    }
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        return true;
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return bytes[3..].iter().all(|&b| b != 0);
    }
    if bytes.contains(&0) {
        return false;
    }

    let sample_len = bytes.len().min(SNIFF_LIMIT);
    let sample = &bytes[..sample_len];
    let control = sample
        .iter()
        .filter(|&&b| b < 0x20 && b != b'\n' && b != b'\r' && b != b'\t' && b != 0x0C)
        .count();

    if control * 100 > sample_len.max(1) * 3 {
        return false;
    }

    match std::str::from_utf8(sample) {
        Ok(_) => true,
        Err(err) => {
            let valid = err.valid_up_to();
            valid * 100 >= sample_len.max(1) * 90
        }
    }
}

fn decode_text(bytes: &[u8]) -> Result<(String, String), String> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let text = std::str::from_utf8(&bytes[3..])
            .map_err(|_| "The file contains invalid UTF-8 text.".to_owned())?;
        return Ok((text.replace("\r\n", "\n"), "UTF-8".to_owned()));
    }

    if bytes.starts_with(&[0xFF, 0xFE]) {
        return Ok((
            decode_utf16(&bytes[2..], true)?.replace("\r\n", "\n"),
            "UTF-16 LE".to_owned(),
        ));
    }

    if bytes.starts_with(&[0xFE, 0xFF]) {
        return Ok((
            decode_utf16(&bytes[2..], false)?.replace("\r\n", "\n"),
            "UTF-16 BE".to_owned(),
        ));
    }

    match std::str::from_utf8(bytes) {
        Ok(text) => Ok((text.replace("\r\n", "\n"), "UTF-8".to_owned())),
        Err(_) => {
            let lossy = String::from_utf8_lossy(bytes);
            let replacements = lossy.chars().filter(|&c| c == '\u{FFFD}').count();
            if replacements * 100 > lossy.chars().count().max(1) {
                return Err("This file uses an unsupported text encoding.".to_owned());
            }
            Ok((lossy.replace("\r\n", "\n"), "UTF-8 (lossy)".to_owned()))
        }
    }
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> Result<String, String> {
    if !bytes.len().is_multiple_of(2) {
        return Err("The file contains a malformed UTF-16 sequence.".to_owned());
    }
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            if little_endian {
                u16::from_le_bytes([pair[0], pair[1]])
            } else {
                u16::from_be_bytes([pair[0], pair[1]])
            }
        })
        .collect();
    String::from_utf16(&units).map_err(|_| "The file contains invalid UTF-16 text.".to_owned())
}
