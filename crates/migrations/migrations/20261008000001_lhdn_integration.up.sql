-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

ALTER TABLE workspace
    ADD COLUMN lhdn_client_id TEXT,
    ADD COLUMN lhdn_client_secret TEXT,
    ADD COLUMN lhdn_tin TEXT,
    ADD COLUMN lhdn_sandbox BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN lhdn_base_url TEXT;

UPDATE workspace
SET lhdn_base_url = CASE
    WHEN lhdn_sandbox THEN 'https://preprod-sdk.myinvois.hasil.gov.my'
    ELSE 'https://sdk.myinvois.hasil.gov.my'
END
WHERE lhdn_base_url IS NULL;

ALTER TABLE party
    ADD COLUMN tin TEXT;

ALTER TABLE invoice
    ADD COLUMN lhdn_status TEXT,
    ADD COLUMN lhdn_uuid TEXT,
    ADD COLUMN lhdn_error TEXT;

CREATE TABLE e_invoice_submission (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    invoice_id UUID NOT NULL REFERENCES invoice(id) ON DELETE CASCADE,
    lhdn_uuid TEXT,
    lhdn_submission_uid TEXT,
    status TEXT,
    request_json JSONB,
    response_json JSONB,
    error_message TEXT,
    submitted_at TIMESTAMPTZ,
    polled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_e_invoice_submission_workspace_invoice ON e_invoice_submission(workspace_id, invoice_id);
