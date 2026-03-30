use eframe::egui;
use serde::Deserialize;
use std::sync::mpsc;

#[derive(PartialEq)]
enum SearchMode {
    Books,
    Owners,
}

pub enum MainAction {
    OpenAccount,
    OpenBookDetails(Book),
}

#[derive(Deserialize, Clone)]
pub struct Book {
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub owner_id: i32,
    pub available: i32,
    pub condition: i32,
    pub borrow_date: Option<String>,
}

#[derive(Deserialize)]
pub struct Owner {
    pub id: i32,
    pub name: String,
    pub lastname: String,
    pub mail: String,
}

#[derive(serde::Serialize)]
struct BookSearchRequest {
    isbn: Option<String>,
    title: Option<String>,
    author: Option<String>,
    owner_id: Option<i32>,
}

#[derive(serde::Serialize)]
struct OwnerSearchRequest {
    mail: Option<String>,
    name: Option<String>,
    lastname: Option<String>,
}

pub struct MainApp {
    pub token: String,

    pub books: Vec<Book>,
    pub owners: Vec<Owner>,
    pub loading: bool,
    pub status: String,

    rx_books: Option<mpsc::Receiver<Result<Vec<Book>, String>>>,
    rx_owners: Option<mpsc::Receiver<Result<Vec<Owner>, String>>>,

    search_mode: SearchMode,

    search_title: String,
    search_author: String,
    search_isbn: String,
    search_owner_id: String,

    search_mail: String,
    search_name: String,
    search_lastname: String,

    pub late_books: Vec<Book>,
    rx_late_books: Option<mpsc::Receiver<Result<Vec<Book>, String>>>,
    pub show_notifications: bool,

    selected_book: Option<Book>,
}

