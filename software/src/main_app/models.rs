use serde::{Deserialize};


#[derive(Deserialize, Clone)]
pub struct Book {
    pub isbn: i64,
    pub title: String,
    pub author: String,
    pub owner_id: i64,
    pub available: bool,
    pub condition: i8,
    pub borrow_date: Option<String>,
    pub borrower_mail: Option<String>,
}

impl From<BookRow> for Book {
    fn from(row: BookRow) -> Self {
        Book {
            isbn: row.isbn,
            title: row.title,
            author: row.author,
            owner_id: row.owner_id,
            available: row.available != 0,
            condition: row.condition,
            borrow_date: row.borrow_date,
            borrower_mail: row.borrower_mail,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct BookRow {
    pub isbn: i64,
    pub title: String,
    pub author: String,
    pub owner_id: i64,
    pub available: i8,
    pub condition: i8,
    pub borrow_date: Option<String>,
    pub borrower_mail: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Owner {
    pub id: i64,
    pub name: String,
    pub lastname: String,
    pub mail: String,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Condition {
    Bad = -1,
    Acceptable = 0,
    Good = 1,
    Excellent = 2,
    New = 3,
}

impl Condition {
    pub const ALL: [Condition; 5] = [
        Condition::Bad, Condition::Acceptable, Condition::Good,
        Condition::Excellent, Condition::New,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Condition::Bad => "Bad",
            Condition::Acceptable => "Acceptable",
            Condition::Good => "Good",
            Condition::Excellent => "Excellent",
            Condition::New => "New",
        }
    }
}

impl TryFrom<i8> for Condition {
    type Error = ();

    fn try_from(v: i8) -> Result<Self, Self::Error> {
        Condition::ALL
            .into_iter()
            .find(|c| i8::from(*c) == v)
            .ok_or(())
    }
}

impl From<Condition> for i8 {
    fn from(c: Condition) -> i8 {
        c as i8
    }
}
