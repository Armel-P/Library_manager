use eframe::egui;
use serde::{de::DeserializeOwned, Serialize};

pub trait SearchForm: Default + Send + 'static {
    type Item: Clone + Send + 'static;
    type Request: Serialize + Send + 'static;
    type Response: DeserializeOwned + Send + 'static;

    const ENDPOINT: &'static str;
    const ENDPOINT_SAMPLE: &'static str;
    const ENDPOINT_COUNT: &'static str;

    fn to_request(&self) -> Self::Request;
    fn unwrap_response(resp: Self::Response) -> Vec<Self::Item>;
    fn clear(&mut self);
    fn ui_fields(&mut self, ui: &mut egui::Ui);
    fn render_item(item: &Self::Item, ui: &mut egui::Ui) -> bool;
    fn check_all_empty(&self) -> bool;
}