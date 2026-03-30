use eframe::egui;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

// ===== API TYPES =====

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

// ===== LOGIN APP =====

pub struct LoginApp {
    pub username: String,
    pub password: String,

    pub status: String,
    pub loading: bool,

    rx: Option<mpsc::Receiver<Result<String, String>>>,
    pub token: Option<String>,
}

impl Default for LoginApp {
    fn default() -> Self {
        Self {
            username: "".into(),
            password: "".into(),
            status: "".into(),
            loading: false,
            rx: None,
            token: None,
        }
    }
}

impl LoginApp {
    pub fn update(&mut self, ctx: &egui::Context) -> Option<String> {

        // Check async response
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.loading = false;

                match result {
                    Ok(token) => {
                        self.status = "Login successful!".into();
                        self.token = Some(token);
                    }
                    Err(err) => {
                        self.status = format!("Error: {}", err);
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Login");

                ui.add_space(10.0);

                // Username
                ui.label("Username:");
                let username_resp = ui.text_edit_singleline(&mut self.username);

                // Password
                ui.label("Password:");
                let password_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .password(true),
                );

                // Enter key submits
                let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let enter_submit =
                    (username_resp.lost_focus() || password_resp.lost_focus()) && enter_pressed;

                let clicked = ui.button("Login").clicked();

                if (clicked || enter_submit) && !self.loading {
                    self.loading = true;
                    self.status = "Logging in...".into();

                    let (tx, rx) = mpsc::channel();
                    self.rx = Some(rx);

                    let username = self.username.clone();
                    let password = self.password.clone();

                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();

                        rt.block_on(async move {
                            let client = reqwest::Client::new();

                            let res = client
                                .post("http://88.175.41.67:8080/get-token")
                                .json(&LoginRequest { username, password })
                                .send()
                                .await;

                            match res {
                                Ok(resp) => {
                                    if resp.status().is_success() {
                                        match resp.json::<LoginResponse>().await {
                                            Ok(data) => {
                                                let _ = tx.send(Ok(data.token));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(Err(format!("Invalid response: {}", e)));
                                            }
                                        }
                                    } else {
                                        match resp.json::<ErrorResponse>().await {
                                            Ok(err) => {
                                                let _ = tx.send(Err(err.error));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(Err(format!("Error parsing error: {}", e)));
                                            }
                                        }
                                    }
                                }
                                Err(err) => {
                                    let _ = tx.send(Err(format!("Request failed: {}", err)));
                                }
                            }
                        });
                    });
                }

                ui.add_space(10.0);

                // Status
                if self.loading {
                    ui.label("⏳ Logging in...");
                } else {
                    ui.label(&self.status);
                }
            });
        });

        // Return token once (triggers screen switch)
        self.token.take()
    }
}