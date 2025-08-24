-- Update job_status enum to include 'succeeded' instead of 'done'  
ALTER TYPE job_status RENAME VALUE 'done' TO 'succeeded';

-- Add new columns to existing jobs table
ALTER TABLE jobs ADD COLUMN payload JSONB DEFAULT '{}';
ALTER TABLE jobs ADD COLUMN max_attempts INT DEFAULT 25;
ALTER TABLE jobs ADD COLUMN backoff_seconds INT DEFAULT 0;
ALTER TABLE jobs ADD COLUMN visibility_till TIMESTAMPTZ;
ALTER TABLE jobs ADD COLUMN reserved_by UUID;
ALTER TABLE jobs ADD COLUMN created_at TIMESTAMPTZ DEFAULT now();
ALTER TABLE jobs ADD COLUMN updated_at TIMESTAMPTZ DEFAULT now();

-- Set NOT NULL constraints after adding defaults
ALTER TABLE jobs ALTER COLUMN payload SET NOT NULL;
ALTER TABLE jobs ALTER COLUMN max_attempts SET NOT NULL;
ALTER TABLE jobs ALTER COLUMN backoff_seconds SET NOT NULL;
ALTER TABLE jobs ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE jobs ALTER COLUMN updated_at SET NOT NULL;

-- Change kind from enum to text
ALTER TABLE jobs ALTER COLUMN kind TYPE TEXT;

-- Drop the old job_kind enum (no longer needed)
DROP TYPE job_kind;

-- Add new indexes for efficient job polling
CREATE INDEX jobs_visibility_till_idx ON jobs (visibility_till);
CREATE INDEX jobs_reserved_by_idx ON jobs (reserved_by);

-- Add trigger for updated_at
CREATE TRIGGER trg_jobs_updated_at
BEFORE UPDATE ON jobs
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
