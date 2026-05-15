CREATE TABLE qr.scan_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    qr_code_id UUID NOT NULL REFERENCES qr.qr_codes(id) ON DELETE CASCADE,
    ip VARCHAR(45),
    user_agent TEXT,
    browser VARCHAR(100),
    device_type VARCHAR(50),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_scan_logs_qr_code_id ON qr.scan_logs(qr_code_id);
CREATE INDEX idx_scan_logs_timestamp ON qr.scan_logs(timestamp);
