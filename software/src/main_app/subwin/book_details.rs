use std::sync::mpsc;
use eframe::egui;
use reqwest::Method;
use crate::{
    constants::{HEADER_TOKEN},
    main_app::models::{Book, Condition},
    requests::{self, structs::{BorrowBookRequest, ReturnBookRequest, MessResponse}},
    theme::Theme,
};

pub struct BookDetailsPanel {
    pub open: bool,
    book: Option<Book>,
    borrower_mail: String,
    status: String,
    status_error: bool,
    loading: bool,
    rx: Option<mpsc::Receiver<Result<MessResponse, String>>>,
}

impl BookDetailsPanel {
    pub fn new() -> Self {
        BookDetailsPanel {
            open: false,
            book: None,
            borrower_mail: String::new(),
            status: String::new(),
            status_error: false,
            loading: false,
            rx: None,
        }
    }

    pub fn open_for(&mut self, book: Book) {
        self.book = Some(book);
        self.open = true;
        self.borrower_mail.clear();
        self.status.clear();
        self.status_error = false;
    }

    pub fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.loading = false;
                match result {
                    Ok(message) => {
                        if let Some(book) = &mut self.book {
                            if book.available {
                                book.available = false;
                                book.borrower_mail = Some(self.borrower_mail.clone());
                            } else {
                                book.available = true;
                                book.borrower_mail = None;
                                self.borrower_mail.clear();
                            }
                        }
                        self.status = message.message;
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

    fn borrow(&mut self, token: String) {
        let Some(book) = &self.book else { return };
        if self.borrower_mail.trim().is_empty() {
            return;
        }
        self.loading = true;
        self.status = "Loading...".into();
        self.rx = Some(requests::spawn_request::<BorrowBookRequest, MessResponse>(
            Method::POST,
            "book/borrow".into(),
            Some(BorrowBookRequest {
                isbn: book.isbn.clone(),
                borrower_mail: self.borrower_mail.clone(),
            }),
            Some(vec![(HEADER_TOKEN.to_string(), token)]),
        ));
    }

    fn return_book(&mut self, token: String) {
        let Some(book) = &self.book else { return };
        self.loading = true;
        self.status = "Loading...".into();
        self.rx = Some(requests::spawn_request::<ReturnBookRequest, MessResponse>(
            Method::POST,
            "book/return".into(),
            Some(ReturnBookRequest { isbn: book.isbn.clone() }),
            Some(vec![(HEADER_TOKEN.to_string(), token)]),
        ));
    }

    pub fn ui(&mut self, token: String, theme: Theme, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        let Some(book) = self.book.clone() else {
            self.open = false;
            return;
        };

        let mut open = self.open;
        egui::Window::new("Book Details")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(format!("ISBN: {}", book.isbn));
                ui.label(format!("Title: {}", book.title));
                ui.label(format!("Author: {}", book.author));
                ui.label(format!("Owner ID: {}", book.owner_id));
                let condition_label = Condition::try_from(book.condition)
                    .map(|c| c.label())
                    .unwrap_or("Unknown");
                ui.label(format!("Condition: {}", condition_label));

                if book.available {
                    ui.colored_label(egui::Color32::from_rgb(80, 180, 90), "Available");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(200, 90, 90), "Borrowed");
                    if let Some(mail) = &book.borrower_mail {
                        ui.label(format!("Borrower: {}", mail));
                    }
                    if let Some(date) = &book.borrow_date {
                        ui.label(format!("Since: {}", date));
                    }
                }

                ui.separator();

                if book.available {
                    ui.heading("Borrow Book");
                    ui.horizontal(|ui| {
                        ui.label("Borrower email:");
                        ui.text_edit_singleline(&mut self.borrower_mail);
                    });
                    if ui
                        .add_enabled(!self.borrower_mail.trim().is_empty() && !self.loading, egui::Button::new("Borrow"))
                        .clicked()
                    {
                        self.borrow(token.clone());
                    }
                } else {
                    ui.heading("Return Book");
                    if ui
                        .add_enabled(!self.loading, egui::Button::new("Return"))
                        .clicked()
                    {
                        self.return_book(token.clone());
                    }
                }

                ui.add_space(4.0);

                if self.status_error {
                    ui.colored_label(theme.palette().error, &self.status);
                } else {
                    ui.label(&self.status);
                }
            });
        self.open = open;
    }
}
