use eframe::egui;
use serde::Serialize;
use std::sync::mpsc;

#[derive(Serialize)]
struct AddBookRequest {
    isbn: String,
    title: String,
    author: String,
    owner_id: i32,
    condition: String,
}

pub enum AddBookAction {
    Back,
}

pub struct AddBookApp {
    pub token: String,

    isbn: String,
    title: String,
    author: String,
    owner_id: String,
    condition: String,

    status: String,
    loading: bool,

    rx: Option<mpsc::Receiver<Result<String, String>>>,
}

impl AddBookApp {
    pub fn new(token: String) -> Self {
        Self {
            token,
            isbn: "".into(),
            title: "".into(),
            author: "".into(),
            owner_id: "".into(),
            condition: "good".into(),
            status: "".into(),
            loading: false,
            rx: None,
        }
    }
        pub fn update(&mut self, ctx: &egui::Context) -> Option<AddBookAction> {

        // 🔁 Receive async result
        let mut action = None;
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.loading = false;

                match result {
                    Ok(msg) => self.status = msg,
                    Err(err) => self.status = format!("Error: {}", err),
                }
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Add Book");

            ui.label("ID:");
            ui.text_edit_singleline(&mut self.isbn);
            ui.label("Titre:");
            ui.text_edit_singleline(&mut self.title);
            ui.label("Autheur:");
            ui.text_edit_singleline(&mut self.author);
            ui.label("Emprunteur:");
            ui.text_edit_singleline(&mut self.owner_id);
            ui.label("Condition:");
            ui.text_edit_singleline(&mut self.condition);

            if ui.button("Submit").clicked() && !self.loading {
                self.loading = true;
                self.status = "Sending...".into();

                let (tx, rx) = mpsc::channel();
                self.rx = Some(rx);

                let token = self.token.clone();

                let req = AddBookRequest {
                    isbn: self.isbn.clone(),
                    title: self.title.clone(),
                    author: self.author.clone(),
                    owner_id: self.owner_id.parse().unwrap_or(0),
                    condition: self.condition.clone(),
                };

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();

                    rt.block_on(async move {
                        let client = reqwest::Client::new();

                        let res = client
                            .post("http://88.175.41.67:8080/add-book")
                            .header("Authorization", format!("Bearer {}", token))
                            .json(&req)
                            .send()
                            .await;

                        match res {
                            Ok(resp) => {
                                let text = resp.text().await.unwrap();
                                let _ = tx.send(Ok(text));
                            }
                            Err(err) => {
                                let _ = tx.send(Err(err.to_string()));
                            }
                        }
                    });
                });
            }

            if self.loading {
                ui.label("⏳ Sending...");
            } else {
                ui.label(&self.status);
            }

            ui.separator();

            if ui.button("⬅ Back").clicked() {
                action = Some(AddBookAction::Back);
            }
        });

        action
    }
}