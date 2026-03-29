use eframe::egui;
use serde::Deserialize;
use std::sync::mpsc;

pub enum MainAction {
    OpenAddBook,
}

#[derive(Deserialize)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub available: i32,
    pub borrow_date: String,
    pub owner_id: i32,
    // add other fields if needed
}

#[derive(Deserialize)]
struct BooksResponse {
    books: Vec<Book>,
}

#[derive(serde::Serialize)]
struct SearchRequest {
    isbn: Option<String>,
    title: Option<String>,
    author: Option<String>,
    owner_id: Option<i32>,
}

pub struct MainApp {
    pub token: String,

    pub books: Vec<Book>,
    pub loading: bool,
    pub status: String,

    rx: Option<mpsc::Receiver<Result<Vec<Book>, String>>>,

    search_title: String,
    search_author: String,
    search_isbn: String,
    search_owner_id: String,
}

impl MainApp {
    pub fn new(token: String) -> Self {
    Self {
        token,
        books: vec![],
        loading: false,
        status: "".into(),
        rx: None,
        search_title: "".into(),
        search_author: "".into(),
        search_isbn: "".into(),
        search_owner_id: "".into(),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) -> Option<MainAction> {
        // 🔁 Receive async result
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
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

        let mut action = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Late Borrowed Books");
            if ui.button("Refresh").clicked() && !self.loading {
    self.fetch_books(None, None, None, None); // ✅ provide 4 arguments
}
            if self.loading {
                ui.label("⏳ Loading...");
            } else {
                ui.label(&self.status);
            }
            if ui.button("➕ Add Book").clicked() {
                action = Some(MainAction::OpenAddBook);
            }
            ui.separator();

            // 📚 Display books
            for book in &self.books {
                ui.group(|ui| {
                    ui.label(format!("Titre: {}", book.title));
                });
            }
            ui.separator();
            ui.heading("🔍 Search");

            ui.horizontal(|ui| {
                ui.label("Title:");
                ui.text_edit_singleline(&mut self.search_title);

                ui.label("Author:");
                ui.text_edit_singleline(&mut self.search_author);
            });

            ui.horizontal(|ui| {
                ui.label("ISBN:");
                ui.text_edit_singleline(&mut self.search_isbn);

                ui.label("Owner ID:");
                ui.text_edit_singleline(&mut self.search_owner_id);
            });

            if ui.button("Search").clicked() && !self.loading {
                self.search_books();
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.search_books();
            }
        });

        action
    }

    fn search_books(&mut self) {
    self.loading = true;
    self.status = "Searching...".into();

    let (tx, rx) = mpsc::channel();
    self.rx = Some(rx);

    let token = self.token.clone();

    let req = SearchRequest {
        isbn: if self.search_isbn.is_empty() { None } else { Some(self.search_isbn.clone()) },
        title: if self.search_title.is_empty() { None } else { Some(self.search_title.clone()) },
        author: if self.search_author.is_empty() { None } else { Some(self.search_author.clone()) },
        owner_id: self.search_owner_id.parse().ok(),
    };

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            let client = reqwest::Client::new();

            // ✅ Use POST, not GET
            let res = client
                .post("http://88.175.41.67:8080/search-books")  // POST instead of GET
                .header("Authorization", format!("Bearer {}", token))
                .json(&req) // JSON body
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

    let (tx, rx) = mpsc::channel();
    self.rx = Some(rx);

    let token = self.token.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async move {
            let client = reqwest::Client::new();

            // Build the JSON body
            #[derive(serde::Serialize)]
            struct SearchRequest {
                isbn: Option<String>,
                title: Option<String>,
                author: Option<String>,
                owner_id: Option<String>,
            }

            let req = SearchRequest { isbn, title, author, owner_id };

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
}