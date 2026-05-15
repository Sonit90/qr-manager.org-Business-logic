CREATE TABLE target_urls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    qr_code_id UUID NOT NULL REFERENCES qr_codes(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_target_urls_qr_code_id ON target_urls(qr_code_id);
