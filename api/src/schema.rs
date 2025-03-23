// @generated automatically by Diesel CLI.

diesel::table! {
    course_progress (id) {
        id -> Int8,
        user_id -> Int8,
        course_id -> Int8,
        progress -> Int2,
        completed -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
