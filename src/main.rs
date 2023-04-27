//First extern crate, then mod, then use.
extern crate diesel;
extern crate dotenv;
extern crate rocket;
extern crate serde_json;
extern crate chrono;
mod schema; //Import schema.rs
//mod tables; //Import tables.rs
mod models; //Import models.rs

use diesel::{prelude::*, update, insert_into};
use dotenv::dotenv;
use rocket::*;
use std::{env, time::SystemTime};
use sha256::*;

use models::*;
use chrono::{NaiveDate, Local};

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
        .expect("Error loading books"); //In-case of fail
    let mut book_list = String::new();

    for book in all_books {
        book_list.push_str(&serde_json::to_string(&book).unwrap());
    }

    book_list
}

#[get("/books/by-isbn/<isbn>")] //Function returning books based on ISBN13.
fn get_book_isbn(isbn: String) -> String {
    use schema::books::dsl::*; //Get the books table.
    let connection = &mut create_connection();
    let database_book = books.filter(isbn13.eq(isbn)).first::<Book>(connection); //Filter the result based on isbn.
    match database_book {
        Ok(book) => serde_json::to_string(&book).unwrap(),
        Err(_) => serde_json::to_string("Error retrieving book").unwrap(),
    }
}

#[get("/books/by-id/<bid_input>")] //Function returning books based on bid.
fn get_book_bid(bid_input: i32) -> String {
    use schema::books::dsl::*; //Get the books table.
    let connection = &mut create_connection();
    let database_book = books.filter(bid.eq(bid_input)).first::<Book>(connection);
    match database_book {
        Ok(book) => serde_json::to_string(&book).unwrap(),
        Err(_) => serde_json::to_string("Error retrieving book").unwrap(),
    }
}

#[get("/books/related-course/by-id/<bid>")]
fn get_related_course_by_id(bid: i32) -> String
{
    use schema::coursebooks::dsl::*;
    use schema::courses::dsl::*;

    let connection = &mut create_connection(); //Establish connection
    let related_course_bids = coursebooks.filter(coursebook_book_id.eq(bid))
        .load::<Coursebook>(connection) //Load all the coursebooks that have a matching bid.
        .expect("Error loading coursebooks"); //In-case of fail
        let mut course_string = String::new();

    for coursebook in related_course_bids { //Iterate over all the found coursebooks.
        let related_course = courses
        .filter(cid.eq(coursebook.coursebook_course_id))
        .first::<Course>(connection)
        .expect("Error loading related courses");
        course_string.push_str(&related_course.course_name);
        course_string.push_str(",");
        course_string.push_str(" ");
    }
    course_string.pop();
    course_string.pop();
    course_string
    //Establish connection,
    //Look in coursebook table for related course.
    //Get the coursenames.

}
#[get("/increase/<isbn>/<quantity>")]
fn increase_stock(isbn: String, quantity: i32) {
/*     if !totp_gen().contains(&totp) {
        return
    } */

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

#[get("/decrease/<isbn>")] // Decreases stock by 1.
fn decrease_stock(isbn: String) {
/*     if !totp_gen().contains(&totp) {
        return
    } */

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


#[get("/borrow/<address>/<husnummer>/<postkod>/<stad>/<isbn>/<token>")]
fn borrow_book(address: String, husnummer: String, postkod: i32, stad: String, isbn: String, token: String)
{ //Function for a user to borrow book.
    use schema::books::dsl::*;
    use schema::userbooks::dsl::*;
    use schema::coursebooks::dsl::*;
    use schema::userbooks::columns::user_id;
    use schema::courses::dsl::*;
    use schema::users::dsl::*;
    
    // TOKEN AUTHENTICATION INSERT HERE //
    // NEED BOOK_ID, BORROW_DATE, RETURN_DATE, USER_ID
    let isbn_clone = isbn.clone(); //Clone because isbn is moved later.
    let connection = &mut create_connection(); //Establish connection
    
    //GETTING BOOK_ID
    let found_book = books.filter(isbn13.eq(isbn)).first::<Book>(connection).unwrap(); //Get the book we want to borrow.
    let book_bid = found_book.bid; //Get the book id.
    //ONLY FIRST BELOW?
    let coursebook_entry = coursebooks.filter(coursebook_book_id.eq(book_bid)).first::<Coursebook>(connection).unwrap(); //
    let course_id = coursebook_entry.coursebook_book_id;
    
    let found_course = courses.filter(cid.eq(course_id)).first::<Course>(connection).unwrap(); //Get the associated course.
    let course_end = found_course.period_end;
    
    let borrow_dat = Local::now().date_naive();
    let return_dat = match course_end {
        1 => NaiveDate::from_ymd_opt(2023, 11, 30),
        2 => NaiveDate::from_ymd_opt(2023, 1, 30),
        3 => NaiveDate::from_ymd_opt(2024, 3, 30),
        4 => NaiveDate::from_ymd_opt(2023, 6, 30),
        5 => NaiveDate::from_ymd_opt(2023, 9, 31),
        _ => NaiveDate::from_ymd_opt(1970, 1, 1), //If out of bounds period.
        }.unwrap(); //Get the return date.
    let associated_user_id = users.filter(uid.eq(token)).first::<User>(connection).unwrap().user_id;
/*     let new_userbook_entry = Userbook {
        borrow_id: 1,
        book_id: book_bid,
        borrow_date: borrow_dat,
        return_date: return_dat,
        user_id: associated_user_id,
    }; */
    
    insert_into(userbooks)
        .values((book_id.eq(book_bid), borrow_date.eq(borrow_dat), return_date.eq(return_dat), user_id.eq(associated_user_id)))
        .execute(connection);
    
    //Check what period course is in.
    decrease_stock(isbn_clone); //Decrease the stock of book when it has been succesfully borrowed.
} 
/* #[get("/userbooks/<uid>/<token>")] //Function to get all books
fn get_userbooks(uid: String, token: String) -> String
{
    return format!("ben")
    //Get the book id's from all the userbook tables, use them to search through the 
} */
//#[get("userbooks/termin")] - få ut böcker sorterade via termin
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![get_books])
        .mount("/", routes![get_book_isbn])
        .mount("/", routes![get_book_bid])
        .mount("/", routes![increase_stock])
        .mount("/", routes![decrease_stock])
        .mount("/", routes![get_related_course_by_id])
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