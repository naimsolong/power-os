-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

CREATE TABLE payment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    party_id UUID REFERENCES party(id) ON DELETE SET NULL,
    bank_account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    payment_date DATE NOT NULL,
    amount NUMERIC(19,4) NOT NULL CHECK (amount > 0),
    currency TEXT NOT NULL DEFAULT 'MYR',
    payment_method TEXT NOT NULL,
    reference TEXT,
    notes TEXT,
    direction TEXT NOT NULL CHECK (direction IN ('received', 'sent')),
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'posted', 'cancelled')),
    journal_entry_id UUID REFERENCES journal_entry(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE payment_allocation (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id UUID NOT NULL REFERENCES payment(id) ON DELETE CASCADE,
    invoice_id UUID REFERENCES invoice(id) ON DELETE CASCADE,
    bill_id UUID REFERENCES bill(id) ON DELETE CASCADE,
    amount NUMERIC(19,4) NOT NULL CHECK (amount > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        (invoice_id IS NOT NULL AND bill_id IS NULL) OR
        (invoice_id IS NULL AND bill_id IS NOT NULL)
    )
);
