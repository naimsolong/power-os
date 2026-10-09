-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

DROP INDEX IF EXISTS idx_invoice_line_tax_code_id;

ALTER TABLE invoice_line
  DROP COLUMN IF EXISTS tax_amount,
  DROP COLUMN IF EXISTS tax_code_id;
