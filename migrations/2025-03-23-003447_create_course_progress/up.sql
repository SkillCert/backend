CREATE TABLE course_progress (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    course_id BIGINT NOT NULL REFERENCES courses(id),
    progress SMALLINT NOT NULL DEFAULT 0, -- Percentage 0-100
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_user_course UNIQUE (user_id, course_id)
);
