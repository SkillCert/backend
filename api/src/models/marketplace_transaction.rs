use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::{Queryable, Insertable};
use crate::schema::marketplace_transactions;

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct MarketplaceTransaction {
    pub id: i64,
    pub buyer_id: i64,
    pub course_id: i64,
    pub amount: i64,
    pub currency: String,
    pub transaction_hash: String,
    pub status: String, // "Pending", "Completed", "Failed", "Refunded"
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "marketplace_transactions"]
pub struct NewMarketplaceTransaction {
    pub buyer_id: i64,
    pub course_id: i64,
    pub amount: i64,
    pub currency: String,
    pub transaction_hash: String,
    pub status: String, /
}
