CREATE TABLE tax_code (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    description TEXT NOT NULL,
    rate NUMERIC(5,2) NOT NULL DEFAULT 0,
    tax_type TEXT NOT NULL CHECK (tax_type IN ('sst', 'service_tax')),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, code)
);

CREATE INDEX idx_tax_code_workspace_id ON tax_code(workspace_id);
CREATE INDEX idx_tax_code_workspace_active ON tax_code(workspace_id, is_active);