impl MainApp {
    pub fn new(token: String) -> Self {
    Self {
        token,
        books: vec![],
        owners: vec![],
        loading: false,
        status: "".into(),

        rx_books: None,
        rx_owners: None,

        search_mode: SearchMode::Books,

        search_title: "".into(),
        search_author: "".into(),
        search_isbn: "".into(),
        search_owner_id: "".into(),

        search_mail: "".into(),
        search_name: "".into(),
        search_lastname: "".into(),

        late_books: vec![],
        rx_late_books: None,
        show_notifications: false,
        selected_book: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) -> Option<MainAction> {
        if let Some(rx_books) = &self.rx_books {
            if let Ok(result) = rx_books.try_recv() {
                self.loading = false;

                match result {
                    Ok(books) => {
                        self.books = books;
                        self.status = "Loaded books!".into();
                    }
                    Err(err) => {
                        self.status = format!("Error: {}", err);
                    }
                }
            }
        }
        if let Some(rx_owners) = &self.rx_owners {
            if let Ok(result) = rx_owners.try_recv() {
                self.loading = false;

                match result {
                    Ok(owners) => {
                        self.owners = owners;
                        self.status = "Loaded owners!".into();
                    }
                    Err(err) => {
                        self.status = format!("Error: {}", err);
                    }
                }
            }
        }

        let mut action = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Library Manager");
            ui.horizontal(|ui| {
                if ui.button("Refresh").clicked() && !self.loading {
                    self.fetch_books(None, None, None, None);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let notif_label = if self.late_books.is_empty() {
                        "🔔".to_string()
                    } else {
                        format!("🔔 ({})", self.late_books.len())
                    };

                    if ui.button(notif_label).clicked() {
                        self.fetch_late_books();
                        self.show_notifications = !self.show_notifications;
                    }

                    if ui.button("☰").clicked() {
                        action = Some(MainAction::OpenAccount);
                    }
                });
            });

            ui.separator();

            ui.heading("Search Mode");
            ui.horizontal(|ui| {
                if ui.radio(self.search_mode == SearchMode::Books, "Books").clicked() {
                    self.search_mode = SearchMode::Books;
                }
                if ui.radio(self.search_mode == SearchMode::Owners, "Owners").clicked() {
                    self.search_mode = SearchMode::Owners;
                }
            });

            ui.separator();

            match self.search_mode {
                SearchMode::Books => {
                    ui.horizontal(|ui| {
                        ui.label("Title:"); ui.text_edit_singleline(&mut self.search_title);
                        ui.label("Author:"); ui.text_edit_singleline(&mut self.search_author);
                    });
                    ui.horizontal(|ui| {
                        ui.label("ISBN:"); ui.text_edit_singleline(&mut self.search_isbn);
                        ui.label("Owner ID:"); ui.text_edit_singleline(&mut self.search_owner_id);
                    });
                    if ui.button("Search Books").clicked() && !self.loading {
                        self.search_books();
                    }

                    ui.label(if self.loading { "⏳ Loading..." } else { &self.status });

                    ui.separator();
                    ui.heading("Books Results");
                    for book in &self.books {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("Titre: {}", book.title));

                                if ui.button("📖 View").clicked() {
                                    action = Some(MainAction::OpenBookDetails(book.clone()));
                                }
                            });
                        });
                    }
                }

                SearchMode::Owners => {
                    ui.horizontal(|ui| {
                        ui.label("Mail:"); ui.text_edit_singleline(&mut self.search_mail);
                        ui.label("Name:"); ui.text_edit_singleline(&mut self.search_name);
                        ui.label("Lastname:"); ui.text_edit_singleline(&mut self.search_lastname);
                    });
                    if ui.button("Search Owners").clicked() && !self.loading {
                        self.search_owners();
                    }

                    ui.label(if self.loading { "⏳ Loading..." } else { &self.status });

                    ui.separator();
                    ui.heading("Owners Results");
                    for owner in &self.owners {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} {} ({})", owner.name, owner.lastname, owner.mail));
                        });
                    }
                }
            }
        });

        if self.show_notifications {
            egui::Window::new("Late Borrowed Books")
                .open(&mut self.show_notifications)
                .show(ctx, |ui| {
                    if self.late_books.is_empty() {
                        ui.label("No late books");
                    } else {
                        for book in &self.late_books {
                            ui.label(format!(
                                "{} by {} (Owner ID: {})",
                                book.title, book.author, book.owner_id
                            ));
                        }
                    }
                });
        }

        action
    }
