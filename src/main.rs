mod schema; //Import schema.rs
mod tables; //Import tables.rs
use tables::Book;

use diesel::{prelude::*, update};
use dotenv::dotenv;
use rocket::*;
use std::{env, time::SystemTime};
use sha256::*;

pub fn create_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[get("/")] //Function returning all books as string.
fn get_books() -> String {
    use schema::books::dsl::*; //Get the books table.
    let connection = &mut create_connection();
    let all_books = books
        .load::<Book>(connection) //Load all the books
        //.filter(id.eq(1)) --  In-case we later want to filter results.
        .expect("Error loading books"); //In-case of fail
    let mut book_list = String::new();

    for book in all_books {
        book_list.push_str(&serde_json::to_string(&book).unwrap());
    }

    book_list
}

#[get("/by-isbn/<isbn>")] //Function returning books based on ISBN13.
fn get_book_isbn(isbn: String) -> String {
    use schema::books::dsl::*; //Get the books table.
    let connection = &mut create_connection();
    let database_book = books.filter(isbn13.eq(isbn)).first::<Book>(connection); //Filter the result based on isbn.
    match database_book {
        Ok(book) => serde_json::to_string(&book).unwrap(),
        Err(_) => serde_json::to_string("Error retrieving book").unwrap(),
    }
}

#[get("/by-id/<bid_input>")] //Function returning books based on bid.
fn get_book_bid(bid_input: i32) -> String {
    use schema::books::dsl::*; //Get the books table.
    let connection = &mut create_connection();
    let database_book = books.filter(bid.eq(bid_input)).first::<Book>(connection);
    match database_book {
        Ok(book) => serde_json::to_string(&book).unwrap(),
        Err(_) => serde_json::to_string("Error retrieving book").unwrap(),
    }
}
#[get("/increase/<isbn>/<quantity>/<totp>")]
fn increase_stock(isbn: String, quantity: i32, totp: String) {
    if !totp_gen().contains(&totp) {
        return
    }

    use schema::books::dsl::*; //Get the books table.
    let isbn_clone = isbn.clone();
    let connection = &mut create_connection(); //Establish connection
    let query_book = books.filter(isbn13.eq(isbn)).first::<Book>(connection); //Get the book to update.
    let new_stock = match query_book {
        Ok(book) => book.stock.map(|other_stock| other_stock + quantity), //Need to use map since stock is optional value.
        Err(_) => None,
    };
    //Update the book with the new stock value.
    let __result = update(books.filter(isbn13.eq(isbn_clone))) //Find the row we want to update.
        .set(stock.eq(new_stock)) //Set new stock value.
        .execute(connection);
}

#[get("/decrease/<isbn>/<totp>")] // Decreases stock by 1.
fn decrease_stock(isbn: String, totp: String) {
    if !totp_gen().contains(&totp) {
        return
    }

    use schema::books::dsl::*; //Get the books table.
    let isbn_clone = isbn.clone();
    let connection = &mut create_connection(); //Establish connection
    let query_book = books.filter(isbn13.eq(isbn)).first::<Book>(connection); //Get the book to update.
    let new_stock = match query_book {
        Ok(book) => book.stock.map(|other_stock| other_stock - 1), //Need to use map since stock is optional value.
        Err(_) => None,
    };
    //Update the book with the new stock value.
    let __result = update(books.filter(isbn13.eq(isbn_clone))) //Find the row we want to update.
        .set(stock.eq(new_stock)) //Set new stock value.
        .execute(connection);
}

#[get("/user/<uid>")]
fn user_books(uid: String) -> String {
    todo!()
}


#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/books", routes![get_books, get_book_isbn, get_book_bid, increase_stock, decrease_stock])

}

fn totp_gen() -> Vec<String> {
    let current_tm = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("fuck")
    };
    let tm_0 = ((current_tm-30)/30).to_string();
    let tm_1 = (current_tm/30).to_string();
    let tm_2 = ((current_tm+30)/30).to_string();

    let key = env::var("TOTP_SECRET").expect("uhhhhhhhhh");
    let totp_0 = digest(format!("{}{}", key, tm_0));
    let totp_1 = digest(format!("{}{}", key, tm_1));
    let totp_2 = digest(format!("{}{}", key, tm_2));

    vec![
        totp_0[totp_0.len()-6..totp_0.len()].to_string(), 
        totp_1[totp_1.len()-6..totp_1.len()].to_string(), 
        totp_2[totp_2.len()-6..totp_2.len()].to_string()
        ]

}