pub mod palette;
mod dark_purple;
mod light_beige;

use palette::Palette;
use eframe::egui::{self, Visuals, Stroke};


#[derive(Clone, Copy, PartialEq, Default)]
pub enum Theme {
    #[default]
    DarkPurple,
    LightBeige,
}

impl Theme {
    pub fn palette(&self) -> Palette {
        match self {
            Theme::DarkPurple => dark_purple::PALETTE,
            Theme::LightBeige => light_beige::PALETTE,
        }
    }

    pub fn toggle(&mut self) {
        *self = match self {
            Theme::DarkPurple => Theme::LightBeige,
            Theme::LightBeige => Theme::DarkPurple,
        };
    }

    pub fn toggle_label(&self) -> &'static str {
        match self {
            Theme::DarkPurple => "Light",
            Theme::LightBeige => "Dark",
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let p = self.palette();
        let mut visuals = match self {
            Theme::DarkPurple => Visuals::dark(),
            Theme::LightBeige => Visuals::light(),
        };
        
        visuals.panel_fill = p.bg_top;
        visuals.window_fill = p.bg_top;

        visuals.widgets.inactive.bg_fill = p.card_fill;
        visuals.widgets.hovered.bg_fill = p.accent_hover;
        visuals.widgets.active.bg_fill = p.accent;

        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, p.accent_hover);
        visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, p.accent);
        visuals.widgets.active.bg_stroke = Stroke::new(2.5, p.accent);

        visuals.override_text_color = Some(p.text);

        visuals.selection.bg_fill = p.accent;
        visuals.hyperlink_color = p.accent;

        ctx.set_visuals(visuals);

        // ----------------------------------------------------------------------------
        let mut style = (*ctx.style()).clone();
    
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(20.0, egui::FontFamily::Proportional),
        );

        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(17.0, egui::FontFamily::Proportional),
        );

        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(30.0, egui::FontFamily::Proportional),
        );

        ctx.set_style(style);
    }
}