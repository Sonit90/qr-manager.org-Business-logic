CREATE TABLE qr.contact_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(320) NOT NULL,
    message TEXT NOT NULL,
    ip VARCHAR(45),
    user_agent TEXT,
    captcha_token TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contact_messages_created_at ON qr.contact_messages(created_at DESC);
CREATE INDEX idx_contact_messages_email ON qr.contact_messages(email);
