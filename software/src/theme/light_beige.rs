use super::palette::Palette;
use eframe::egui::Color32;

pub const PALETTE: Palette = Palette {
    bg_top: Color32::from_rgb(240, 200, 150),
    bg_bottom: Color32::from_rgb(230, 220, 205),
    card_fill: Color32::from_rgb(235, 235, 235),
    text: Color32::from_rgb(60, 50, 40),
    accent: Color32::from_rgb(210, 170, 120),
    accent_hover: Color32::from_rgb(200, 160, 110),
    error: Color32::from_rgb(190, 75, 65),
};