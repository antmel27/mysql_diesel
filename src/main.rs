//First extern crate, then mod, then use.
extern crate diesel;
extern crate dotenv;
extern crate rocket;

mod schema; //Import schema.rs
mod tables; //Import tables.rs

use diesel::prelude::*;
use dotenv::dotenv;
use rocket::*;
use std::env;
use tables::Book;

pub fn create_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[get("/books")] //Function returning all books as string.
fn get_books() -> String {
    use schema::books::dsl::*; //Get the books table.
    let connection = &mut create_connection();
    let all_books = books
        .load::<Book>(connection) //Load all the books
        //.filter(id.eq(1)) --  In-case we later want to filter results.
        .expect("Error loading books"); //In-case of fail
    let mut book_list = String::new();

    for book in all_books {
        book_list.push_str(&format!("{:?}\n", book));
    }

    book_list
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![get_books])
}
