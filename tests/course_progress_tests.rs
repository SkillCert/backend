use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

use crate::models::course_progress::{CourseProgress, NewCourseProgress};
use crate::schema::course_progress::dsl::*;

fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

#[test]
fn test_create_progress_record() {
    let conn = establish_connection();

    let new_progress = NewCourseProgress {
        user_id: 1,
        course_id: 1,
        progress: 0,
        completed: false,
    };

    let result = diesel::insert_into(course_progress)
        .values(&new_progress)
        .get_result::<CourseProgress>(&conn);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().progress, 0);
}

#[test]
fn test_update_progress() {
    let conn = establish_connection();

    let target_user_id = 1;
    let target_course_id = 1;

    let result = diesel::update(course_progress.filter(user_id.eq(target_user_id).and(course_id.eq(target_course_id))))
        .set(progress.eq(50))
        .execute(&conn);

    assert_eq!(result.unwrap(), 1);

    let updated_progress = course_progress
        .filter(user_id.eq(target_user_id).and(course_id.eq(target_course_id)))
        .first::<CourseProgress>(&conn)
        .unwrap();

    assert_eq!(updated_progress.progress, 50);
}

#[test]
fn test_retrieve_progress() {
    let conn = establish_connection();

    let result = course_progress
        .filter(user_id.eq(1).and(course_id.eq(1)))
        .first::<CourseProgress>(&conn);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().progress, 50);
}
