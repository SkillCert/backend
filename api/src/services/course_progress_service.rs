use diesel::prelude::*;
use crate::models::course_progress::{CourseProgress, NewCourseProgress};
use crate::schema::course_progress::dsl::*;

pub fn create_course_progress(
    conn: &PgConnection,
    user: i64,
    course: i64,
) -> Result<CourseProgress, diesel::result::Error> {
    let new_progress = NewCourseProgress {
        user_id: user,
        course_id: course,
        progress: 0,
        completed: false,
    };

    diesel::insert_into(course_progress)
        .values(&new_progress)
        .get_result(conn)
}
