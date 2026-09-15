use eframe::egui;
use crate::main_app::models::{Condition};

pub fn int_field(en_int: &mut Option<i64>, text: &str, ui: &mut egui::Ui) {
    let mut isbn: i64 = en_int.unwrap_or(0);

    ui.label(text); ui.add(egui::DragValue::new(&mut isbn));

    *en_int = if isbn == 0 { None } else { Some(isbn) };
}

pub fn condition_field(value: &mut Option<i8>, ui: &mut egui::Ui) {
    ui.label("Condition:");

    let selected_text = value
        .and_then(|v| Condition::try_from(v).ok())
        .map(|c| c.label())
        .unwrap_or("None");

    egui::ComboBox::from_id_source("condition")
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            for condition in Condition::ALL {
                ui.selectable_value(value, Some(i8::from(condition)), condition.label());
            }
        });
}