use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Text,
    Video,
    Audio,
    Pdf,
    Office,
    Archive,
    Font,
    Svg,
    Binary,
    Unknown,
}

impl MediaKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Image => "Image",
            Self::Text => "Text",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Pdf => "PDF",
            Self::Office => "Document",
            Self::Archive => "Archive",
            Self::Font => "Font",
            Self::Svg => "Vector image",
            Self::Binary => "Binary",
            Self::Unknown => "File",
        }
    }
}

pub fn classify(path: &Path) -> MediaKind {
    let ext = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "jfif" | "gif" | "webp" | "bmp" | "ico" | "tif" | "tiff"
        | "tga" | "dds" | "pnm" | "ppm" | "pgm" | "pbm" | "hdr" | "qoi" => MediaKind::Image,
        "svg" | "svgz" => MediaKind::Svg,
        "mp4" | "m4v" | "mkv" | "webm" | "avi" | "mov" | "wmv" | "flv" | "mpg" | "mpeg"
        | "m2ts" | "mts" | "ts" | "3gp" | "ogv" | "rmvb" | "vob" => MediaKind::Video,
        "mp3" | "wav" | "flac" | "ogg" | "oga" | "opus" | "m4a" | "aac" | "wma" | "aiff"
        | "aif" | "alac" | "mid" | "midi" | "ape" => MediaKind::Audio,
        "pdf" => MediaKind::Pdf,
        "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" | "rtf"
        | "epub" | "pages" | "numbers" | "key" => MediaKind::Office,
        "zip" | "jar" | "apk" | "ipa" | "xpi" | "whl" | "nupkg" | "vsix" => MediaKind::Archive,
        "7z" | "rar" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "zst" | "cab" | "iso" => {
            MediaKind::Archive
        }
        "ttf" | "otf" | "ttc" | "otc" | "woff" | "woff2" => MediaKind::Font,
        "exe" | "dll" | "sys" | "msi" | "com" | "scr" | "cpl" | "ocx" | "drv" | "bin" => {
            MediaKind::Binary
        }
        _ if is_text_extension(&ext) || is_known_text_file(path) => MediaKind::Text,
        _ => MediaKind::Unknown,
    }
}

pub fn is_text_extension(ext: &str) -> bool {
    matches!(
        ext,
        "txt"
            | "md"
            | "markdown"
            | "mdx"
            | "log"
            | "json"
            | "jsonc"
            | "json5"
            | "yaml"
            | "yml"
            | "toml"
            | "ini"
            | "cfg"
            | "conf"
            | "config"
            | "properties"
            | "env"
            | "csv"
            | "tsv"
            | "xml"
            | "html"
            | "htm"
            | "xhtml"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "js"
            | "mjs"
            | "cjs"
            | "jsx"
            | "ts"
            | "tsx"
            | "vue"
            | "svelte"
            | "rs"
            | "py"
            | "pyw"
            | "pyi"
            | "c"
            | "h"
            | "cc"
            | "cpp"
            | "cxx"
            | "hpp"
            | "hh"
            | "cs"
            | "csx"
            | "java"
            | "kt"
            | "kts"
            | "go"
            | "rb"
            | "php"
            | "swift"
            | "m"
            | "mm"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "ps1"
            | "psm1"
            | "bat"
            | "cmd"
            | "sql"
            | "lua"
            | "r"
            | "dart"
            | "gradle"
            | "groovy"
            | "scala"
            | "clj"
            | "cljs"
            | "ex"
            | "exs"
            | "erl"
            | "hrl"
            | "hs"
            | "lhs"
            | "ml"
            | "mli"
            | "fs"
            | "fsx"
            | "zig"
            | "nim"
            | "v"
            | "d"
            | "jl"
            | "pl"
            | "pm"
            | "awk"
            | "vim"
            | "asm"
            | "s"
            | "tex"
            | "bib"
            | "proto"
            | "graphql"
            | "gql"
            | "diff"
            | "patch"
            | "reg"
            | "desktop"
            | "url"
            | "nfo"
            | "srt"
            | "vtt"
            | "ass"
            | "ssa"
            | "lrc"
            | "gitignore"
            | "gitattributes"
            | "editorconfig"
            | "dockerignore"
            | "npmignore"
            | "lock"
            | "cmake"
            | "make"
            | "mk"
            | "am"
            | "ac"
            | "m4"
            | "bazel"
            | "bzl"
            | "tf"
            | "tfvars"
            | "hcl"
            | "nginx"
            | "service"
            | "timer"
            | "plist"
            | "entitlements"
    )
}

fn is_known_text_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        "readme"
            | "license"
            | "licence"
            | "changelog"
            | "changes"
            | "authors"
            | "contributors"
            | "copying"
            | "notice"
            | "makefile"
            | "dockerfile"
            | "containerfile"
            | "procfile"
            | "vagrantfile"
            | "gemfile"
            | "rakefile"
            | "cmakelists.txt"
            | "cargo.lock"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | "go.sum"
            | "go.mod"
            | "pipfile"
            | "pipfile.lock"
            | "poetry.lock"
            | "requirements.txt"
            | ".gitignore"
            | ".gitattributes"
            | ".editorconfig"
            | ".env"
    )
}

pub fn language_name(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "rs" => "Rust",
        "py" | "pyw" | "pyi" => "Python",
        "js" | "mjs" | "cjs" => "JavaScript",
        "jsx" => "React",
        "ts" => "TypeScript",
        "tsx" => "React + TS",
        "json" | "jsonc" | "json5" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "html" | "htm" | "xhtml" => "HTML",
        "css" => "CSS",
        "scss" | "sass" => "Sass",
        "md" | "markdown" | "mdx" => "Markdown",
        "c" | "h" => "C",
        "cc" | "cpp" | "cxx" | "hpp" | "hh" => "C++",
        "cs" | "csx" => "C#",
        "java" => "Java",
        "kt" | "kts" => "Kotlin",
        "go" => "Go",
        "rb" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "sh" | "bash" | "zsh" | "fish" => "Shell",
        "ps1" | "psm1" => "PowerShell",
        "bat" | "cmd" => "Batch",
        "sql" => "SQL",
        "lua" => "Lua",
        "r" => "R",
        "dart" => "Dart",
        "xml" => "XML",
        "csv" | "tsv" => "Delimited text",
        "log" | "txt" | "nfo" => "Plain text",
        "ini" | "cfg" | "conf" | "properties" | "env" => "Config",
        "tex" => "LaTeX",
        "diff" | "patch" => "Diff",
        _ => "Text",
    }
}
