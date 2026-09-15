use std::sync::mpsc;
use eframe::egui;
use reqwest::Method;
use crate::constants::HEADER_TOKEN;
use crate::requests::{self, structs::{AddBookRequest, AddBookResponse}};
use crate::theme::Theme;
use crate::utils::{int_field, condition_field};

#[derive(Clone)]
pub struct Fields {
    pub isbn: Option<i64>,
    pub title: String,
    pub author: String,
    pub owner_id: Option<i64>,
    pub condition: Option<i8>,
}

pub struct AddBookPanel {
    pub open: bool,
    fields: Fields,
    status: String,
    status_error: bool,
    loading: bool,
    rx: Option<mpsc::Receiver<Result<AddBookResponse, String>>>
}

impl AddBookPanel {
    pub fn new() -> Self {
        AddBookPanel {
            open: false,
            fields: Fields {
                isbn: None,
                title: String::new(),
                author: String::new(),
                owner_id: None,
                condition: None,
            },
            status: String::new(),
            status_error: false,
            loading: false,
            rx: None,
        }
    }

    pub fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.loading = false;
                match result {
                    Ok(message) => {
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

    fn cast_fields(&mut self) -> Option<AddBookRequest> {
        let isbn = match self.fields.isbn {
            Some(isbn) => { isbn },
            None => { self.status_error = true; self.status = "Missing isbn".into(); return None; }
        };
        let owner_id = match self.fields.owner_id {
            Some(id) => { id },
            None => { self.status_error = true; self.status = "Missing owner id".into(); return None; }
        };
        let condition = match self.fields.condition {
            Some(condition) => { condition },
            None => { self.status_error = true; self.status = "Missing condition".into(); return None; }
        };
        let title = match self.fields.title.trim().is_empty() {
            false => { self.fields.title.clone() },
            true => { self.status_error = true; self.status = "Missing title".into(); return None; }
        };
        let author = match self.fields.author.trim().is_empty() {
            false => { self.fields.author.clone() },
            true => { self.status_error = true; self.status = "Missing author".into(); return None; }
        };

        Some(AddBookRequest {
            isbn: isbn,
            title: title,
            author: author,
            owner_id: owner_id,
            condition: condition,
        })
    }

    pub fn fetch(&mut self, token: String) {
        let body = match self.cast_fields() {
            Some(body) => { body },
            None => { return; }
        };
        self.loading = true;
        self.status = "Loading...".into();
        self.rx = Some(requests::spawn_request::<AddBookRequest, AddBookResponse>(
            Method::POST,
            "book/create".into(),
            Some(body),
            Some(vec![(HEADER_TOKEN.to_string(), token)]),
        ));
    }

    pub fn ui(&mut self, token: String, theme: Theme, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        let mut open = self.open;
        egui::Window::new("Add a book")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui|{
                    int_field(&mut self.fields.isbn, "ISBN", ui);
                });
                ui.horizontal(|ui|{
                    ui.label("Title:"); ui.text_edit_singleline(&mut self.fields.title);
                });
                ui.horizontal(|ui|{
                    ui.label("Author:"); ui.text_edit_singleline(&mut self.fields.author);
                });
                ui.horizontal(|ui|{
                    int_field(&mut self.fields.owner_id, "Owner ID", ui);
                });
                ui.horizontal(|ui|{
                    condition_field(&mut self.fields.condition, ui);
                });

                ui.add_space(2.0);

                if ui.button("Add").clicked() {
                    self.fetch(token);
                }
                if self.status_error {
                    ui.colored_label(theme.palette().error, &self.status);
                } else {
                    ui.label(&self.status);
                }
            });
        self.open = open;
    }
}
