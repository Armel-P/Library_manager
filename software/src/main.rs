use eframe::egui;

mod constants;
mod requests;
mod theme;
mod login;
mod main_app;
mod utils;

enum AppState {
    Login(login::LoginApp),
    Main(main_app::MainApp),
    // Account(account::AccountApp), // maybe, not sure, will see after
}

struct MyApp {
    state: AppState,
    theme: theme::Theme,
}

impl Default for MyApp {
    fn default() -> Self {
        let theme = theme::Theme::default();
        Self {
            state: AppState::Login(login::LoginApp::new(theme)),
            theme: theme
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        match &mut self.state {
            AppState::Login(login_app) => {
                if let Some(token) = login_app.update(ctx) {
                    self.state = AppState::Main(main_app::MainApp::new(token, self.theme));
                }
            }

            AppState::Main(main_app) => {
                main_app.update(ctx);
            }
        }
    }
}

// Entry point
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Library Manager",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}
