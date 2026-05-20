ALTER TABLE qr.qr_codes
    ADD COLUMN image_format VARCHAR(10) NOT NULL DEFAULT 'png';
