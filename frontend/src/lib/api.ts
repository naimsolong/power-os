const API_BASE = "/api";

export interface AuthUser {
  user_id: string;
  workspace_id: string;
  email: string;
  name: string | null;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export type PartyType = "customer" | "vendor" | "other";

export interface Party {
  id: string;
  name: string;
  email: string | null;
  phone: string | null;
  address: string | null;
  party_type: PartyType;
}

export interface PartyCreate {
  name: string;
  email?: string;
  phone?: string;
  address?: string;
  party_type: PartyType;
}

export type PartyUpdate = Partial<PartyCreate>;

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
  return response.json() as Promise<T>;
}

export async function loginUser(credentials: LoginRequest): Promise<AuthUser> {
  const response = await fetch(`${API_BASE}/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    credentials: "include",
    body: JSON.stringify(credentials),
  });
  return handleResponse<AuthUser>(response);
}

export async function logoutUser(): Promise<void> {
  const response = await fetch(`${API_BASE}/auth/logout`, {
    method: "POST",
    credentials: "include",
  });
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`Logout failed with status ${response.status}: ${text}`);
  }
}

export async function fetchMe(): Promise<AuthUser> {
  const response = await fetch(`${API_BASE}/auth/me`, {
    credentials: "include",
  });
  return handleResponse<AuthUser>(response);
}

export async function fetchParties(
  partyType?: PartyType,
  search?: string
): Promise<Party[]> {
  const params = new URLSearchParams();
  if (partyType) params.set("party_type", partyType);
  if (search) params.set("search", search);
  const query = params.toString();
  const url = `${API_BASE}/parties${query ? `?${query}` : ""}`;

  const response = await fetch(url, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Party[]>(response);
}

export async function fetchParty(id: string): Promise<Party> {
  const response = await fetch(`${API_BASE}/parties/${id}`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Party>(response);
}

export async function createParty(party: PartyCreate): Promise<Party> {
  const response = await fetch(`${API_BASE}/parties`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(party),
  });
  return handleResponse<Party>(response);
}

export async function updateParty(id: string, party: PartyUpdate): Promise<Party> {
  const response = await fetch(`${API_BASE}/parties/${id}`, {
    method: "PATCH",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(party),
  });
  return handleResponse<Party>(response);
}

export async function deleteParty(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/parties/${id}`, {
    method: "DELETE",
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
}

export const PARTY_TYPE_LABELS: Record<PartyType, string> = {
  customer: "Customer",
  vendor: "Vendor",
  other: "Other",
};

export const PARTY_TYPE_OPTIONS: PartyType[] = ["customer", "vendor", "other"];

export type InvoiceStatus = "draft" | "sent" | "paid" | "overdue";

export const INVOICE_STATUS_LABELS: Record<InvoiceStatus, string> = {
  draft: "Draft",
  sent: "Sent",
  paid: "Paid",
  overdue: "Overdue",
};

export const INVOICE_STATUS_OPTIONS: InvoiceStatus[] = [
  "draft",
  "sent",
  "paid",
  "overdue",
];

export interface InvoiceLine {
  id: string;
  invoice_id: string;
  description: string;
  quantity: number;
  unit_price: number;
  line_total: number;
}

export interface Invoice {
  id: string;
  party_id: string;
  invoice_number: string;
  issue_date: string;
  due_date: string;
  status: InvoiceStatus;
  total_amount: number;
  currency: string;
}

export interface InvoiceCreate {
  party_id: string;
  invoice_number: string;
  issue_date: string;
  due_date: string;
  status?: InvoiceStatus;
  currency?: string;
}

export type InvoiceUpdate = Partial<InvoiceCreate>;

export interface InvoiceLineCreate {
  description: string;
  quantity: number;
  unit_price: number;
  line_total: number;
}

export type InvoiceLineUpdate = Partial<InvoiceLineCreate>;

export async function fetchInvoices(
  search?: string
): Promise<Invoice[]> {
  const params = new URLSearchParams();
  if (search) params.set("search", search);
  const query = params.toString();
  const url = `${API_BASE}/invoices${query ? `?${query}` : ""}`;

  const response = await fetch(url, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Invoice[]>(response);
}

export async function fetchInvoice(id: string): Promise<Invoice> {
  const response = await fetch(`${API_BASE}/invoices/${id}`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Invoice>(response);
}

export async function createInvoice(invoice: InvoiceCreate): Promise<Invoice> {
  const response = await fetch(`${API_BASE}/invoices`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(invoice),
  });
  return handleResponse<Invoice>(response);
}

export async function updateInvoice(
  id: string,
  invoice: InvoiceUpdate
): Promise<Invoice> {
  const response = await fetch(`${API_BASE}/invoices/${id}`, {
    method: "PATCH",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(invoice),
  });
  return handleResponse<Invoice>(response);
}

export async function deleteInvoice(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/invoices/${id}`, {
    method: "DELETE",
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
}

export async function postInvoice(id: string): Promise<Invoice> {
  const response = await fetch(`${API_BASE}/invoices/${id}/post`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  });
  return handleResponse<Invoice>(response);
}

export async function fetchInvoiceLines(id: string): Promise<InvoiceLine[]> {
  const response = await fetch(`${API_BASE}/invoices/${id}/lines`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<InvoiceLine[]>(response);
}

export async function createInvoiceLine(
  id: string,
  line: InvoiceLineCreate
): Promise<InvoiceLine> {
  const response = await fetch(`${API_BASE}/invoices/${id}/lines`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(line),
  });
  return handleResponse<InvoiceLine>(response);
}

export async function updateInvoiceLine(
  id: string,
  lineId: string,
  line: InvoiceLineUpdate
): Promise<InvoiceLine> {
  const response = await fetch(`${API_BASE}/invoices/${id}/lines/${lineId}`, {
    method: "PATCH",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(line),
  });
  return handleResponse<InvoiceLine>(response);
}

export async function deleteInvoiceLine(
  id: string,
  lineId: string
): Promise<void> {
  const response = await fetch(`${API_BASE}/invoices/${id}/lines/${lineId}`, {
    method: "DELETE",
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
}
