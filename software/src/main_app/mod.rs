pub mod search_panel;
pub mod models;
pub mod forms;
pub mod subwin;

use search_panel::{SearchPanel};
use forms::forms_struct::{BookSearchForm, OwnerSearchForm};
use subwin::{notifications::NotificationsPanel,
            add_book::AddBookPanel, add_owner::AddOwnerPanel,
            book_details::BookDetailsPanel,
        };
use crate::{main_app::forms::search_form::SearchForm, theme::Theme};
use eframe::egui::{self, Pos2, Rect, Vec2, Stroke};

enum SearchMode {
    Books,
    Owners,
}

pub struct MainApp {
    books_panel: SearchPanel<BookSearchForm>,
    owners_panel: SearchPanel<OwnerSearchForm>,
    search_mode: SearchMode,

    notif_panel: NotificationsPanel,

    add_book_panel: AddBookPanel,
    add_owner_panel: AddOwnerPanel,

    book_details_panel: BookDetailsPanel,

    token: String,
    theme: Theme,
}

impl MainApp {
    pub fn new(token: String, theme: Theme) -> Self {
        Self {
            books_panel: SearchPanel::new(token.clone()),
            owners_panel: SearchPanel::new(token.clone()),
            search_mode: SearchMode::Books,
            notif_panel: NotificationsPanel::new(token.clone()),
            add_book_panel: AddBookPanel::new(),
            add_owner_panel: AddOwnerPanel::new(),
            book_details_panel: BookDetailsPanel::new(),
            token: token.clone(),
            theme: theme,
            // details panels
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        self.theme.apply(ctx);

        self.books_panel.poll();
        self.owners_panel.poll();

        self.notif_panel.ui(self.theme, ctx);
        self.notif_panel.poll();

        self.add_book_panel.ui(self.token.clone(), self.theme, ctx);
        self.add_book_panel.poll();

        self.add_owner_panel.ui(self.token.clone(), self.theme, ctx);
        self.add_owner_panel.poll();

        self.book_details_panel.ui(self.token.clone(), self.theme, ctx);
        self.book_details_panel.poll();

        egui::CentralPanel::default().show(ctx, |ui| {
            let full_rect = ui.max_rect();

            ui.allocate_ui_at_rect(
                Rect::from_min_size(
                    Pos2::new(full_rect.right() - 70.0, full_rect.top() + 10.0),
                    Vec2::new(80.0, 25.0),
                ),
                |ui| {
                    let label = self.theme.toggle_label();
                    if ui.button(label).clicked() { self.theme.toggle(); }

                    ui.add_space(4.0);

                    // Maybe wrap this in an other rect to have a button image instead of an emoji label
                    // But the actual way permit to easily display the number of late books
                    if ui.button(self.notif_panel.badge()).clicked() {
                        self.notif_panel.open = true;
                    }
                },
            );

            let screen = ctx.screen_rect();
            let card_size = egui::Vec2::new(screen.width() * 0.3, screen.height() * 0.1);
            let card_rect = Rect::from_center_size(Pos2 { x: screen.width() / 2.0, y: 60.0 }, card_size);
            ui.allocate_ui_at_rect(card_rect, |ui| {
                egui::Frame::none()
                    .rounding(16.0)
                    .fill(self.theme.palette().bg_bottom)
                    .stroke(Stroke::new(2.0, self.theme.palette().accent))
                    .inner_margin(egui::Margin::same(24.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Library Manager");
                        })
                    });
            });

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button("🔄").clicked() {
                    self.notif_panel.fetch(self.token.clone());
                    match self.search_mode {
                        SearchMode::Books => {
                            self.books_panel.form.clear();
                            self.books_panel.search();
                        }
                        SearchMode::Owners => {
                            self.owners_panel.form.clear();
                            self.owners_panel.search();
                        }
                    }
                }
                if ui.button("➕").clicked() { // put an emoji so it have a closer size the reset button
                    match self.search_mode {
                        SearchMode::Books => {
                            self.add_book_panel.open = true;
                        }
                        SearchMode::Owners => {
                            self.add_owner_panel.open = true;
                        }
                    }
                }
                if ui.radio(matches!(self.search_mode, SearchMode::Books), "Books").clicked() {
                    self.search_mode = SearchMode::Books;
                }
                if ui.radio(matches!(self.search_mode, SearchMode::Owners), "Owners").clicked() {
                    self.search_mode = SearchMode::Owners;
                }
            });
            ui.separator();

            match self.search_mode {
                SearchMode::Books => {
                    if let Some(i) = self.books_panel.ui(self.theme, ui) {
                        self.book_details_panel.open_for(self.books_panel.items[i].clone());
                    }
                }
                SearchMode::Owners => {
                    if let Some(i) = self.owners_panel.ui(self.theme, ui) {
                        // same idea for an OwnerDetailsPanel later
                    }
                }
            }
        });
    }
}
