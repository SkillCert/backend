use diesel::prelude::*;
use diesel::dsl::now;
use crate::models::marketplace_transaction::{MarketplaceTransaction, NewMarketplaceTransaction};
use crate::schema::marketplace_transactions::dsl::*;

/// Inserts a new course purchase into the marketplace_transactions table.
/// This function sets the initial status to "Pending".
pub fn process_course_purchase(
    conn: &PgConnection,
    buyer: i64,
    course: i64,
    amount_value: i64,
    curr: &str,
    tx_hash: &str,
) -> Result<MarketplaceTransaction, diesel::result::Error> {
    // Create the new transaction struct
    let new_tx = NewMarketplaceTransaction {
        buyer_id: buyer,
        course_id: course,
        amount: amount_value,
        currency: curr.to_string(),
        transaction_hash: tx_hash.to_string(),
        status: "Pending".to_string(),
    };

    // Insert and return the newly created transaction
    diesel::insert_into(marketplace_transactions)
        .values(&new_tx)
        .get_result(conn)
}

/// Updates the status of an existing transaction.
/// This can be used after validating payment on Stellar.
pub fn update_transaction_status(
    conn: &PgConnection,
    tx_id: i64,
    new_status: &str,
) -> Result<MarketplaceTransaction, diesel::result::Error> {
    diesel::update(marketplace_transactions.find(tx_id))
        .set((
            status.eq(new_status),
            updated_at.eq(now),
        ))
        .get_result(conn)
}

/// Retrieves the transaction status for a given transaction ID.
pub fn get_transaction_status(
    conn: &PgConnection,
    tx_id: i64,
) -> Result<String, diesel::result::Error> {
    marketplace_transactions
        .find(tx_id)
        .select(status)
        .first(conn)
}

/// Lists all transactions for a given user (buyer).
pub fn list_transactions_for_user(
    conn: &PgConnection,
    user: i64,
) -> Result<Vec<MarketplaceTransaction>, diesel::result::Error> {
    marketplace_transactions
        .filter(buyer_id.eq(user))
        .load::<MarketplaceTransaction>(conn)
}

/// Issues a refund for a transaction (Admin Only).
/// This function updates the transaction status to "Refunded".
pub fn issue_refund(
    conn: &PgConnection,
    tx_id: i64,
) -> Result<MarketplaceTransaction, diesel::result::Error> {
    diesel::update(marketplace_transactions.find(tx_id))
        .set((
            status.eq("Refunded"),
            updated_at.eq(now),
        ))
        .get_result(conn)
}

/// Deletes a transaction record (Admin Only).
pub fn delete_transaction(
    conn: &PgConnection,
    tx_id: i64,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(marketplace_transactions.find(tx_id))
        .execute(conn)
}
