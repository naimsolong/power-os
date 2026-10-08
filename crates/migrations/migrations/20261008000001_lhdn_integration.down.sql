-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

DROP INDEX IF EXISTS idx_e_invoice_submission_workspace_invoice;
DROP TABLE IF EXISTS e_invoice_submission;

ALTER TABLE party DROP COLUMN IF EXISTS tin;

ALTER TABLE invoice
    DROP COLUMN IF EXISTS lhdn_status,
    DROP COLUMN IF EXISTS lhdn_uuid,
    DROP COLUMN IF EXISTS lhdn_error;

ALTER TABLE workspace
    DROP COLUMN IF EXISTS lhdn_client_id,
    DROP COLUMN IF EXISTS lhdn_client_secret,
    DROP COLUMN IF EXISTS lhdn_tin,
    DROP COLUMN IF EXISTS lhdn_sandbox,
    DROP COLUMN IF EXISTS lhdn_base_url;
