-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

ALTER TABLE invoice_line
  ADD COLUMN tax_code_id UUID REFERENCES tax_code(id) ON DELETE SET NULL,
  ADD COLUMN tax_amount NUMERIC(19,4) NOT NULL DEFAULT 0;

CREATE INDEX idx_invoice_line_tax_code_id ON invoice_line(tax_code_id);
