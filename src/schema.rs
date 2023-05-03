// @generated automatically by Diesel CLI.

diesel::table! {
    book_tags (book_id, tag_id) {
        book_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::table! {
    books (bid) {
        bid -> Integer,
        isbn13 -> Nullable<Varchar>,
        title -> Varchar,
        authors -> Varchar,
        edition -> Nullable<Varchar>,
        stock -> Nullable<Integer>,
        description -> Nullable<Text>,
        price -> Nullable<Integer>,
    }
}

diesel::table! {
    coursebooks (coursebook_book_id, coursebook_course_id) {
        coursebook_book_id -> Integer,
        coursebook_course_id -> Integer,
    }
}

diesel::table! {
    courses (cid) {
        cid -> Integer,
        course_name -> Varchar,
        period_start -> Integer,
        period_end -> Integer,
        term_start -> Integer,
        term_end -> Integer,
    }
}

diesel::table! {
    tags (tid) {
        tid -> Integer,
        tag -> Varchar,
    }
}

diesel::table! {
    userbooks (borrow_id) {
        borrow_id -> Integer,
        book_id -> Integer,
        borrow_date -> Date,
        return_date -> Date,
        user_id -> Integer,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Integer,
        subscription_status -> Bool,
        term -> Integer,
        year -> Integer,
        uid -> Varchar,
    }
}

diesel::joinable!(book_tags -> books (book_id));
diesel::joinable!(book_tags -> tags (tag_id));
diesel::joinable!(coursebooks -> books (coursebook_book_id));
diesel::joinable!(coursebooks -> courses (coursebook_course_id));
diesel::joinable!(userbooks -> books (book_id));
diesel::joinable!(userbooks -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    book_tags,
    books,
    coursebooks,
    courses,
    tags,
    userbooks,
    users,
);
