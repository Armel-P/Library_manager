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
struct UserPrivilegeRequest {
    username: String,
    user_level: String,
}

pub enum UserPrivilegeAction {
    Back,
}

pub struct UserPrivilegeApp {
    pub token: String,

    username: String,
    admin_level: AdminLevel,

    status: String,
    loading: bool,

    rx: Option<mpsc::Receiver<Result<String, String>>>,
}


impl UserPrivilegeApp {
    pub fn new(token: String) -> Self {
        Self {
            token,
            username: "".into(),
            admin_level: AdminLevel::User,
            status: "".into(),
            loading: false,
            rx: None,
        }
    }

        pub fn update(&mut self, ctx: &egui::Context) -> Option<UserPrivilegeAction> {

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

            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);
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

                let req = UserPrivilegeRequest {
                    username: self.username.clone(),
                    user_level: self.admin_level.as_str().to_string(),
                };

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();

                    rt.block_on(async move {
                        let client = reqwest::Client::new();

                        let res = client
                            .post("http://88.175.41.67:8080/change-user-level")
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
                action = Some(UserPrivilegeAction::Back);
            }
        });

        action
    }
}