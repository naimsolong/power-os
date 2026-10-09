-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

-- journal_line
ALTER TABLE journal_line
  ADD COLUMN foreign_debit NUMERIC(19,4) NOT NULL DEFAULT 0,
  ADD COLUMN foreign_credit NUMERIC(19,4) NOT NULL DEFAULT 0,
  ADD COLUMN exchange_rate NUMERIC(19,6) NOT NULL DEFAULT 1,
  ADD COLUMN currency VARCHAR(3) NOT NULL DEFAULT 'MYR';

-- Existing journal lines were MYR at 1:1, so mirror the functional amounts.
UPDATE journal_line SET foreign_debit = debit, foreign_credit = credit WHERE exchange_rate = 1;

-- invoice
ALTER TABLE invoice
  ADD COLUMN exchange_rate NUMERIC(19,6) NOT NULL DEFAULT 1;

-- invoice_line
ALTER TABLE invoice_line
  ADD COLUMN foreign_unit_price NUMERIC(19,4) NOT NULL DEFAULT 0,
  ADD COLUMN foreign_amount NUMERIC(19,4) NOT NULL DEFAULT 0;

UPDATE invoice_line SET foreign_unit_price = unit_price, foreign_amount = line_total;

-- bill
ALTER TABLE bill
  ADD COLUMN exchange_rate NUMERIC(19,6) NOT NULL DEFAULT 1;

-- bill_line
ALTER TABLE bill_line
  ADD COLUMN foreign_unit_price NUMERIC(19,4) NOT NULL DEFAULT 0,
  ADD COLUMN foreign_amount NUMERIC(19,4) NOT NULL DEFAULT 0;

UPDATE bill_line SET foreign_unit_price = unit_price, foreign_amount = amount;

-- payment
ALTER TABLE payment
  ADD COLUMN exchange_rate NUMERIC(19,6) NOT NULL DEFAULT 1,
  ADD COLUMN foreign_amount NUMERIC(19,4) NOT NULL DEFAULT 0;

UPDATE payment SET foreign_amount = amount;

ALTER TABLE payment_allocation
  ADD COLUMN foreign_amount NUMERIC(19,4) NOT NULL DEFAULT 0;

-- expense
ALTER TABLE expense
  ADD COLUMN exchange_rate NUMERIC(19,6) NOT NULL DEFAULT 1;

-- expense_line
ALTER TABLE expense_line
  ADD COLUMN foreign_amount NUMERIC(19,4) NOT NULL DEFAULT 0;

UPDATE expense_line SET foreign_amount = amount;
