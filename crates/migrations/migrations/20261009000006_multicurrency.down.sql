-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

ALTER TABLE expense_line DROP COLUMN IF EXISTS foreign_amount;
ALTER TABLE expense DROP COLUMN IF EXISTS exchange_rate;
ALTER TABLE payment DROP COLUMN IF EXISTS foreign_amount;
ALTER TABLE payment DROP COLUMN IF EXISTS exchange_rate;
ALTER TABLE payment_allocation DROP COLUMN IF EXISTS foreign_amount;
ALTER TABLE bill_line DROP COLUMN IF EXISTS foreign_amount;
ALTER TABLE bill_line DROP COLUMN IF EXISTS foreign_unit_price;
ALTER TABLE bill DROP COLUMN IF EXISTS exchange_rate;
ALTER TABLE invoice_line DROP COLUMN IF EXISTS foreign_amount;
ALTER TABLE invoice_line DROP COLUMN IF EXISTS foreign_unit_price;
ALTER TABLE invoice DROP COLUMN IF EXISTS exchange_rate;
ALTER TABLE journal_line DROP COLUMN IF EXISTS currency;
ALTER TABLE journal_line DROP COLUMN IF EXISTS exchange_rate;
ALTER TABLE journal_line DROP COLUMN IF EXISTS foreign_credit;
ALTER TABLE journal_line DROP COLUMN IF EXISTS foreign_debit;
