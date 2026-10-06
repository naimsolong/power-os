# Power OS Roadmap

Open-source, self-hostable business operating system for Malaysian SMEs.
MIT licensed. Rust backend. One-record architecture across CRM, finance, people, and AI.

---

## 1. Product Vision

Power OS is a Malaysian-focused business operating system that gives SMEs one login, one record, and one AI assistant across sales, finance, people, and leadership.

It is designed to be:
- **Open-source** (MIT) — inspectable, forkable, resellable by Malaysian agencies
- **Self-hostable** — Docker Compose on a single VPS by default
- **Compliance-first** — LHDN e-Invoice, EPF/SOCSO/EIS/PCB, SST as core features
- **AI-augmented** — read-only agents that report across the business, not autonomous actors

---

## 2. Core Principles

1. **One record per party.** A person or organization can be a lead, customer, vendor, and employee without data duplication.
2. **Read-only AI by default.** Agents report; humans act. Every tool call is logged.
3. **Compliance is a feature, not a plugin.** Malaysian statutory requirements are first-class modules.
4. **Build for self-hosting first.** Managed hosting is the same image, managed by us.
5. **Phased, not parallel.** Finish one pillar before adding the next.

---

## 3. Target Architecture

### Backend
| Layer | Choice |
|-------|--------|
| Language | Rust |
| Web framework | Axum |
| Database access | SQLx (compile-time checked queries) |
| Database | PostgreSQL |
| Async runtime | Tokio |
| Currency | rust_decimal only |
| Auth | Argon2 + session cookies |
| Background jobs | pg-boss / sqlx queue table first, Redis later |
| Email | lettre |
| HTTP client | reqwest |
| API docs | Utoipa (OpenAPI) |
| CLI tools | clap |
| Logging | tracing |

### Frontend
| Layer | Choice |
|-------|--------|
| Framework | React (SPA) |
| Language | TypeScript |
| Styling | Tailwind CSS |
| Server state | TanStack Query |
| Components | shadcn/ui / Radix primitives |
| Build | Vite |

### Self-Hosting
- Docker Compose: app, PostgreSQL, MinIO, Caddy
- Environment-driven configuration
- One-command install script
- Backup script for managed hosting tier

### AI Layer
- OpenRouter as default LLM gateway
- Ollama/local model support for self-hosters
- Function-calling with tool-call logging
- Read-only tools by default

---

## 4. Foundational Data Model

These tables are built in Phase 1 and extended by every later phase.

- `workspace` — tenant isolation
- `user` — authentication identity
- `workspace_user` — membership and role
- `party` — unified entity (lead, customer, vendor, employee)
- `party_relationship` — typed links between parties
- `account` — chart of accounts
- `journal_entry` + `journal_line` — double-entry foundation
- `invoice` + `invoice_line` — billing
- `deal` + `deal_stage` + `deal_pipeline` — CRM
- `employee` — people record
- `activity` — audit timeline
- `ai_session` + `ai_tool_call` — AI provenance

The general ledger exists in the schema from Phase 1. The UI exposes more of it over time.

---

## 5. Phase Plan

### Phase 1 — MVP: One-Record OS (horizontal slice)
**Goal:** Ship a usable, self-hostable product that demonstrates all four pillars with minimal depth.

**Deliverables:**
- Project scaffold: Rust workspace, migrations, Docker Compose, CI skeleton
- Auth: registration, login, logout, password reset, workspace creation, roles
- Party model: unified contacts/companies with role tagging
- CRM: contacts, companies, deals pipeline, stages, notes, tasks
- Finance: Chart of Accounts seed, invoice creation, invoice → journal entry, payment recording
- LHDN e-Invoice: sandbox registration, invoice JSON mapping, submission, status polling, QR code
- People: employee directory linked to party
- AI assistant: reads contacts, deals, invoices, employees; answers plain questions; logs tool calls
- Landing page, README, MIT license, self-host docs

**Commercial state:** Not monetized. Free, open-source, self-host only.

**Exit gate:** A non-developer can install it and create an invoice that submits to LHDN sandbox.

---

### Phase 2 — Full Finance Surface
**Goal:** Make the accounting layer usable for real books.

**Deliverables:**
- Full Chart of Accounts UI (create, edit, archive accounts)
- Journal entry UI (manual entries with debit/credit validation)
- Bank reconciliation
- Expense recording and bills
- Payment methods and payment allocation
- Multi-currency support (MYR primary, USD/EUR/SGD secondary)
- SST support: tax codes, SST-02 prep, submission-ready reports
- Financial reports: P&L, Balance Sheet, Trial Balance, Aged Receivables/Payables
- Fiscal year and period locking

