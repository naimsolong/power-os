-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

CREATE TABLE employee (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    party_id UUID NOT NULL REFERENCES party(id) ON DELETE CASCADE,
    employee_code TEXT NOT NULL,
    job_title TEXT,
    department TEXT,
    hire_date DATE,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, employee_code)
);

CREATE INDEX idx_employee_workspace ON employee(workspace_id);
CREATE INDEX idx_employee_party ON employee(party_id);
CREATE INDEX idx_employee_status ON employee(status);
