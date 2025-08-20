-- Add up migration script here
-- updated_at trigger helper
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END $$;

-- users
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email TEXT NOT NULL UNIQUE,
  pw_hash TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- items
CREATE TABLE items (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  url TEXT NOT NULL,
  title TEXT,
  site TEXT,
  status item_status NOT NULL DEFAULT 'pending',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- auto-maintain updated_at
CREATE TRIGGER trg_items_updated_at
BEFORE UPDATE ON items
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- contents (1:1 with items; PK = item_id)
CREATE TABLE contents (
  item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
  html TEXT,
  text TEXT,
  lang VARCHAR(16),
  extracted_at TIMESTAMPTZ,
  checksum TEXT
);

-- tags
CREATE TABLE tags (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  CONSTRAINT uq_tags_user_name UNIQUE (user_id, name)
);

-- item_tag join
CREATE TABLE item_tags (
  item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  tag_id  UUID NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
  PRIMARY KEY (item_id, tag_id)
);

-- jobs
CREATE TABLE jobs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  kind job_kind NOT NULL,
  item_id UUID REFERENCES items(id) ON DELETE SET NULL,
  status job_status NOT NULL DEFAULT 'queued',
  run_at TIMESTAMPTZ NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
  last_error TEXT
);

-- ---------- indexes ----------
-- items: common filters
CREATE INDEX idx_items_user_id        ON items(user_id);
CREATE INDEX idx_items_status         ON items(status);
CREATE INDEX idx_items_created_at     ON items(created_at DESC);
-- optional: avoid duplicate URLs per user (comment out if you want duplicates)
-- CREATE UNIQUE INDEX uq_items_user_url ON items(user_id, url);

-- contents: fast text lookups by item_id already covered by PK

-- tags: lookups by user
CREATE INDEX idx_tags_user_id ON tags(user_id);

-- item_tags: reverse lookups
CREATE INDEX idx_item_tags_tag_id ON item_tags(tag_id);

-- jobs: queue polling
CREATE INDEX idx_jobs_status_run_at ON jobs(status, run_at);
CREATE INDEX idx_jobs_ready ON jobs(run_at) WHERE status = 'queued';