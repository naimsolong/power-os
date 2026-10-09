-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

CREATE TABLE fiscal_year (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    is_closed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_fiscal_year_workspace ON fiscal_year(workspace_id);

CREATE TABLE accounting_period (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    fiscal_year_id UUID NOT NULL REFERENCES fiscal_year(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    is_closed BOOLEAN NOT NULL DEFAULT false,
    closed_at TIMESTAMPTZ,
    closed_by_user_id UUID REFERENCES "user"(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_accounting_period_workspace ON accounting_period(workspace_id);
CREATE INDEX idx_accounting_period_fiscal_year ON accounting_period(fiscal_year_id);
CREATE INDEX idx_accounting_period_closed_at ON accounting_period(workspace_id, start_date);
