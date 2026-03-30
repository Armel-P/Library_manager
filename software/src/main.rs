use eframe::egui;

mod login;
mod app;
mod book_details;
mod account;
mod add_book;
mod add_owner;
mod add_user;
mod user_privilege;

// App state (which screen are we on)
enum AppState {
    Login(login::LoginApp),
    Main(app::MainApp),
    BookDetails(book_details::BookDetailsApp),
    Account(account::AccountApp),
    AddBook(add_book::AddBookApp),
    AddOwner(add_owner::AddOwnerApp),
    AddUser(add_user::AddUserApp),
    UserPrivilege(user_privilege::UserPrivilegeApp),
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

// Core app logic
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
                        app::MainAction::OpenAccount => {
                            self.state = AppState::Account(
                                account::AccountApp::new(main_app.token.clone())
                            );
                        },
                        app::MainAction::OpenBookDetails(book) => {
                            self.state = AppState::BookDetails(
                                book_details::BookDetailsApp::new(book, main_app.token.clone())
                            );
                        }
                    }
                }
            }

            AppState::BookDetails(details_app) => {
                if details_app.update(ctx) {
                    self.state = AppState::Main(
                        app::MainApp::new(details_app.token.clone())
                    );
                }
            }

            AppState::Account(account_app) => {
                if let Some(action) = account_app.update(ctx) {
                    match action {
                        account::AccountAction::Back => {
                            self.state = AppState::Main(
                                app::MainApp::new(account_app.token.clone())
                            );
                        }
                        account::AccountAction::OpenAddBook => {
                            self.state = AppState::AddBook(
                                add_book::AddBookApp::new(account_app.token.clone())
                            );
                        }
                        account::AccountAction::OpenAddOwner => {
                            self.state = AppState::AddOwner(
                                add_owner::AddOwnerApp::new(account_app.token.clone())
                            );
                        }
                        account::AccountAction::OpenAddUser => {
                            self.state = AppState::AddUser(
                                add_user::AddUserApp::new(account_app.token.clone())
                            );
                        }
                        account::AccountAction::OpenUserPrivilege => {
                            self.state = AppState::UserPrivilege(
                                user_privilege::UserPrivilegeApp::new(account_app.token.clone())
                            );
                        }
                    }
                }
            }

            AppState::AddBook(add_book_app) => {
                if let Some(action) = add_book_app.update(ctx) {
                    match action {
                        add_book::AddBookAction::Back => {
                            self.state = AppState::Account(
                                account::AccountApp::new(add_book_app.token.clone())
                            );
                        }
                    }
                }
            }

            AppState::AddOwner(add_owner_app) => {
                if let Some(action) = add_owner_app.update(ctx) {
                    match action {
                        add_owner::AddOwnerAction::Back => {
                            self.state = AppState::Account(
                                account::AccountApp::new(add_owner_app.token.clone())
                            );
                        }
                    }
                }
            }

            AppState::AddUser(add_user_app) => {
                if let Some(action) = add_user_app.update(ctx) {
                    match action {
                        add_user::AddUserAction::Back => {
                            self.state = AppState::Account(
                                account::AccountApp::new(add_user_app.token.clone())
                            );
                        }
                    }
                }
            }

            AppState::UserPrivilege(user_privilege_app) => {
                if let Some(action) = user_privilege_app.update(ctx) {
                    match action {
                        user_privilege::UserPrivilegeAction::Back => {
                            self.state = AppState::Account(
                                account::AccountApp::new(user_privilege_app.token.clone())
                            );
                        }
                    }
                }
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