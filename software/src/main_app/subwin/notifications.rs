use std::sync::mpsc;
use eframe::egui;
use reqwest::Method;
use crate::constants::HEADER_TOKEN;
use crate::requests::{self, structs::LateBooksResponse};
use crate::main_app::models::Book;
use crate::theme::Theme;

pub struct NotificationsPanel {
    pub open: bool,
    books: Vec<Book>,
    status: String,
    status_error: bool,
    loading: bool,
    rx: Option<mpsc::Receiver<Result<LateBooksResponse, String>>>
}

impl NotificationsPanel {
    pub fn new(token: String) -> Self {
        let mut panel = Self {
            open: false,
            books: vec![],
            status: String::new(),
            status_error: false,
            loading: false,
            rx: None,
        };
        panel.fetch(token);
        panel
    }

    pub fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.loading = false;
                match result {
                    Ok(resp) => {
                        self.books = resp.late_books.into_iter().map(Book::from).collect();
                        if self.books.is_empty() {
                            self.status = "No late books".into();
                        } else {
                            self.status = "Loaded".into();
                        }
                        self.status_error = false;
                    }
                    Err(e) => {
                        self.status = format!("Error: {e}");
                        self.status_error = true;
                    }
                }
                self.rx = None;
            }
        }
    }

    pub fn fetch(&mut self, token: String) {
        self.loading = true;
        self.status = "Loading notifications...".into();
        self.rx = Some(requests::spawn_request::<(), LateBooksResponse>(
            Method::POST,
            "book/late-borrowed".into(),
            None,
            Some(vec![(HEADER_TOKEN.to_string(), token.clone())]),
        ));
    }

    pub fn ui(&mut self, theme: Theme, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        let mut open = self.open;
        egui::Window::new("Late Borrowed Books")
            .open(&mut open)
            .show(ctx, |ui| {
                if self.status_error {
                    ui.colored_label(theme.palette().error, &self.status);
                } else if self.books.len() > 1 {
                    for book in &self.books {
                        ui.label(format!("{} by {} (Owner ID: {})", book.title, book.author, book.owner_id));
                    }
                } else {
                    ui.label(&self.status);
                }
            });
        self.open = open;
    }

    pub fn badge(&self) -> String {
        if self.books.is_empty() { "🔔".to_string() } else { format!("🔔({})", self.books.len()) }
    }
}
