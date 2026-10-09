import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseQueryOptions,
} from "@tanstack/react-query";

const API_BASE = "/api";
const ACCOUNTS_KEY = "accounts";
const ACCOUNT_KEY = "account";

export type AccountType = "asset" | "liability" | "equity" | "revenue" | "expense";

export const ACCOUNT_TYPE_LABELS: Record<AccountType, string> = {
  asset: "Asset",
  liability: "Liability",
  equity: "Equity",
  revenue: "Revenue",
  expense: "Expense",
};

export const ACCOUNT_TYPE_OPTIONS: AccountType[] = [
  "asset",
  "liability",
  "equity",
  "revenue",
  "expense",
];

export interface Account {
  id: string;
  workspace_id: string;
  code: string;
  name: string;
  account_type: AccountType;
  parent_account_id: string | null;
  parent_code: string | null;
  parent_name: string | null;
  is_active: boolean;
  archived_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface AccountCreate {
  code: string;
  name: string;
  account_type: AccountType;
  parent_account_id?: string | null;
}

export type AccountUpdate = Partial<AccountCreate>;

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
  return response.json() as Promise<T>;
}

export async function fetchAccounts(): Promise<Account[]> {
  const response = await fetch(`${API_BASE}/accounts`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Account[]>(response);
}

export async function fetchAccount(id: string): Promise<Account> {
  const response = await fetch(`${API_BASE}/accounts/${id}`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Account>(response);
}

export async function createAccount(account: AccountCreate): Promise<Account> {
  const response = await fetch(`${API_BASE}/accounts`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(account),
  });
  return handleResponse<Account>(response);
}

export async function updateAccount(
  id: string,
  account: AccountUpdate
): Promise<Account> {
  const response = await fetch(`${API_BASE}/accounts/${id}`, {
    method: "PATCH",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(account),
  });
  return handleResponse<Account>(response);
}

export async function archiveAccount(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/accounts/${id}`, {
    method: "DELETE",
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
}

export function useAccounts(
  options?: Omit<UseQueryOptions<Account[]>, "queryKey" | "queryFn">
) {
  return useQuery<Account[]>({
    queryKey: [ACCOUNTS_KEY],
    queryFn: fetchAccounts,
    ...options,
  });
}

export function useAccount(id: string | null) {
  return useQuery<Account>({
    queryKey: [ACCOUNT_KEY, id],
    queryFn: () => fetchAccount(id as string),
    enabled: id !== null,
  });
}

export function useCreateAccount() {
  const queryClient = useQueryClient();
  return useMutation<Account, Error, AccountCreate>({
    mutationFn: createAccount,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [ACCOUNTS_KEY] });
    },
  });
}

export function useUpdateAccount() {
  const queryClient = useQueryClient();
  return useMutation<Account, Error, { id: string; data: AccountUpdate }>({
    mutationFn: ({ id, data }) => updateAccount(id, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [ACCOUNTS_KEY] });
      queryClient.invalidateQueries({
        queryKey: [ACCOUNT_KEY, variables.id],
      });
    },
  });
}

export function useArchiveAccount() {
  const queryClient = useQueryClient();
  return useMutation<void, Error, string>({
    mutationFn: archiveAccount,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [ACCOUNTS_KEY] });
    },
  });
}