**Commercial state:** Still free/self-host. Gather real usage before paid hosting.

**Exit gate:** A user can close a month and produce a P&L and balance sheet.

---

### Phase 3 — HR Operations
**Goal:** Add operational people management without payroll complexity yet.

**Deliverables:**
- Leave types and leave balances
- Leave application and approval workflow
- Attendance tracking (manual entry + CSV import)
- Claims: submission, receipts, approval, status
- Org chart
- Employee documents and notes
- Calendar/schedule views

**Commercial state:** Optional early managed hosting pilot for willing users.

**Exit gate:** A company can track leave and claims end-to-end.

---

### Phase 4 — Payroll + Malaysian Statutory Compliance
**Goal:** Run payroll with correct Malaysian statutory deductions.

**Deliverables:**
- Monthly payroll run with basic salary, allowances, deductions
- EPF employee/employer contribution calculation
- SOCSO employee/employer contribution calculation
- EIS employee/employer contribution calculation
- PCB income tax deduction (MTD tables)
- Payslip generation
- Bank file export for salary payment
- Statutory contribution reports and exports
- Integration with Phase 3 leave/claims so unpaid leave and claimable amounts flow into payroll
- Review by a Malaysian payroll/accounting consultant

**Commercial state:** Launch paid managed hosting. This is the phase where statutory risk justifies paying for hosted, supported software.

**Exit gate:** A user can run a payroll month and generate EPF/SOCSO/EIS/PCB numbers.

---

### Phase 5 — Marketing + C-Suite Agents
**Goal:** Add revenue-generating and leadership features.

**Deliverables:**
- Meta Business / Google Ads campaign import
- Campaign spend and lead tracking
- Lead form builder
- AI chatbot on website
- Marketing automations engine
- Creative bank (asset storage + AI variant generation)
- Magic Ads-style autopilot rules (read-only recommendations first)
- C-Suite agents: CEO Overview, CMO, CHRO, CFO
- Cross-module reporting: cash + pipeline + payroll + campaign ROI
- Voice interface for C-Suite agents

**Commercial state:** Managed hosting becomes primary revenue. Self-host remains MIT.

**Exit gate:** A business owner can ask "how is the month going?" and get a cited answer across sales, cash, and payroll.

---

### Phase 6 — Commercial Platform
**Goal:** Turn the open-source project into a sustainable business.

**Deliverables:**
- Managed hosting control panel
- Tiered pricing (anchor: RM199/mo for Lite, RM499/mo Plus, RM999/mo Max)
- Backup, monitoring, and upgrade automation
- Support portal and SLA tiers
- App marketplace / plugin system
- White-label / agency reseller program
- Migration service from Kuasa, Qne, SQL Account, Kakitangan, etc.
- SOC 2 / ISO 27001 readiness path

**Commercial state:** Paid managed hosting is the main revenue model. Open-source core drives adoption and trust.

**Exit gate:** First 100 paying managed-hosting customers.

---

## 6. Key Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| No domain expertise in accounting/payroll | High | Start with invoice-driven journals. Hire Malaysian consultant before Phase 4. |
| LHDN API changes | High | Isolate e-Invoice logic in its own crate/module with versioned adapters. |
| Rust development velocity (solo) | Medium | Keep modules small. Use SQLx for confidence. Avoid premature abstraction. |
| Scope creep across four pillars | High | Strict milestone gates. No new pillar until current one ships. |
| Self-host support burden | Medium | Paid hosting is the recommended path; self-host is community-supported. |
| Competition with Kuasa's marketing | Medium | Compete on openness, data ownership, price, and self-hosting — not ads. |

---

## 7. Success Metrics

- Phase 1: First LHDN sandbox submission, first external self-hosted user
- Phase 2: First user closes a monthly book
- Phase 3: First company tracking leave/claims
- Phase 4: First payroll run with statutory reports
- Phase 5: First C-Suite agent answer across modules
- Phase 6: 100 paying managed-hosting customers

---

## 8. Open Decisions

These can be deferred until they block implementation:

1. **Pricing exact figures** — use Kuasa anchor temporarily; finalize before Phase 6.
2. **AI credit model** — resell OpenRouter credits, or BYO-key only?
3. **Third-party integrations** — which bank feed providers? Which ad platforms first?
4. **Compliance review budget** — when and how to engage Malaysian accountant/payroll consultant.

---

## 9. References

This roadmap is based on a competitive analysis of Kuasa OS (launch.kuasa.ai), ERPNext/Frappe architecture, and Malaysian SME compliance requirements.
