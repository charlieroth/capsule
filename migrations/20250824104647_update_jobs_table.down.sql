-- Revert trigger
DROP TRIGGER IF EXISTS trg_jobs_updated_at ON jobs;

-- Revert indexes
DROP INDEX IF EXISTS jobs_visibility_till_idx;
DROP INDEX IF EXISTS jobs_reserved_by_idx;

-- Recreate job_kind enum
CREATE TYPE job_kind AS ENUM ('fetch_and_extract', 'reindex_item', 'delete_item');

-- Remove new columns
ALTER TABLE jobs DROP COLUMN payload;
ALTER TABLE jobs DROP COLUMN max_attempts;
ALTER TABLE jobs DROP COLUMN backoff_seconds; 
ALTER TABLE jobs DROP COLUMN visibility_till;
ALTER TABLE jobs DROP COLUMN reserved_by;
ALTER TABLE jobs DROP COLUMN created_at;
ALTER TABLE jobs DROP COLUMN updated_at;

-- Change kind back to enum
ALTER TABLE jobs ALTER COLUMN kind TYPE job_kind USING kind::job_kind;

-- Revert job_status enum change
ALTER TYPE job_status RENAME VALUE 'succeeded' TO 'done';
