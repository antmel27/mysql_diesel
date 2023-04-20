
extern crate diesel;
extern crate dotenv;
extern crate serde;

/* use diesel::Associations;
use diesel::Identifiable; */
use diesel::{Queryable/*Identifiable*/};
use serde::{Deserialize, Serialize};
//coursebooks relational table -- use mod coursebook?
/* #[derive(Debug, Queryable, Associations, Identifiable)]
#[belongs_to(Book, foreign_key = "book_id")]
#[belongs_to(Course, foreign_key = "course_id")]
#[diesel(primary_key(book_id, course_id))]
pub struct Coursebook {
    pub book_id: i32,
    pub course_id: i32,
} */

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


#[derive(Debug, Queryable)]
pub struct Course {
    pub cid: i32,
    pub title: String,
    pub instructor: String,
    pub period_start: i32,
    pub period_end: i32,
    pub term_start: i32,
    pub term_end: i32,
}

