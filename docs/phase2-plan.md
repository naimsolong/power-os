# Phase 2 — Full Finance Surface: Detailed Implementation Plan

> Target exit gate: a user can close a month and produce a P&L, Balance Sheet, and Trial Balance.

This document breaks Phase 2 into eight implementation waves. Each wave is sized to be merged independently and includes the data model, API, frontend, business rules, and acceptance criteria.

---

## Cross-cutting principles

1. **The general ledger is the source of truth.** Every commercial transaction (invoice, bill, payment, expense) must create or update journal entries. UI totals and reports read from `journal_line` wherever possible.
2. **Workspace-scoped.** All finance tables include `workspace_id` and respect tenant isolation.
3. **BigDecimal everywhere.** Money is `NUMERIC(19,4)` in Postgres and `BigDecimal` in Rust.
4. **Functional currency is MYR.** Multi-currency support is added in Wave 5; until then every amount is treated as MYR.
5. **No deletes on posted transactions.** Posted journal entries, invoices, and bills can be cancelled or reversed, not hard-deleted.
6. **Period-aware from Wave 8 onward.** After periods are introduced, transactions cannot be posted into a locked period.

---

## Shared data model additions

The following columns and tables are added incrementally across the waves.

### `account` enhancements

| Column | Type | Default | Purpose |
|--------|------|---------|---------|
| `parent_account_id` | UUID → `account(id)` | NULL | Already exists; enables hierarchical COA. |
| `is_active` | BOOLEAN | `true` | Soft-disable an account without losing history. |
| `archived_at` | TIMESTAMPTZ | NULL | Marks an archived account. |
| `currency` | TEXT | `'MYR'` | Functional or foreign-currency account (Wave 5). |

Account types remain: `asset`, `liability`, `equity`, `revenue`, `expense`.

### `journal_entry` enhancements

| Column | Type | Default | Purpose |
|--------|------|---------|---------|
| `source_type` | TEXT | `NULL` | Enum: `manual`, `invoice`, `bill`, `payment`, `expense`, `currency_adjustment`. |
| `source_id` | UUID | `NULL` | Links the journal back to the originating document. |
| `accounting_period_id` | UUID → `accounting_period(id)` | NULL | Set on post (Wave 8). |

### `journal_line` enhancements

| Column | Type | Default | Purpose |
|--------|------|---------|---------|
| `party_id` | UUID → `party(id)` | NULL | Already exists; used for AR/AP sub-ledgers. |
| `foreign_debit` | NUMERIC(19,4) | `0` | Wave 5: foreign currency debit. |
| `foreign_credit` | NUMERIC(19,4) | `0` | Wave 5: foreign currency credit. |
| `exchange_rate` | NUMERIC(19,6) | `1` | Wave 5: rate used to convert to MYR. |

### New tables introduced

| Table | Introduced in | Purpose |
|-------|---------------|---------|
| `payment` | Wave 3 | Customer or vendor payments. |
| `payment_allocation` | Wave 3 | Links a payment to one or more invoices/bills. |
| `bill` / `bill_line` | Wave 4 | Accounts payable documents. |
| `expense` / `expense_line` | Wave 4 | Direct expenses without a bill. |
| `tax_code` | Wave 6 | SST tax codes and rates. |
| `fiscal_year` | Wave 8 | Annual accounting boundary. |
| `accounting_period` | Wave 8 | Monthly period within a fiscal year. |

---

## Wave 1 — Chart of Accounts UI

### Goal
Let users view, create, edit, and archive the full chart of accounts.

### Data model
Use existing `account` table plus new `is_active` and `archived_at` columns.

Migration snippet:

```sql
ALTER TABLE account
  ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT true,
  ADD COLUMN archived_at TIMESTAMPTZ;
```

### Backend
- `GET /api/accounts` — list accounts for the workspace, optionally as a tree.
- `GET /api/accounts/{id}` — single account.
- `POST /api/accounts` — create account.
- `PATCH /api/accounts/{id}` — update name, code, parent, type.
- `DELETE /api/accounts/{id}` — archive (set `is_active = false, archived_at = now()`). Hard-delete only if no journal lines reference it.

