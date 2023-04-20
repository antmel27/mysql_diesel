
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
    }
}

diesel::table! {
    coursebooks (bid, cid) {
        bid -> Integer,
        cid -> Integer,
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
        encrypted_user_id -> Varchar,
        borrow_date -> Date,
        return_date -> Date,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Integer,
        encrypted_user_id -> Varchar,
        salted_user_id -> Varchar,
        subscription_status -> Bool,
        term -> Integer,
        year -> Integer,
    }
}

diesel::joinable!(book_tags -> books (book_id));
diesel::joinable!(book_tags -> tags (tag_id));
diesel::joinable!(coursebooks -> books (bid));
diesel::joinable!(coursebooks -> courses (cid));
diesel::joinable!(userbooks -> books (book_id));

diesel::allow_tables_to_appear_in_same_query!(
    book_tags,
    books,
    coursebooks,
    courses,
    tags,
    userbooks,
    users,
);
