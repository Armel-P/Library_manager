use eframe::egui;

#[derive(serde::Deserialize)]
struct UserInfo {
    username: String,
    user_role: i32,
}

fn role_to_string(role: i32) -> &'static str {
    match role {
        1 => "Admin",
        0 => "User",
        _ => "Unknown",
    }
}

#[derive(serde::Deserialize)]
struct UserResponse {
    user: UserInfo,
}

pub enum AccountAction {
    Back,
    OpenAddBook,
    OpenAddOwner,
    OpenAddUser,
    OpenUserPrivilege,
}

pub struct AccountApp {
    pub token: String,
    username: String,
    user_level: String,
    loading: bool,
    status: String,
    rx: Option<std::sync::mpsc::Receiver<Result<(String, String), String>>>,
}

impl AccountApp {
    pub fn new(token: String) -> Self {
        Self {
            token,
            username: "".into(),
            user_level: "".into(),
            loading: true,
            status: "".into(),
            rx: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) -> Option<AccountAction> {
        if self.loading && self.rx.is_none() {
            self.fetch_user_info();
        }
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.loading = false;
                match result {
                    Ok((username, level)) => {
                        self.username = username;
                        self.user_level = level;
                    }
                    Err(err) => {
                        self.status = format!("Error: {}", err);
                    }
                }
            }
        }

        let mut action = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Account");

            if self.loading {
                ui.label("⏳ Loading user info...");
            } else {
                ui.label(format!("Hi {}, {}", self.username, self.user_level));
            }

            ui.separator();

            if ui.button("➕ Add Book").clicked() {
                action = Some(AccountAction::OpenAddBook);
            }

            if ui.button("➕ Add Owner").clicked() {
                action = Some(AccountAction::OpenAddOwner);
            }

            if ui.button("👤 Add User").clicked() {
                action = Some(AccountAction::OpenAddUser);
            }

            if ui.button("Promote/Demote User").clicked() {
                action = Some(AccountAction::OpenUserPrivilege);
            }

            ui.separator();

            if ui.button("⬅ Back").clicked() {
                action = Some(AccountAction::Back);
            }
        });

        action
    }

    pub fn fetch_user_info(&mut self) {
        self.loading = true;
        self.status = "Loading user info...".into();

        let token = self.token.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.rx = Some(rx);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let client = reqwest::Client::new();

                let res = client
                    .get("http://88.175.41.67:8080/get-user-infos")
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await;

                match res {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            match resp.json::<UserResponse>().await {
                                Ok(data) => {
                                    let username = data.user.username;
                                    let level = role_to_string(data.user.user_role).to_string();
                                    let _ = tx.send(Ok((username, level)));
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(format!("Parse error: {}", e)));
                                }
                            }
                        } else {
                            let text = resp.text().await.unwrap_or_default();
                            let _ = tx.send(Err(format!("Error response: {}", text)));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(format!("Request failed: {}", e)));
                    }
                }
            });
        });
    }
}
