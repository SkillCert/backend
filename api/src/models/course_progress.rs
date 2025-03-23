use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::{Queryable, Insertable};
use crate::schema::course_progress;

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct CourseProgress {
    pub id: i64,
    pub user_id: i64,
    pub course_id: i64,
    pub progress: i16,      
    pub completed: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "course_progress"]
pub struct NewCourseProgress {
    pub user_id: i64,
    pub course_id: i64,
    pub progress: i16,    
    pub completed: bool,    
}
