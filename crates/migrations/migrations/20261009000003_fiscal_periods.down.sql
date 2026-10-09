-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

DROP INDEX IF EXISTS idx_accounting_period_closed_at;
DROP INDEX IF EXISTS idx_accounting_period_fiscal_year;
DROP INDEX IF EXISTS idx_accounting_period_workspace;
DROP TABLE IF EXISTS accounting_period;

DROP INDEX IF EXISTS idx_fiscal_year_workspace;
DROP TABLE IF EXISTS fiscal_year;
