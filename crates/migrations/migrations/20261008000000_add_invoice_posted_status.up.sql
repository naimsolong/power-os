-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

ALTER TABLE invoice DROP CONSTRAINT IF EXISTS invoice_status_check;
ALTER TABLE invoice ADD CONSTRAINT invoice_status_check CHECK (status IN ('draft', 'sent', 'paid', 'overdue', 'cancelled', 'posted'));
