
extern crate diesel;
extern crate dotenv;
extern crate serde;
//extern crate chrono;

use diesel::{Queryable, Identifiable, Associations};
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
#[derive(Debug, Queryable, Associations, /*Identifiable*/)]
#[belongs_to(Book, foreign_key = "book_id")]
#[belongs_to(Course, foreign_key = "course_id")]
//#[diesel(primary_key(book_id, course_id))]
pub struct Coursebook {
    pub book_id: i32,
    pub course_id: i32,
} 

#[derive(Debug, Clone, Queryable, Serialize, Deserialize)]
pub struct Book {
    pub bid: i32,
    pub isbn13: Option<String>,
    pub title: String,
    pub authors: String,
    pub edition: Option<String>,
    pub stock: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct Course {
    pub cid: i32,
    pub title: String,
    pub instructor: String,
    pub period_start: i32,
    pub period_end: i32,
    pub term_start: i32,
    pub term_end: i32,
}
#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct Userbook {
    pub borrow_id: i32,
    pub book_id: i32,
    pub borrow_date: NaiveDate,
    pub return_date: NaiveDate,
    pub user_id: i32, //relational?
} 

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct User {
    pub user_id: i32, //generic user id
    pub subscription_status: bool,
    pub term: i32,
    pub year: i32,
    pub user_token_id: String, //token
}

