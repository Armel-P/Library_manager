#[derive(Default)]
pub struct BookSearchForm {
    pub isbn: Option<i64>,
    pub title: String,
    pub author: String,
    pub owner_id: Option<i64>,
}

#[derive(Default)]
pub struct OwnerSearchForm {
    pub name: String,
    pub lastname: String,
    pub mail: String,
}
