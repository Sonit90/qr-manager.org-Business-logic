-- NOTE: requires auth.users table created by rust-generic-user-backend
-- migrations FIRST. Auth service owns the `auth` schema.
-- Deploy auth service migrations before qr-api migrations.
CREATE TABLE qr.qr_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    code VARCHAR(255) NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_qr_codes_code ON qr.qr_codes(code);
CREATE INDEX idx_qr_codes_user_id ON qr.qr_codes(user_id);
