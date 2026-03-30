use eframe::egui;
use serde::Serialize;
use std::sync::mpsc;

#[derive(Clone, PartialEq)]
pub enum AdminLevel {
    User,
    Admin,
}

impl AdminLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdminLevel::User => "user",
            AdminLevel::Admin => "admin",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AdminLevel::User => "User",
            AdminLevel::Admin => "Admin",
        }
    }
}

#[derive(Serialize)]
struct AddUserRequest {
    username: String,
    password: String,
    user_level: String,
}

pub enum AddUserAction {
    Back,
}

pub struct AddUserApp {
    pub token: String,

    username: String,
    password: String,
    admin_level: AdminLevel,

    status: String,
    loading: bool,

    rx: Option<mpsc::Receiver<Result<String, String>>>,
}


impl AddUserApp {
    pub fn new(token: String) -> Self {
        Self {
            token,
            username: "".into(),
            password: "".into(),
            admin_level: AdminLevel::User,
            status: "".into(),
            loading: false,
            rx: None,
        }
    }

        pub fn update(&mut self, ctx: &egui::Context) -> Option<AddUserAction> {

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
            ui.heading("Add User");

            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);
            ui.label("Password:");
            ui.text_edit_singleline(&mut self.password);
            ui.label("User Level:");
            egui::ComboBox::from_label("")
                .selected_text(self.admin_level.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.admin_level, AdminLevel::User, AdminLevel::User.label());
                    ui.selectable_value(&mut self.admin_level, AdminLevel::Admin, AdminLevel::Admin.label());
                });

            if ui.button("Submit").clicked() && !self.loading {
                self.loading = true;
                self.status = "Sending...".into();

                let (tx, rx) = mpsc::channel();
                self.rx = Some(rx);

                let token = self.token.clone();

                let req = AddUserRequest {
                    username: self.username.clone(),
                    password: self.password.clone(),
                    user_level: self.admin_level.as_str().to_string(),
                };

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();

                    rt.block_on(async move {
                        let client = reqwest::Client::new();

                        let res = client
                            .post("http://88.175.41.67:8080/create-user")
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
                action = Some(AddUserAction::Back);
            }
        });

        action
    }
}
