CREATE TABLE marketplace_transactions (
    id BIGSERIAL PRIMARY KEY,
    buyer_id BIGINT NOT NULL REFERENCES users(id),
    course_id BIGINT NOT NULL REFERENCES courses(id),
    amount BIGINT NOT NULL,
    currency VARCHAR(10) NOT NULL,
    transaction_hash VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'Pending',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Optionally, you can add a unique constraint if needed:
-- CREATE UNIQUE INDEX unique_purchase ON marketplace_transactions(buyer_id, course_id)
-- WHERE status != 'Refunded';
