export interface Contact {
  id: string;
  name: string;
  email: string;
  company: string;
  status: "lead" | "customer" | "partner";
}

export interface Company {
  id: string;
  name: string;
  industry: string;
  size: string;
  revenue: string;
}

export interface Deal {
  id: string;
  name: string;
  company: string;
  value: number;
  stage: "prospecting" | "negotiation" | "closed-won" | "closed-lost";
  probability: number;
}

export interface Invoice {
  id: string;
  number: string;
  client: string;
  amount: number;
  status: "draft" | "sent" | "paid" | "overdue";
  dueDate: string;
}

export interface Employee {
  id: string;
  name: string;
  role: string;
  department: string;
  email: string;
}

export const contacts: Contact[] = [
  { id: "1", name: "Alice Johnson", email: "alice@example.com", company: "Acme Corp", status: "customer" },
  { id: "2", name: "Bob Smith", email: "bob@example.com", company: "Globex", status: "lead" },
  { id: "3", name: "Carol White", email: "carol@example.com", company: "Initech", status: "partner" },
];

export const companies: Company[] = [
  { id: "1", name: "Acme Corp", industry: "Manufacturing", size: "50-200", revenue: "$5M" },
  { id: "2", name: "Globex", industry: "Technology", size: "200-1000", revenue: "$25M" },
  { id: "3", name: "Initech", industry: "Software", size: "10-50", revenue: "$2M" },
];

export const deals: Deal[] = [
  { id: "1", name: "Enterprise License", company: "Acme Corp", value: 50000, stage: "negotiation", probability: 75 },
  { id: "2", name: "Annual Support", company: "Globex", value: 12000, stage: "closed-won", probability: 100 },
  { id: "3", name: "Pilot Project", company: "Initech", value: 8000, stage: "prospecting", probability: 30 },
];

export const invoices: Invoice[] = [
  { id: "1", number: "INV-001", client: "Acme Corp", amount: 5000, status: "paid", dueDate: "2026-09-15" },
  { id: "2", number: "INV-002", client: "Globex", amount: 12000, status: "sent", dueDate: "2026-10-20" },
  { id: "3", number: "INV-003", client: "Initech", amount: 3000, status: "overdue", dueDate: "2026-09-01" },
];

export const employees: Employee[] = [
  { id: "1", name: "Sarah Connor", role: "CEO", department: "Executive", email: "sarah@poweros.dev" },
  { id: "2", name: "John Doe", role: "Engineer", department: "Engineering", email: "john@poweros.dev" },
  { id: "3", name: "Emily Chen", role: "Designer", department: "Product", email: "emily@poweros.dev" },
];
