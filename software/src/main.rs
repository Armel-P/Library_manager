use eframe::egui;

mod login;
mod app;
mod add_book;
mod book_details;

// 🔁 App state (which screen are we on?)
enum AppState {
    Login(login::LoginApp),
    Main(app::MainApp),
    AddBook(add_book::AddBookApp),
    BookDetails(book_details::BookDetailsApp),
}

struct MyApp {
    state: AppState,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: AppState::Login(login::LoginApp::default()),
        }
    }
}

// 🎯 Core app logic
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        match &mut self.state {

            AppState::Login(login_app) => {
                if let Some(token) = login_app.update(ctx) {
                    self.state = AppState::Main(app::MainApp::new(token));
                }
            }

            AppState::Main(main_app) => {
                if let Some(action) = main_app.update(ctx) {
                    match action {
                        app::MainAction::OpenAddBook => {
                            self.state = AppState::AddBook(
                                add_book::AddBookApp::new(main_app.token.clone())
                            );
                        },
                        app::MainAction::OpenBookDetails(book) => {
                            self.state = AppState::BookDetails(
                                book_details::BookDetailsApp::new(book, main_app.token.clone())
                            );
                        },
                    }
                }
            }
            AppState::BookDetails(details_app) => {
                if details_app.update(ctx) {
                    // go back to main
                    self.state = AppState::Main(
                        app::MainApp::new(details_app.book.owner_id.to_string()) // ❗ see note below
                    );
                }
            }
            AppState::AddBook(add_book_app) => {
                if let Some(action) = add_book_app.update(ctx) {
                    match action {
                        add_book::AddBookAction::Back => {
                            self.state = AppState::Main(
                                app::MainApp::new(add_book_app.token.clone())
                            );
                        }
                    }
                }
            }
        }
    }
}

// 🚀 Entry point
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Library Manager",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}