### Validation rules
- `code` unique per workspace.
- `parent_account_id` cannot be itself or a descendant (no cycles).
- Parent and child must have compatible types (optional: parent type must equal child type).
- Cannot archive a system account or an account with non-zero balance.

### Frontend
- New page `/chart-of-accounts`.
- Tree/table view with expand/collapse.
- Form drawer for create/edit: code, name, type, parent.
- Archive action with confirmation.

### Acceptance criteria
- [ ] User can create a nested chart of accounts.
- [ ] Duplicate account code is rejected.
- [ ] Archived accounts are hidden by default but still visible in historical reports.
- [ ] Seeded Malaysian SME chart of accounts is created for every new workspace.

---

## Wave 2 — Manual Journal Entries

### Goal
Allow users to post balanced double-entry journals directly to the GL.

### Data model
No new tables; uses `journal_entry` and `journal_line`.

### Backend
- `GET /api/journal-entries` — list with optional date range.
- `GET /api/journal-entries/{id}` — full entry with lines.
- `POST /api/journal-entries` — create draft or posted entry.
- `PATCH /api/journal-entries/{id}` — update if still draft.
- `POST /api/journal-entries/{id}/post` — post a draft entry.
- `POST /api/journal-entries/{id}/cancel` — reverse a posted entry with an offsetting journal.

### Request shape

```json
{
  "entry_date": "2026-10-08",
  "reference": "ADJ-001",
  "description": "Opening balances",
  "status": "draft",
  "lines": [
    { "account_id": "...", "debit": "10000.00", "credit": "0", "description": "Cash" },
    { "account_id": "...", "debit": "0", "credit": "10000.00", "description": "Capital" }
  ]
}
```

### Validation rules
- At least two lines.
- Sum of debits must equal sum of credits.
- Each line must have either debit or credit > 0, not both.
- All accounts belong to the workspace and are active.
- Entry date must fall within an open period after Wave 8.

### Frontend
- New page `/journal-entries`.
- Journal entry list with status badges.
- Form page with dynamic lines, running debit/credit totals, and validation errors.
- Post / cancel actions.

### Acceptance criteria
- [ ] User can create a balanced journal entry.
- [ ] Unbalanced entry is rejected with a clear message.
- [ ] Posted journals appear in the GL and cannot be edited.
- [ ] Cancelling a posted journal creates a reversing entry and leaves an audit trail.

---

## Wave 3 — Payments & Allocation

### Goal
Record customer and vendor payments and allocate them to invoices or bills.

### Data model

```sql
CREATE TABLE payment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    party_id UUID REFERENCES party(id) ON DELETE SET NULL,
    payment_date DATE NOT NULL,
    amount NUMERIC(19,4) NOT NULL CHECK (amount > 0),
    currency TEXT NOT NULL DEFAULT 'MYR',
    payment_method TEXT NOT NULL, -- bank_transfer, cash, cheque, card
    reference TEXT,
    notes TEXT,
    direction TEXT NOT NULL CHECK (direction IN ('received', 'sent')), -- AR vs AP
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
```

### Backend
- `GET /api/payments`
- `GET /api/payments/{id}`
- `POST /api/payments` — create with allocations.
- `PATCH /api/payments/{id}` — only if draft.
- `POST /api/payments/{id}/post`
- `POST /api/payments/{id}/cancel`

### GL posting rules

**Customer payment (received):**
```
Debit:  Bank (asset)
Credit: Accounts Receivable (asset) — optionally by party
```

**Vendor payment (sent):**
```
Debit: Accounts Payable (liability) — optionally by party
Credit: Bank (asset)
```

The payment also updates invoice/bill status:
- Fully allocated → `paid`
- Partially allocated → `partial`
- Over-allocated → rejected

### Frontend
- New page `/payments`.
- Payment form: party, date, amount, method, reference, allocations.
- Allocation picker shows open invoices/bills for the selected party.

### Acceptance criteria
- [ ] Recording a full payment marks the invoice as paid.
- [ ] Partial payment marks the invoice as partial and leaves remaining balance.
- [ ] Total allocations cannot exceed payment amount.
- [ ] GL shows the bank and AR/AP impact.
- [ ] Cancelling a payment reverses the GL entry and allocations.

