use std::sync::mpsc;
use eframe::egui;
use reqwest::Method;
use crate::{
    requests,
    main_app::forms::{search_form::SearchForm},
    constants::{HEADER_TOKEN, ITEMS_PER_PAGE},
    theme::Theme,
    requests::structs::{SampleBookRequest, CountResponse}
};

pub struct SearchPanel<F: SearchForm> {
    pub form: F,
    pub items: Vec<F::Item>,
    pub status: String,
    pub status_error: bool,
    pub loading: bool,
    rx: Option<mpsc::Receiver<Result<F::Response, String>>>,
    count_rx: Option<mpsc::Receiver<Result<CountResponse, String>>>,
    token: String,
    page: i64,
    total_pages: i64,
}

impl<F: SearchForm> SearchPanel<F> {
    pub fn new(token: String) -> Self {
        let mut panel = Self {
            form: F::default(),
            items: vec![],
            status: "Loading...".into(),
            status_error: false,
            loading: false,
            rx: None,
            count_rx: None,
            token,
            page: 0,
            total_pages: 1,
        };
        panel.fetch_count();
        panel.search();
        panel
    }

    fn fetch_count(&mut self) {
        self.count_rx = Some(requests::spawn_request::<(), CountResponse>(
            Method::POST,
            F::ENDPOINT_COUNT.into(),
            None,
            Some(vec![(HEADER_TOKEN.to_string(), self.token.clone())]),
        ));
    }

    pub fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.loading = false;
                match result {
                    Ok(resp) => {
                        self.items = F::unwrap_response(resp);
                        self.status = "Loaded!".into();
                        self.status_error = false;
                    }
                    Err(e) => {
                        self.status = format!("Error: {e}");
                        self.status_error = true;
                    }
                }
                self.rx = None;
            }
        }

        if let Some(rx) = &self.count_rx {
            if let Ok(result) = rx.try_recv() {
                if let Ok(resp) = result {
                    self.total_pages = ((resp.count + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE).max(1);
                }
                self.count_rx = None;
            }
        }
    }

    pub fn search(&mut self) {
        self.loading = true;
        self.status = "Loading...".into();
        if self.form.check_all_empty() {
            let begin = self.page * ITEMS_PER_PAGE;
            let end = begin + ITEMS_PER_PAGE;
            self.rx = Some(requests::spawn_request::<SampleBookRequest, F::Response>(
                Method::POST,
                F::ENDPOINT_SAMPLE.into(),
                Some(SampleBookRequest { begin, end }),
                Some(vec![(HEADER_TOKEN.to_string(), self.token.clone())]),
            ));
        } else {
            self.rx = Some(requests::spawn_request::<F::Request, F::Response>(
                Method::POST,
                F::ENDPOINT.into(),
                Some(self.form.to_request()),
                Some(vec![(HEADER_TOKEN.to_string(), self.token.clone())]),
            ));
        }
    }

    fn go_to_page(&mut self, page: i64) {
        self.page = page.clamp(0, self.total_pages - 1);
        self.search();
    }

    fn pagination_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.add_enabled(self.page > 0, egui::Button::new("<- Prev")).clicked() {
                self.go_to_page(self.page - 1);
            }
            ui.label(format!("Page {} / {}", self.page + 1, self.total_pages));
            if ui.add_enabled(self.page + 1 < self.total_pages, egui::Button::new("Next ->")).clicked() {
                self.go_to_page(self.page + 1);
            }
        });
    }

    pub fn ui(&mut self, theme: Theme, ui: &mut egui::Ui) -> Option<usize> {
        self.form.ui_fields(ui);

        ui.add_space(4.0);

        if ui.button("Search").clicked() && !self.loading {
            self.search();
        }
        if self.status_error {
            ui.colored_label(theme.palette().error, &self.status);
        } else {
            ui.label(&self.status);
        }
        ui.separator();

        let mut clicked = None;
        for (i, item) in self.items.iter().enumerate() {
            ui.group(|ui| {
                if F::render_item(item, ui) { clicked = Some(i); }
            });
        }

        if self.form.check_all_empty() {
            ui.separator();
            self.pagination_ui(ui);
        }

        clicked
    }
}
