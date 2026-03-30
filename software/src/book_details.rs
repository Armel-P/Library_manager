use eframe::egui;
use crate::app::Book;

pub struct BookDetailsApp {
    pub book: Book,
    pub token: String,
}

impl BookDetailsApp {
    pub fn new(book: Book, token: String) -> Self {
        Self { book, token }
    }

    pub fn update(&mut self, ctx: &egui::Context) -> bool {
        let mut go_back = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📖 Book Details");

            ui.label(format!("ISBN: {}", self.book.isbn));
            ui.label(format!("Title: {}", self.book.title));
            ui.label(format!("Author: {}", self.book.author));
            ui.label(format!("Owner ID: {}", self.book.owner_id));
            ui.label(format!("Available: {}", self.book.available));
            ui.label(format!("Condition: {}", self.book.condition));

            ui.label(format!(
                "Borrow date: {}",
                self.book.borrow_date.clone().unwrap_or("N/A".into())
            ));

            ui.separator();

            if ui.button("⬅ Back").clicked() {
                go_back = true;
            }
        });

        go_back
    }
}