---

## Wave 4 — Bills & Expenses

### Goal
Handle accounts payable and ad-hoc expenses.

### Data model

```sql
CREATE TABLE bill (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
    party_id UUID NOT NULL REFERENCES party(id) ON DELETE CASCADE,
    bill_number TEXT NOT NULL,
    issue_date DATE NOT NULL,
    due_date DATE,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'open', 'partial', 'paid', 'overdue', 'cancelled')),
    total_amount NUMERIC(19,4) NOT NULL DEFAULT 0,
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
    quantity NUMERIC(19,4) NOT NULL DEFAULT 1,
    unit_price NUMERIC(19,4) NOT NULL DEFAULT 0,
    amount NUMERIC(19,4) NOT NULL DEFAULT 0,
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
    total_amount NUMERIC(19,4) NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'MYR',
    payment_method TEXT,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'posted', 'cancelled')),
    journal_entry_id UUID REFERENCES journal_entry(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE expense_line (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    expense_id UUID NOT NULL REFERENCES expense(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    amount NUMERIC(19,4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### GL posting rules

**Post a bill:**
```
Debit:  Expense / Asset account (from lines)
Credit: Accounts Payable (liability)
```

**Post an expense:**
```
Debit:  Expense account (from lines)
Credit: Bank or Cash account (if paid immediately) OR Accounts Payable (if to be paid later)
```

### Backend
- `GET/POST/PATCH` for `/api/bills` and `/api/expenses`.
- Post and cancel actions.

### Frontend
- New pages `/bills` and `/expenses`.
- Forms similar to invoices.

### Acceptance criteria
- [ ] User can create and post a bill.
- [ ] User can record a direct expense.
- [ ] Bills integrate with payment allocation (Wave 3).
- [ ] AP balance is visible in reports after posting.

---

## Wave 5 — Multi-Currency

### Goal
Support foreign currency transactions while keeping MYR as the functional currency.

### Data model
- Add `currency` and `exchange_rate` to `journal_line`.
- Add `currency` to `account` (already added in Wave 1).

```sql
ALTER TABLE journal_line
  ADD COLUMN foreign_debit NUMERIC(19,4) NOT NULL DEFAULT 0,
  ADD COLUMN foreign_credit NUMERIC(19,4) NOT NULL DEFAULT 0,
  ADD COLUMN exchange_rate NUMERIC(19,6) NOT NULL DEFAULT 1;
