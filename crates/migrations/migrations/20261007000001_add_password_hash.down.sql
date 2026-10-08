-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

ALTER TABLE "user" DROP COLUMN IF EXISTS email_verified_at;
ALTER TABLE "user" DROP COLUMN IF EXISTS password_hash;
