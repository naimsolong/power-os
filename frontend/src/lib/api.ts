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