```

### Supported currencies
Primary: `MYR`. Secondary: `USD`, `EUR`, `SGD`.

### Rules
- Every transaction stores the foreign amount and the exchange rate.
- The functional (MYR) amount is `foreign_amount * exchange_rate`.
- GL balances are always reported in MYR.
- Bank and AR/AP accounts can be foreign-currency denominated.

### Frontend
- Currency selector on invoices, bills, payments, expenses.
- Exchange rate input with editable default fetched from a simple rate table or user entry.

### Acceptance criteria
- [ ] User can create a USD invoice and see the MYR equivalent in the GL.
- [ ] Reports show MYR totals while transaction views show original currency.
- [ ] Unrealised forex gains/losses can be recorded manually (auto-calculation deferred).

---

## Wave 6 — SST Support

### Goal
Add Sales and Service Tax codes and produce an SST-02 prep report.

### Data model

```sql
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
```

Add `tax_code_id` and `tax_amount` to `invoice_line` and `bill_line`.

### GL posting rules

Tax is collected on behalf of the government and posted to a liability account:

```
Debit:  Accounts Receivable / Bank (total including tax)
Credit: Revenue (net)
Credit: SST Payable (liability)
```

### Backend
- `GET/POST/PATCH/DELETE /api/tax-codes`
- SST report endpoint: `GET /api/reports/sst-02?from=...&to=...`

### Frontend
- Tax code dropdown on invoice/bill lines.
- SST-02 prep report page with taxable amount, tax amount, and total.

### Acceptance criteria
- [ ] User can assign an SST tax code to an invoice line.
- [ ] SST liability account increases when invoice is posted.
- [ ] SST-02 report shows totals by tax code for a date range.

---

## Wave 7 — Financial Reports

### Goal
Generate the four core financial statements from the GL.

### Reports

| Report | Description |
|--------|-------------|
| **Trial Balance** | All accounts with opening balance, debits, credits, and closing balance for a period. |
| **Profit & Loss** | Revenue minus expenses for a date range. |
| **Balance Sheet** | Assets = Liabilities + Equity as at a date. |
| **Aged Receivables** | Unpaid customer invoices grouped by age bucket. |
| **Aged Payables** | Unpaid vendor bills grouped by age bucket. |

### Backend
- `GET /api/reports/trial-balance?period_id=...`
- `GET /api/reports/profit-loss?from=...&to=...`
- `GET /api/reports/balance-sheet?as_at=...`
- `GET /api/reports/aged-receivables?as_at=...`
- `GET /api/reports/aged-payables?as_at=...`

### Report formulas

**Trial Balance:**
- Opening = sum of debits − credits before the period start.
- Period debits/credits = sum within the period.
- Closing = Opening + Period debits − Period credits.

**P&L:**
- Revenue total − Expense total.
- Ignores asset, liability, equity accounts.

**Balance Sheet:**
- Assets = Liabilities + Equity.
- Uses closing balances as at the report date.
- Retained earnings can be approximated by cumulative P&L until Wave 8 introduces formal retained earnings.

### Frontend
- New page `/reports` with tabs for each report.
- Date/period selectors.
- Export to CSV.

### Acceptance criteria
- [ ] P&L matches the sum of revenue and expense journal lines.
- [ ] Balance Sheet balances (Assets = Liabilities + Equity).
- [ ] Aged receivables total equals open customer invoice balance.

---

## Wave 8 — Fiscal Year & Period Locking

### Goal
Enable month-end/year-end close and prevent edits to locked periods.

### Data model

```sql
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
```

### Rules
- A fiscal year is automatically created when a workspace is registered.
- Each fiscal year has 12 monthly accounting periods.
- Posting, editing, or cancelling a transaction in a closed period is rejected.
- Closing a period sets `is_closed = true` and records the user/timestamp.
- Re-opening is allowed only if the following period is also open.

### Backend
- `GET /api/fiscal-years`
- `GET /api/accounting-periods`
- `POST /api/accounting-periods/{id}/close`
- `POST /api/accounting-periods/{id}/reopen`

### Frontend
- Settings page section for fiscal years and periods.
- Close/reopen buttons with confirmation.

### Acceptance criteria
- [ ] Workspace has an active fiscal year with 12 periods on creation.
- [ ] User can close a period.
- [ ] Attempting to post a journal into a closed period is rejected.
- [ ] Re-opening a closed period works if no later closed period exists.

---

## Suggested implementation order

1. **Wave 1 + Wave 2** — COA and manual journals. These unlock the GL surface.
2. **Wave 3** — Payments (depends on invoices from Phase 1).
3. **Wave 4** — Bills and expenses.
4. **Wave 5** — Multi-currency (can be deferred if not needed immediately).
5. **Wave 6** — SST (needed for Malaysian compliance).
6. **Wave 7** — Reports (depends on Waves 1–4).
7. **Wave 8** — Period locking (depends on all posting flows).

---

## Open questions to resolve before coding

1. Should we auto-create retained earnings and equity accounts during workspace setup, or rely on the seeded COA?
2. Do we need bank/cash accounts seeded, and how many default payment methods?
3. Should multi-currency exchange rates be stored in a table with history, or entered per transaction for Wave 5?
4. For SST-02, do we need exact LHDN form formatting now, or just totals grouped by code?
5. Should period locking also lock invoice/bill status changes, or only GL postings?

---

## Definition of Phase 2 done

- User can create and manage a full chart of accounts.
- User can post manual journal entries.
- User can record payments and allocate them to invoices/bills.
- User can record bills and expenses.
- (Optional) User can handle USD/EUR/SGD transactions.
- User can apply SST tax codes and view SST liability.
- User can run P&L, Balance Sheet, Trial Balance, Aged Receivables/Payables.
- User can close an accounting period.

> Exit gate: **A user can close a month and produce a P&L and balance sheet.**
