-- Add down migration script here
-- drop in FK-safe order
DROP INDEX IF EXISTS idx_jobs_ready;
DROP INDEX IF EXISTS idx_jobs_status_run_at;
DROP TABLE IF EXISTS jobs;

DROP INDEX IF EXISTS idx_item_tags_tag_id;
DROP TABLE IF EXISTS item_tags;

DROP INDEX IF EXISTS idx_tags_user_id;
DROP TABLE IF EXISTS tags;

DROP TABLE IF EXISTS contents;

DROP TRIGGER IF EXISTS trg_items_updated_at ON items;
DROP INDEX IF EXISTS idx_items_created_at;
DROP INDEX IF EXISTS idx_items_status;
DROP INDEX IF EXISTS idx_items_user_id;
-- DROP INDEX IF EXISTS uq_items_user_url;
DROP TABLE IF EXISTS items;

DROP TABLE IF EXISTS users;

DROP FUNCTION IF EXISTS set_updated_at();
