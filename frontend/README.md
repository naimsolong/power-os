# Power OS Frontend

React SPA scaffold for Power OS.

## Stack

- Vite
- React 19 + TypeScript
- Tailwind CSS 4
- TanStack Query
- React Router
- Radix UI primitives + shadcn/ui-style components

## Getting Started

```bash
npm install
npm run dev
```

The dev server starts at `http://localhost:5173` by default.

## Available Scripts

- `npm run dev` – Start the development server
- `npm run build` – Type-check and build for production
- `npm run preview` – Preview the production build
- `npm run lint` – Run Oxlint

## Project Structure

```
src/
  components/
    layout.tsx       # App shell with sidebar and top bar
    ui/              # Reusable UI components (Button, Input, Card, Table)
  data/
    placeholders.ts  # Static placeholder data
  lib/
    utils.ts         # cn() helper for Tailwind classes
  pages/
    dashboard.tsx
    contacts.tsx
    companies.tsx
    deals.tsx
    invoices.tsx
    employees.tsx
  App.tsx            # Router + TanStack Query provider
  main.tsx           # Entry point
```

## Routes

- `/` – Dashboard
- `/contacts` – Contacts
- `/companies` – Companies
- `/deals` – Deals
- `/invoices` – Invoices
- `/employees` – Employees

All routes render placeholder data; no backend integration is configured yet.
