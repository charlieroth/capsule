-- Add down migration script here
-- Reverse the contents table extension

-- Drop the indexes we created
DROP INDEX IF EXISTS contents_clean_text_gin;
DROP INDEX IF EXISTS contents_item_id_checksum_uq;

-- Remove the new columns
ALTER TABLE contents
  DROP COLUMN clean_html,
  DROP COLUMN clean_text;

-- Restore original column names
ALTER TABLE contents
  RENAME COLUMN raw_html TO html,
  RENAME COLUMN raw_text TO text;
