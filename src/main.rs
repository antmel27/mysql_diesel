//First extern crate, then mod, then use.
extern crate diesel;
extern crate dotenv;
extern crate rocket;
extern crate serde_json;
extern crate chrono;

mod schema; //Import schema.rs
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
    let all_books_option = books.load::<Book>(connection); //Load all the books
    
    let all_books = match all_books_option {
        Ok(all_books) => all_books,
        Err(err) => return format!("Error: {}", err),
    };
    
    let mut book_list = String::new();
    for book in all_books {
        book_list.push_str(&serde_json::to_string(&book).unwrap());
    }
    return book_list
}

#[get("/books/by-isbn/<isbn>")] //Function returning books based on ISBN13.
fn get_book_isbn(isbn: String) -> String {
    use schema::books::dsl::*; //Get the books table.
    let connection = &mut create_connection();
    let database_book = books.filter(isbn13.eq(isbn)).first::<Book>(connection); //Filter the result based on isbn.
    match database_book {
        Ok(book) => serde_json::to_string(&book).unwrap(),
        Err(_) => serde_json::to_string("Error: Couldn't retrieve book with such ISBN13.").unwrap(),
    }
}

#[get("/books/by-id/<bid_input>")] //Function returning books based on bid.
fn get_book_bid(bid_input: i32) -> String {
    use schema::books::dsl::*; //Get the books table.
    let connection = &mut create_connection();
    let database_book = books.filter(bid.eq(bid_input)).first::<Book>(connection);
    match database_book {
        Ok(book) => serde_json::to_string(&book).unwrap(),
        Err(_) => serde_json::to_string("Error: Couldn't retrieve book with such bid.").unwrap(),
    }
}