fn search_books(&mut self) {
    self.loading = true;
    self.status = "Searching...".into();
    let (tx, rx_books) = mpsc::channel();
    self.rx_books = Some(rx_books);
    let token = self.token.clone();
    let req = BookSearchRequest {
        isbn: if self.search_isbn.is_empty() { None } else { Some(self.search_isbn.clone()) },
        title: if self.search_title.is_empty() { None } else { Some(self.search_title.clone()) },
        author: if self.search_author.is_empty() { None } else { Some(self.search_author.clone()) },
        owner_id: self.search_owner_id.parse().ok(),
    };
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let client = reqwest::Client::new();
            let res = client
                .post("http://88.175.41.67:8080/search-books")
                .header("authorization", format!("Bearer {}", token))
                .json(&req)
                .send()
                .await;
            match res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        // Get raw text first so we can log it on failure
                        match resp.text().await {
                            Ok(body) => {
                                // Log the raw response to see exactly what the server returns
                                eprintln!("Raw response body: {}", body);

                                // Try direct Vec<Book> first
                                match serde_json::from_str::<Vec<Book>>(&body) {
                                    Ok(books) => {
                                        let _ = tx.send(Ok(books));
                                    }
                                    Err(e1) => {
                                        // Fall back to wrapped { "books": [...] }
                                        #[derive(Deserialize)]
                                        struct BooksResponse {
                                            books: Vec<Book>,
                                        }
                                        match serde_json::from_str::<BooksResponse>(&body) {
                                            Ok(data) => {
                                                let _ = tx.send(Ok(data.books));
                                            }
                                            Err(e2) => {
                                                let _ = tx.send(Err(format!(
                                                    "Parse error (array): {e1} | (wrapped): {e2} | body: {body}"
                                                )));
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(format!("Failed to read body: {e}")));
                            }
                        }
                    } else {
                        let text = resp.text().await.unwrap_or_default();
                        let _ = tx.send(Err(format!("Error response: {}", text)));
                    }
                }
                Err(err) => {
                    let _ = tx.send(Err(format!("Request failed: {}", err)));
                }
            }
        });
    });
}

    fn search_owners(&mut self) {
        self.loading = true;
        self.status = "Searching owners...".into();

        let (tx, rx_owners) = mpsc::channel();
        self.rx_owners = Some(rx_owners);

        let token = self.token.clone();

        let req = OwnerSearchRequest {
            mail: if self.search_mail.is_empty() { None } else { Some(self.search_mail.clone()) },
            name: if self.search_name.is_empty() { None } else { Some(self.search_name.clone()) },
            lastname: if self.search_lastname.is_empty() { None } else { Some(self.search_lastname.clone()) },
        };

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            rt.block_on(async move {
                let client = reqwest::Client::new();

                let res = client
                    .post("http://88.175.41.67:8080/search-owners")
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&req)
                    .send()
                    .await;

                match res {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            #[derive(Deserialize)]
                            struct OwnersResponse {
                                owners: Vec<Owner>,
                            }

                            match resp.json::<OwnersResponse>().await {
                                Ok(data) => {
                                    let _ = tx.send(Ok(data.owners));
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
                    Err(err) => {
                        let _ = tx.send(Err(format!("Request failed: {}", err)));
                    }
                }
            });
        });
    }

    fn fetch_books(&mut self, isbn: Option<String>, title: Option<String>, author: Option<String>, owner_id: Option<String>) {
        self.loading = true;
        self.status = "Loading, please wait.".into();

        let (tx, rx_books) = mpsc::channel();
        self.rx_books = Some(rx_books);

        let token = self.token.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            rt.block_on(async move {
                let client = reqwest::Client::new();

                #[derive(Deserialize)]
                struct BooksResponse {
                    books: Vec<Book>,
                }

                // Build the JSON body
                #[derive(serde::Serialize)]
                struct BookSearchRequest {
                    isbn: Option<String>,
                    title: Option<String>,
                    author: Option<String>,
                    owner_id: Option<String>,
                }

                let req = BookSearchRequest { isbn, title, author, owner_id };

                // Send POST request
                let res = client
                    .post("http://88.175.41.67:8080/search-books")
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&req)
                    .send()
                    .await;

                match res {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            match resp.json::<BooksResponse>().await {
                                Ok(data) => {
                                    let _ = tx.send(Ok(data.books));
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(format!("Parse error: {}", e)));
                                }
                            }
                        } else {
                            let text = resp.text().await.unwrap();
                            let _ = tx.send(Err(text));
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(Err(format!("Request failed: {}", err)));
                    }
                }
            });
        });
    }

    fn fetch_late_books(&mut self) {
        self.loading = true;
        self.status = "Loading notifications...".into();

        let (tx, rx) = mpsc::channel();
        self.rx_late_books = Some(rx);

        let token = self.token.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let client = reqwest::Client::new();

                #[derive(Deserialize)]
                struct LateBooksResponse {
                    late_books: Vec<Book>,
                }

                let res = client
                    .post("http://88.175.41.67:8080/late-borrowed-books")
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await;

                match res {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            match resp.json::<LateBooksResponse>().await {
                                Ok(data) => {
                                    let _ = tx.send(Ok(data.late_books));
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
                    Err(err) => {
                        let _ = tx.send(Err(format!("Request failed: {}", err)));
                    }
                }
            });
        });
    }
}
