-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

CREATE TABLE bill (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    party_id UUID NOT NULL REFERENCES party(id) ON DELETE CASCADE,
    bill_number TEXT NOT NULL,
    issue_date DATE NOT NULL,
    due_date DATE,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'open', 'partial', 'paid', 'overdue', 'cancelled')),
    total_amount NUMERIC(19, 4) NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'MYR',
    journal_entry_id UUID REFERENCES journal_entry(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, bill_number)
);

CREATE TABLE bill_line (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bill_id UUID NOT NULL REFERENCES bill(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    quantity NUMERIC(19, 4) NOT NULL DEFAULT 1,
    unit_price NUMERIC(19, 4) NOT NULL DEFAULT 0,
    amount NUMERIC(19, 4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE expense (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    party_id UUID REFERENCES party(id) ON DELETE SET NULL,
    expense_date DATE NOT NULL,
    description TEXT NOT NULL,
    reference TEXT,
    total_amount NUMERIC(19, 4) NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'MYR',
    payment_method TEXT,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'posted', 'cancelled')),
    journal_entry_id UUID REFERENCES journal_entry(id) ON DELETE SET NULL,
    paid_from_account_id UUID REFERENCES account(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE expense_line (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    expense_id UUID NOT NULL REFERENCES expense(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    amount NUMERIC(19, 4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
