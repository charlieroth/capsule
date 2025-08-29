-- Add up migration script here
-- Extend contents table to support cleaned content persistence

-- Add columns for cleaned versions while preserving original data
ALTER TABLE contents RENAME COLUMN html TO raw_html;
ALTER TABLE contents RENAME COLUMN text TO raw_text;

ALTER TABLE contents ADD COLUMN clean_html TEXT;
ALTER TABLE contents ADD COLUMN clean_text TEXT;

-- Create composite unique index to prevent duplicate writes when content hasn't changed
-- Note: checksum column already exists from original schema
CREATE UNIQUE INDEX contents_item_id_checksum_uq ON contents(item_id, checksum) WHERE checksum IS NOT NULL;

-- Optional: Add GIN index on clean_text for future full-text search capabilities
CREATE INDEX contents_clean_text_gin ON contents USING GIN (to_tsvector('simple', clean_text)) WHERE clean_text IS NOT NULL;
