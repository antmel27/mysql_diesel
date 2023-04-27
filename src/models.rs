// Generated by diesel_ext

/* #![allow(unused)]
#![allow(clippy::all)] */


use diesel::{Queryable, Identifiable, Associations, Insertable};
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
include!("schema.rs");

#[derive(Queryable, Debug, Identifiable, Serialize, Deserialize)]
#[diesel(primary_key(book_id, tag_id))]
pub struct BookTag {
    pub book_id: i32,
    pub tag_id: i32,
}

#[derive(Queryable, Debug, Identifiable, Serialize, Deserialize)]
#[diesel(primary_key(bid))]
pub struct Book {
    pub bid: i32,
    pub isbn13: Option<String>,
    pub title: String,
    pub authors: String,
    pub edition: Option<String>,
    pub stock: Option<i32>,
    pub description: Option<String>,
}

#[derive(Queryable, Debug, Identifiable, Serialize, Deserialize)]
#[diesel(primary_key(coursebook_book_id, coursebook_course_id))]
pub struct Coursebook {
    pub coursebook_book_id: i32,
    pub coursebook_course_id: i32,
}

#[derive(Queryable, Debug, Identifiable, Serialize, Deserialize)]
#[diesel(primary_key(cid))]
pub struct Course {
    pub cid: i32,
    pub course_name: String,
    pub period_start: i32,
    pub period_end: i32,
    pub term_start: i32,
    pub term_end: i32,
}

#[derive(Queryable, Debug, Identifiable, Serialize, Deserialize)]
#[diesel(primary_key(tid))]
pub struct Tag {
    pub tid: i32,
    pub tag: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = userbooks)]
pub struct Userbook {
    pub borrow_id: i32,
    pub book_id: i32,
    pub borrow_date: NaiveDate,
    pub return_date: NaiveDate,
    pub user_id: i32,
} 

#[derive(Queryable, Debug, Identifiable, Serialize, Deserialize)]
#[diesel(primary_key(user_id))]
pub struct User {
    pub user_id: i32,
    pub subscription_status: bool,
    pub term: i32,
    pub year: i32,
    pub uid: String,
}

