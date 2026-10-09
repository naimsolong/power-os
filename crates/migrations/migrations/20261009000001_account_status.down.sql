-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

ALTER TABLE account
  DROP COLUMN is_active,
  DROP COLUMN archived_at;
