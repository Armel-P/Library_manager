use eframe::egui;
use crate::app::Book;
use std::sync::mpsc;

pub struct BookDetailsApp {
    pub book: Book,
    pub token: String,

    borrower_mail: String,
    status: String,
    loading: bool,
    rx: Option<mpsc::Receiver<Result<String, String>>>,
}

impl BookDetailsApp {
    pub fn new(book: Book, token: String) -> Self {
        Self {
            book,
            token,
            borrower_mail: "".into(),
            status: "".into(),
            loading: false,
            rx: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) -> bool {
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.loading = false;

                match result {
                    Ok(msg) => self.status = msg,
                    Err(err) => self.status = format!("Error: {}", err),
                }
            }
        }

        let mut go_back = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📖 Book Details");

            ui.label(format!("ISBN: {}", self.book.isbn));
            ui.label(format!("Title: {}", self.book.title));
            ui.label(format!("Author: {}", self.book.author));
            ui.label(format!("Owner ID: {}", self.book.owner_id));
            ui.label(format!("Available: {}", self.book.available));
            ui.label(format!("Condition: {}", self.book.condition));

            ui.label(format!(
                "Borrow date: {}",
                self.book.borrow_date.clone().unwrap_or("N/A".into())
            ));

            ui.separator();

            ui.heading("Borrow Book");

            ui.horizontal(|ui| {
                ui.label("Borrower email:");
                ui.text_edit_singleline(&mut self.borrower_mail);
            });

            if ui
                .add_enabled(!self.borrower_mail.is_empty() && !self.loading, egui::Button::new("Borrow"))
                .clicked()
            {
                self.borrow_book();
            }

            ui.label(if self.loading {
                "⏳ Borrowing..."
            } else {
                &self.status
            });

            ui.separator();

            if ui.button("⬅ Back").clicked() {
                go_back = true;
            }
        });

        go_back
    }

    fn borrow_book(&mut self) {
        self.loading = true;
        self.status = "Sending request...".into();

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let token = self.token.clone();
        let isbn = self.book.isbn.clone();
        let borrower_mail = self.borrower_mail.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            rt.block_on(async move {
                let client = reqwest::Client::new();

                #[derive(serde::Serialize)]
                struct BorrowRequest {
                    isbn: String,
                    borrower_mail: String,
                }

                let res = client
                    .post("http://88.175.41.67:8080/borrow-book")
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&BorrowRequest { isbn, borrower_mail })
                    .send()
                    .await;

                match res {
                    Ok(resp) => {
                        let status = resp.status();
                        let text = resp.text().await.unwrap_or_default();

                        if status.is_success() {
                            let _ = tx.send(Ok(format!("{}", text)));
                        } else {
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