#[get("/books/related-course/by-id/<bid>")]
fn get_related_course_by_id(bid: i32) -> String
{
    use schema::coursebooks::dsl::*;
    use schema::courses::dsl::*;

    let connection = &mut create_connection(); //Establish connection
    let related_course_bids_option: Result<Vec<Coursebook>, diesel::result::Error> = coursebooks.filter(coursebook_book_id.eq(bid)).load::<Coursebook>(connection); //Load all the coursebooks that have a matching bid.
    let related_course_bids = match related_course_bids_option {
        Ok(related_course_bids_result) => related_course_bids_result,
        Err(err) => return format!("Error: {}", err),
    };
    
    let mut course_string = String::new();
    for coursebook in related_course_bids { //Iterate over all the found coursebooks.
        let related_course_option = courses
        .filter(cid.eq(coursebook.coursebook_course_id))
        .first::<Course>(connection);
        
        let related_course = match related_course_option {
            Ok(related_course_result) => related_course_result,
            Err(_) => continue, //If error simply skip to next iteration of loop.
        };
        
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
fn increase_stock(isbn: String, quantity: i32) -> String {

    use schema::books::dsl::*; 
    
    let isbn_clone = isbn.clone();
    let connection = &mut create_connection(); //Establish connection
    let query_book = books.filter(isbn13.eq(isbn)).first::<Book>(connection); //Get the book to update.
    let new_stock = match query_book {
        Ok(book) => book.stock.map(|other_stock| other_stock + quantity), //Need to use map since stock is optional value.
        Err(err) => return format!("Error: {}", err),
    };
    //Update the book with the new stock value.
    let result = update(books.filter(isbn13.eq(isbn_clone))) //Find the row we want to update.
        .set(stock.eq(new_stock)) //Set new stock value.
        .execute(connection);
    match result {
        Ok(_) => return format!("Succesfully increased stock."),
        Err(err) => return format!("Error: {}", err)
    };
}

#[get("/decrease/<isbn>")] // Decreases stock by 1.
fn decrease_stock(isbn: String) -> String {
/*     if !totp_gen().contains(&totp) {
        return
    } */

    use schema::books::dsl::*; //Get the books table.
    let isbn_clone = isbn.clone();
    let connection = &mut create_connection(); //Establish connection
    let query_book = books.filter(isbn13.eq(isbn)).first::<Book>(connection); //Get the book to update.
    let new_stock = match query_book {
        Ok(book) => book.stock.map(|other_stock| other_stock - 1), //Need to use map since stock is optional value.
        Err(_) => None, //If stock was null we simply do nothing.
    };

    let result = update(books.filter(isbn13.eq(isbn_clone))) //Find the row we want to update.
        .set(stock.eq(new_stock)) //Set new stock value.
        .execute(connection);
    match result {
        Ok(_) => return format!("Succesfully decreased stock"),
        Err(err) => return format!("Error: {}", err)
    };
}


#[get("/borrow/<name_of_reciever>/<address>/<husnummer>/<postkod>/<stad>/<isbn>/<token>")]
async fn borrow_book(name_of_reciever: String, address: String, husnummer: String, postkod: i32, stad: String, isbn: String, token: String) -> String
{ //Function for a user to borrow book.
    use schema::books::dsl::*;
    use schema::userbooks::dsl::*;
    use schema::coursebooks::dsl::*;
    use schema::userbooks::columns::user_id;
    use schema::courses::dsl::*;
    use schema::users::dsl::*;
    
    // TOKEN AUTHENTICATION INSERT HERE //

    let client = reqwest::Client::new();
	let token_uid: String = match client.get(format!("https://courseLend.akerhielm.nu/auth/whoami/{token}"))
	.send().await {
		Ok(res) => match res.text().await {
			Ok(res) => {
				let sub: serde_json::Value = serde_json::from_str(&res).unwrap();
				if !sub["sub"].is_null() {
					return "Could not authenticate".to_string()
				} else {
                    sub["sub"].as_str().unwrap().to_string()
                }
			},
			Err(_) => return "Could not authenticate".to_string()
		},
		Err(_) => return "Could not authenticate".to_string()
	};


    // NEED BOOK_ID, BORROW_DATE, RETURN_DATE, USER_ID
    let isbn_clone = isbn.clone(); //Clone because isbn is moved later.
    let connection = &mut create_connection(); //Establish connection
    
    //GETTING BOOK_ID
    let found_book_option: Result<Book, diesel::result::Error> = books.filter(isbn13.eq(isbn)).first::<Book>(connection); //Get the book we want to borrow.
    let found_book = match found_book_option {
        Ok(found_book_result) => found_book_result,
        Err(err) => return format!("Error: {}", err)
    };
    let book_bid = found_book.bid; //Get the book id.

    let coursebook_entry_option: Result<Coursebook, diesel::result::Error> = coursebooks.filter(coursebook_book_id.eq(book_bid)).first::<Coursebook>(connection);
    let coursebook_entry = match coursebook_entry_option {
        Ok(coursebook_entry_result) => coursebook_entry_result,
        Err(err) => return format!("Error: {}", err)
    };
    let course_id = coursebook_entry.coursebook_book_id;

    let found_course_option: Result<Course, diesel::result::Error> = courses.filter(cid.eq(course_id)).first::<Course>(connection); //Get the associated course.
    let found_course = match found_course_option {
        Ok(found_course_result) => found_course_result,
        Err(err) => return format!("Error: {}", err)
    };
    let course_end = found_course.period_end;
    
    let borrow_dat = Local::now().date_naive();
    let return_dat = match course_end {
        1 => NaiveDate::from_ymd_opt(2023, 11, 30),
        2 => NaiveDate::from_ymd_opt(2023, 1, 30),
        3 => NaiveDate::from_ymd_opt(2024, 3, 30),
        4 => NaiveDate::from_ymd_opt(2023, 6, 30),
        5 => NaiveDate::from_ymd_opt(2023, 9, 31),
        _ => NaiveDate::from_ymd_opt(2023, 12, 31), //If out of bounds period.
        }.unwrap(); //Get the return date.
    let associated_user_option: Result<User, diesel::result::Error> =  users.filter(uid.eq(token)).first::<User>(connection);
    let associated_user = match associated_user_option {
        Ok(associated_user_result) => associated_user_result,
        Err(err) => return format!("Error: {}", err)
    };
    let associated_user_id = associated_user.user_id;
    
    let result = insert_into(userbooks)
        .values((book_id.eq(book_bid), borrow_date.eq(borrow_dat), return_date.eq(return_dat), user_id.eq(associated_user_id)))
        .execute(connection);
    match result {
        Ok(_) => {
            decrease_stock(isbn_clone);
            format!("Succesfully borrowed book.")
        },
        Err(err) => format!("Error: {}", err),
    }
    //Check what period course is in.
} 

#[get("/userbooks/<token>")] //Function to get all books
async fn get_userbooks(token: String) 
{
    use schema::books::dsl::*;
    use schema::users::dsl::*;
    use schema::userbooks::columns::user_id;
    if !verify_token(token).await {
        return
    }

    use schema::userbooks::dsl::*;
    //AUTHENTICATION

    let connection = &mut create_connection(); //Establish connection
    
    let associated_user_option: Result<User, diesel::result::Error> = users.filter(uid.eq(token)).first::<User>(connection);
    let associated_user = match associated_user_option {
        Ok(associated_user_result) => associated_user_result,
        Err(err) => return format!("{}", err),
    }; //Find associated user    

    let associated_userbooks_option: Result<Vec<Userbook>, diesel::result::Error> = userbooks.filter(user_id.eq(associated_user.user_id)).load::<Userbook>(connection);
    let associated_userbooks = match associated_userbooks_option {
        Ok(associated_userbooks_result) => associated_userbooks_result,
        Err(err) => return format!("{}", err),
    }; //Get the vector with all of the users books.

    let mut frontend_userbooks_string= String::new(); //Create a vector of the frontend userbook.
    
    for single_userbook in associated_userbooks {
        let associated_book_option: Result<Book, diesel::result::Error> = books.filter(bid.eq(single_userbook.book_id)).first::<Book>(connection);
        let associated_book = match associated_book_option {
            Ok(associated_book_result) => associated_book_result,
            Err(err) => return format!("{}", err),
        }; //Get the related book.

        let frontend_userbook = FrontEndUserbook {
            title: associated_book.title,
            isbn13: associated_book.isbn13,
            authors: associated_book.authors,
            borrow_date: single_userbook.borrow_date,
            return_date: single_userbook.return_date,
        }; //Insert all the relevant elements.
        frontend_userbooks_string.push_str(&serde_json::to_string(&frontend_userbook).unwrap()); //Append to vector of userbooks.
    };

    return frontend_userbooks_string

    //For every one of the entries found, get the related book, and fill the frontend userbook with relevant values.
    //Once finished convert the string to json and return it.

} 

#[get("/return/<isbn>/<token>")]
async fn return_book(isbn: String, token: String) -> String { 

    let client = reqwest::Client::new();
	let token_uid: String = match client.get(format!("https://courseLend.akerhielm.nu/auth/whoami/{token}"))
	.send().await {
		Ok(res) => match res.text().await {
			Ok(res) => {
				let sub: serde_json::Value = serde_json::from_str(&res).unwrap();
				if !sub["sub"].is_null() {
					return "Could not authenticate".to_string()
				} else {
                    sub["sub"].as_str().unwrap().to_string()
                }
			},
			Err(_) => return "Could not authenticate".to_string()
		},
		Err(_) => return "Could not authenticate".to_string()
	};


    use schema::books::dsl::*;
    use schema::userbooks::dsl::*;
    use schema::userbooks::columns::user_id;
    use schema::users::dsl::*;
    //Return book

    //Authenticate
    let isbn_clone = isbn.clone(); //Clone because isbn is moved later.
    let connection = &mut create_connection(); //Establish connection
    
    let associated_user_option: Result<User, diesel::result::Error> = users.filter(uid.eq(token_uid)).first::<User>(connection);
    let associated_user = match associated_user_option {
        Ok(associated_user_result) => associated_user_result,
        Err(err) => return format!("Error: {}", err),
    }; //Find associated user
    
    let associated_book_option: Result<Book, diesel::result::Error> = books.filter(isbn13.eq(isbn)).first::<Book>(connection);
    let associated_book = match associated_book_option {
        Ok(associated_book_result) => associated_book_result,
        Err(err) => return format!("Error: {}", err),
    }; //Get the associated book.

    //Get the terms to search userbooks table with.
    let search_book_id = associated_book.bid;
    let search_user_id_int = associated_user.user_id;

    let associated_userbook_option = diesel::delete(userbooks.filter(book_id.eq(search_book_id)).filter(user_id.eq(search_user_id_int))).execute(connection);
    match associated_userbook_option {
        Ok(_) => {
            increase_stock(isbn_clone, 1);
            return format!("Succesfully returned book.")
    },
    Err(err) => return format!("{}", err),
    };
    //Filter via book and user id
    //Remove entry from usertable, 
    //increase stock of the book by 1.


}

#[get("/buy/<isbn>/<token>")]
async fn sell_book(isbn: String, token: String) -> String {

    if !verify_token(token).await {
        return
    }
    //User authentication
    increase_stock(isbn, 1) //Increase the stock by one.
}

#[get("/books/next-period/<token>")] //Function that returns the books the user will need in the next period.
async fn get_next_period_books(token: String) -> String
{

    let client = reqwest::Client::new();
	let token_uid: String = match client.get(format!("https://courseLend.akerhielm.nu/auth/whoami/{token}"))
	.send().await {
		Ok(res) => match res.text().await {
			Ok(res) => {
				let sub: serde_json::Value = serde_json::from_str(&res).unwrap();
				if !sub["sub"].is_null() {
					return "Could not authenticate".to_string()
				} else {
                    sub["sub"].as_str().unwrap().to_string()
                }
			},
			Err(_) => return "Could not authenticate".to_string()
		},
		Err(_) => return "Could not authenticate".to_string()
	};


    use schema::users::dsl::*;
    use schema::books::dsl::*;
    use schema::coursebooks::dsl::*;
    use schema::courses::dsl::*;

    let connection = &mut create_connection(); //Establish connection
    
    let related_user_option: Result<User, diesel::result::Error> = users.filter(uid.eq(token)).first::<User>(connection); //Get the related user.
    let related_user = match related_user_option {
        Ok(related_user_result) => related_user_result,
        Err(err) => return format!("Error: {}", err)
    };

    let today = Local::now().date_naive();
    let user_period = match today {
        d if today >= NaiveDate::from_ymd_opt(2023, 3, 27).unwrap() && today <= NaiveDate::from_ymd_opt(2023, 6, 4).unwrap() => 4,
        d if today >= NaiveDate::from_ymd_opt(2023, 8, 28).unwrap() && today <= NaiveDate::from_ymd_opt(2023, 11, 5).unwrap() => 1,
        d if today >= NaiveDate::from_ymd_opt(2023, 11, 6).unwrap() && today <= NaiveDate::from_ymd_opt(2024, 1, 14).unwrap() => 2,
        d if today >= NaiveDate::from_ymd_opt(2024, 1, 15).unwrap() && today <= NaiveDate::from_ymd_opt(2024, 3, 24).unwrap() => 3,
        _ => return format!("Date outside of known periods")
    }; //Get the user period.

    let user_current_term = related_user.term;
    let next_period_courses_result = match user_period {
        2 => courses.filter(period_start.eq(3)).filter(term_start.eq(user_current_term + 1)).load::<Course>(connection), //If user is currently in period 2, show next term courses and period 3.
        4 => courses.filter(period_start.eq(1)).filter(term_start.eq(user_current_term + 1)).load::<Course>(connection), //If user is currently in period 4, show courses from beginning of next year.
        _ => courses.filter(period_start.eq(user_period + 1)).filter(term_start.eq(user_current_term)).load::<Course>(connection) //If user is period 1 or 3, show next period courses but same term.
    }; //Get all the courses for the user in the next period (vector).
    let next_period_courses = match next_period_courses_result {
        Ok(courses_result) => courses_result,
        Err(err) => return format!("Error: {}", err),
    };
    let mut next_period_books = String::new(); //String with next period books.

    for course in next_period_courses
    {
        //ERROR -- PROBLEM IS WHAT TO DO IF NO RESULT
        let coursebook_result = coursebooks.filter(coursebook_course_id.eq(course.cid)).first::<Coursebook>(connection);         //Find the related coursebook entry and get book id.
        let coursebook: Coursebook = match coursebook_result {
            Ok(coursebook_return) => coursebook_return, //If find
            Err(_) => continue, //Need to skip this somehow.
        };
        let next_period_book: Book = books.filter(bid.eq(coursebook.coursebook_book_id)).first::<Book>(connection).unwrap(); //Find the related book.
        next_period_books.push_str(&serde_json::to_string(&next_period_book).unwrap()); //Convert to string and push. 

    }; 
    next_period_books
} 


#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![get_books])
        .mount("/", routes![get_book_isbn])
        .mount("/", routes![get_book_bid])
        .mount("/", routes![get_related_course_by_id])
        .mount("/", routes![increase_stock])
        .mount("/", routes![decrease_stock])
        .mount("/", routes![borrow_book])
        .mount("/", routes![return_book])
        .mount("/", routes![sell_book])
        .mount("/", routes![get_next_period_books])

}

async fn verify_token(token: String) -> bool {
    let client = reqwest::Client::new();
    match client.get(format!("https://courselend.akerhielm.nu/auth/verify/{token}")).send().await {
        Ok(res) => {
            match res.text().await {
                Ok(text) => {
                    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
                    v["response"].eq("Invalid token")
                },
                Err(_) => false
            }
        },
        Err(_) => false
    }
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