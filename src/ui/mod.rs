pub mod decision;
pub mod folder_explorer;
pub mod icons;
pub mod preview;
pub mod summary;
pub mod welcome;
pub mod widgets;

use egui::Color32;

use crate::preview::MediaKind;
use crate::theme;

pub fn kind_color(kind: MediaKind) -> Color32 {
    match kind {
        MediaKind::Image => theme::BLUE,
        MediaKind::Svg => theme::PURPLE,
        MediaKind::Text => theme::KEEP,
        MediaKind::Video => theme::DELETE,
        MediaKind::Audio => theme::MANUAL,
        MediaKind::Pdf => theme::DELETE,
        MediaKind::Office => theme::BLUE,
        MediaKind::Archive => theme::MANUAL,
        MediaKind::Font => theme::PURPLE,
        MediaKind::Binary => theme::MUTED,
        MediaKind::Unknown => theme::MUTED,
    }
}
