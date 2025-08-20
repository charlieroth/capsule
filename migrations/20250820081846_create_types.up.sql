-- Add up migration script here
-- UUID generation
CREATE EXTENSION IF NOT EXISTS pgcrypto;
-- item status
CREATE TYPE item_status AS ENUM ('pending', 'fetched', 'archived');
-- job kind
CREATE TYPE job_kind AS ENUM ('fetch_and_extract', 'reindex_item', 'delete_item');
-- job status
CREATE TYPE job_status AS ENUM ('queued', 'running', 'done', 'failed');
