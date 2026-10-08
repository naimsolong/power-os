-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

DROP INDEX IF EXISTS idx_password_reset_token_token;
DROP INDEX IF EXISTS idx_password_reset_token_user;
DROP TABLE IF EXISTS password_reset_token;
