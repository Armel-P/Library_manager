pub mod search_form;
pub mod forms_struct;

use search_form::SearchForm;
use forms_struct::{BookSearchForm, OwnerSearchForm};
use eframe::egui::{self};
use crate::{
    main_app::models::{Book, Owner},
    requests::structs::{
        BookSearchRequest, BooksResponse,
        OwnerSearchRequest, OwnersResponse
    },
    utils::{int_field},
};

impl SearchForm for BookSearchForm {
    type Item = Book;
    type Request = BookSearchRequest;
    type Response = BooksResponse;

    const ENDPOINT: &'static str = "book/search";
    const ENDPOINT_SAMPLE: &'static str = "book/sample";
    const ENDPOINT_COUNT: &'static str = "book/count";

    fn to_request(&self) -> BookSearchRequest {
        BookSearchRequest {
            isbn: self.isbn,
            title: (!self.title.trim().is_empty()).then(|| self.title.clone()),
            author: (!self.author.trim().is_empty()).then(|| self.author.clone()),
            owner_id: self.owner_id,
        }
    }

    fn unwrap_response(resp: BooksResponse) -> Vec<Book> {
        resp.books.into_iter().map(Book::from).collect()
    }

    fn clear(&mut self) {
        self.isbn = None;
        self.title = String::new();
        self.author = String::new();
        self.owner_id = None;
    }

    fn ui_fields(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();

        ui.allocate_ui_at_rect(rect, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Title:"); ui.text_edit_singleline(&mut self.title);
                ui.label("Author:"); ui.text_edit_singleline(&mut self.author);
                int_field(&mut self.isbn, "ISBN:", ui);
                int_field(&mut self.owner_id, "Owner ID:", ui);
            });
        });
    }

    fn render_item(book: &Book, ui: &mut egui::Ui) -> bool {
        let mut clicked = false;
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.strong(&book.title);
                        ui.label(format!("by {}", book.author));
                        let (status, color) = if book.available {
                            ("Available", egui::Color32::from_rgb(80, 180, 90))
                        } else {
                            ("Borrowed", egui::Color32::from_rgb(200, 90, 90))
                        };
                        ui.colored_label(color, status);
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("View").clicked() { clicked = true; }
                    });
                });
            });
        ui.add_space(4.0);
        clicked
    }

    fn check_all_empty(&self) -> bool {
        self.isbn.is_none() && self.title.trim().is_empty() && self.author.trim().is_empty() && self.owner_id.is_none()
    }
}


impl SearchForm for OwnerSearchForm {
    type Item = Owner;
    type Request = OwnerSearchRequest;
    type Response = OwnersResponse;

    const ENDPOINT: &'static str = "owner/search";
    const ENDPOINT_SAMPLE: &'static str = "owner/sample";
    const ENDPOINT_COUNT: &'static str = "owner/count";

    fn to_request(&self) -> OwnerSearchRequest {
        OwnerSearchRequest {
            name: (!self.name.trim().is_empty()).then(|| self.name.clone()),
            lastname: (!self.lastname.trim().is_empty()).then(|| self.lastname.clone()),
            mail: (!self.mail.trim().is_empty()).then(|| self.mail.clone()),
        }
    }

    fn unwrap_response(resp: OwnersResponse) -> Vec<Owner> { resp.owners }

    fn clear(&mut self) {
        self.name = String::new();
        self.lastname = String::new();
        self.mail = String::new();
    }

    fn ui_fields(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();

        ui.allocate_ui_at_rect(rect, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Mail:"); ui.text_edit_singleline(&mut self.mail);
                ui.label("Name:"); ui.text_edit_singleline(&mut self.name);
                ui.label("Lastname:"); ui.text_edit_singleline(&mut self.lastname);
            });
        });
    }

    fn render_item(owner: &Owner, ui: &mut egui::Ui) -> bool {
        let mut clicked = false;
        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.strong(&owner.id.to_string());
                        ui.label(format!("{} {} ({})", owner.name, owner.lastname, owner.mail));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("View").clicked() { clicked = true; }
                    });
                });
            });
        ui.add_space(4.0);
        clicked
    }


    fn check_all_empty(&self) -> bool {
        self.name.trim().is_empty() && self.lastname.trim().is_empty() && self.mail.trim().is_empty()
    }
}
