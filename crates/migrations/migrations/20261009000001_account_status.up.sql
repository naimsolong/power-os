-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

ALTER TABLE account
  ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT true,
  ADD COLUMN archived_at TIMESTAMPTZ;
