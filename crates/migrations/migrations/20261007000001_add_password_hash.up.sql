-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

ALTER TABLE "user" ADD COLUMN IF NOT EXISTS password_hash TEXT NOT NULL DEFAULT '';
ALTER TABLE "user" ADD COLUMN IF NOT EXISTS email_verified_at TIMESTAMPTZ;
