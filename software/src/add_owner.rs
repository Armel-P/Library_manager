use eframe::egui;
use serde::Serialize;
use std::sync::mpsc;

#[derive(Serialize)]
struct AddOwnerRequest {
    name: String,
    lastname: String,
    mail: String,
}

pub enum AddOwnerAction {
    Back,
}

pub struct AddOwnerApp {
    pub token: String,

    name: String,
    lastname: String,
    mail: String,

    status: String,
    loading: bool,

    rx: Option<mpsc::Receiver<Result<String, String>>>,
}


impl AddOwnerApp {
    pub fn new(token: String) -> Self {
        Self {
            token,
            name: "".into(),
            lastname: "".into(),
            mail: "".into(),
            status: "".into(),
            loading: false,
            rx: None,
        }
    }

        pub fn update(&mut self, ctx: &egui::Context) -> Option<AddOwnerAction> {

        // Receive async result
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
            ui.heading("Add Owner");

            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
            ui.label("Last Name:");
            ui.text_edit_singleline(&mut self.lastname);
            ui.label("Mail:");
            ui.text_edit_singleline(&mut self.mail);

            if ui.button("Submit").clicked() && !self.loading {
                self.loading = true;
                self.status = "Sending...".into();

                let (tx, rx) = mpsc::channel();
                self.rx = Some(rx);

                let token = self.token.clone();

                let req = AddOwnerRequest {
                    name: self.name.clone(),
                    lastname: self.lastname.clone(),
                    mail: self.mail.clone(),
                };

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();

                    rt.block_on(async move {
                        let client = reqwest::Client::new();

                        let res = client
                            .post("http://88.175.41.67:8080/add-owner")
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
                action = Some(AddOwnerAction::Back);
            }
        });

        action
    }
}