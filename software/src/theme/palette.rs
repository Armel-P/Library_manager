use eframe::egui::Color32;

#[derive(Clone, Copy, PartialEq)]
pub struct Palette {
    pub bg_top: Color32,
    pub bg_bottom: Color32,
    pub card_fill: Color32,
    pub text: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub error: Color32
}
