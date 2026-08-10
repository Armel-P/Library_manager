use eframe::egui::{self, Pos2, Rect, Vec2, Stroke};
use std::sync::mpsc;
use reqwest::Method;
use crate::constants::{ENDPOINT_CREATE_USER, ENDPOINT_LOGIN};
use crate::requests::{spawn_request,
    LoginRequest, LoginResponse, CreateUserRequest, CreateUserResponse};
use crate::theme::{Theme};

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Login,
    CreateAccount,
}

pub struct LoginApp {
    username: String,
    password: String,
    admin_key: String,

    theme: Theme,
    mode: Mode,

    status: String,
    status_error: bool,
    loading: bool,

    login_rx: Option<mpsc::Receiver<Result<LoginResponse, String>>>,
    create_rx: Option<mpsc::Receiver<Result<CreateUserResponse, String>>>,

    token: Option<String>,
}

impl LoginApp {
    pub fn new(theme: Theme) -> Self {
        Self {
            username: "".into(),
            password: "".into(),
            admin_key: "".into(),
            theme: theme,
            mode: Mode::Login,
            status: "".into(),
            status_error: false,
            loading: false,
            login_rx: None,
            create_rx: None,
            token: None,
        }
    }
}

impl LoginApp {
    pub fn update(&mut self, ctx: &egui::Context) -> Option<String> {
        self.theme.apply(ctx);

        if let Some(rx) = &self.login_rx {
            if let Ok(result) = rx.try_recv() {
                self.loading = false;
                match result {
                    Ok(resp) => {
                        self.status = "Login successful!".into();
                        self.token = Some(resp.token);
                        self.status_error = false;
                    }
                    Err(err) => {
                        self.status = format!("Error: {err}");
                        self.status_error = true;
                    }
                }
                self.login_rx = None;
            }
        }

        if let Some(rx) = &self.create_rx {
            if let Ok(result) = rx.try_recv() {
                self.loading = false;
                match result {
                    Ok(_) => {
                        self.status = "Account created, you can log in now.".into();
                        self.mode = Mode::Login;
                        self.status_error = false;
                    }
                    Err(err) => {
                        self.status = format!("Error: {err}");
                        self.status_error = true;
                    }
                }
                self.create_rx = None;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let full_rect = ui.max_rect();

            ui.allocate_ui_at_rect(
                Rect::from_min_size(
                    Pos2::new(full_rect.right() - 90.0, full_rect.top() + 10.0),
                    Vec2::new(80.0, 24.0),
                ),
                |ui| {
                    let label = self.theme.toggle_label();
                    if ui.button(label).clicked() {
                        self.theme.toggle();
                    }
                },
            );

            let screen = ctx.screen_rect();
            let card_size = egui::Vec2::new(screen.width() * 0.35, screen.height() * 0.4);
            let card_rect = Rect::from_center_size(full_rect.center(), card_size);

            ui.allocate_ui_at_rect(card_rect, |ui| {
                egui::Frame::none()
                    .rounding(16.0)
                    .fill(self.theme.palette().bg_bottom)
                    .stroke(Stroke::new(2.0, self.theme.palette().accent))
                    .inner_margin(egui::Margin::same(24.0))
                    .show(ui, |ui| {
                        ui.set_min_size(card_size - Vec2::splat(48.0));
                        ui.vertical_centered(|ui| {
                            let title = match self.mode {
                                Mode::Login => "Login",
                                Mode::CreateAccount => "Create Account",
                            };
                            ui.label(egui::RichText::new(title).heading());
                            ui.add_space(12.0);

                            ui.label("Username");
                            ui.text_edit_singleline(&mut self.username);

                            ui.add_space(6.0);
                            ui.label("Password");
                            ui.add(egui::TextEdit::singleline(&mut self.password).password(true));

                            if self.mode == Mode::CreateAccount {
                                ui.add_space(6.0);
                                ui.label("Admin key");
                                ui.add(egui::TextEdit::singleline(&mut self.admin_key).password(true));
                            }

                            ui.add_space(16.0);

                            let action_label = match self.mode {
                                Mode::Login => "Login",
                                Mode::CreateAccount => "Create",
                            };

                            if ui.button(action_label).clicked() && !self.loading {
                                self.loading = true;
                                self.status_error = false;
                                self.status = match self.mode {
                                    Mode::Login => "Logging in...".into(),
                                    Mode::CreateAccount => "Creating account...".into(),
                                };

                                match self.mode {
                                    Mode::Login => {
                                        let rx = spawn_request::<LoginRequest, LoginResponse>(
                                            Method::POST,
                                            ENDPOINT_LOGIN.into(),
                                            Some(LoginRequest {
                                                username: self.username.clone(),
                                                password: self.password.clone(),
                                            }),
                                            None,
                                        );
                                        self.login_rx = Some(rx);
                                    }
                                    Mode::CreateAccount => {
                                        let rx = spawn_request::<CreateUserRequest, CreateUserResponse>(
                                            Method::POST,
                                            ENDPOINT_CREATE_USER.into(),
                                            Some(CreateUserRequest {
                                                username: self.username.clone(),
                                                password: self.password.clone(),
                                            }),
                                            Some(vec![(
                                                "XAdminKey".to_string(),
                                                self.admin_key.clone(),
                                            )]),
                                        );
                                        self.create_rx = Some(rx);
                                    }
                                }
                            }

                            ui.add_space(8.0);
                            let toggle_label = match self.mode {
                                Mode::Login => "No account yet? Create one",
                                Mode::CreateAccount => "Back to login",
                            };
                            if ui.link(toggle_label).clicked() {
                                self.mode = match self.mode {
                                    Mode::Login => Mode::CreateAccount,
                                    Mode::CreateAccount => Mode::Login,
                                };
                                self.status.clear();
                            }

                            ui.add_space(8.0);
                            if self.status_error {
                                ui.colored_label(self.theme.palette().error, &self.status);
                            } else {
                                ui.label(&self.status);
                            }
                        });
                    });
            });
        });

        self.token.take()
    }
}
