use std::sync::mpsc;
use eframe::egui;
use reqwest::Method;
use crate::constants::HEADER_TOKEN;
use crate::requests::{self, structs::{AddOwnerRequest, AddOwnerResponse}};
use crate::theme::Theme;


pub struct AddOwnerPanel {
    pub open: bool,
    fields: AddOwnerRequest,
    status: String,
    status_error: bool,
    loading: bool,
    rx: Option<mpsc::Receiver<Result<AddOwnerResponse, String>>>
}

impl AddOwnerPanel {
    pub fn new() -> Self {
        AddOwnerPanel {
            open: false,
            fields: AddOwnerRequest {
                name: String::new(),
                lastname: String::new(),
                mail: String::new()
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

    fn cast_fields(&mut self) -> Option<AddOwnerRequest> {
        Some(self.fields.clone())
    }

    pub fn fetch(&mut self, token: String) {
        let body = match self.cast_fields() {
            Some(body) => { body },
            None => { return; }
        };
        self.loading = true;
        self.status = "Loading...".into();
        self.rx = Some(requests::spawn_request::<AddOwnerRequest, AddOwnerResponse>(
            Method::POST,
            "owner/create".into(),
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
                    ui.label("Name:"); ui.text_edit_singleline(&mut self.fields.name);
                });
                ui.horizontal(|ui|{
                    ui.label("Lastname:"); ui.text_edit_singleline(&mut self.fields.lastname);
                });
                ui.horizontal(|ui|{
                    ui.label("Mail:"); ui.text_edit_singleline(&mut self.fields.mail);
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
