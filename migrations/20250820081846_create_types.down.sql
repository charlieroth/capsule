-- Add down migration script here
DROP TYPE IF EXISTS job_status;
DROP TYPE IF EXISTS job_kind;
DROP TYPE IF EXISTS item_status;
DROP EXTENSION IF EXISTS pgcrypto;