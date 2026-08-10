use super::palette::Palette;
use eframe::egui::Color32;

pub const PALETTE: Palette = Palette {
    bg_top: Color32::from_rgb(15, 10, 25),
    bg_bottom: Color32::from_rgb(25, 20, 35),
    card_fill: Color32::from_rgb(30, 25, 35),
    text: Color32::from_rgb(195, 205, 215),
    accent: Color32::from_rgb(110, 50, 180),
    accent_hover: Color32::from_rgb(130, 70, 190),
    error: Color32::from_rgb(210, 75, 80)
};