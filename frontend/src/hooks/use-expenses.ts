import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseQueryOptions,
} from "@tanstack/react-query";

const API_BASE = "/api";
const EXPENSES_KEY = "expenses";
const EXPENSE_KEY = "expense";

export type ExpenseStatus = "draft" | "posted" | "cancelled";

export const EXPENSE_STATUS_LABELS: Record<ExpenseStatus, string> = {
  draft: "Draft",
  posted: "Posted",
  cancelled: "Cancelled",
};

export const EXPENSE_STATUS_OPTIONS: ExpenseStatus[] = [
  "draft",
  "posted",
  "cancelled",
];

export const PAYMENT_METHOD_OPTIONS = [
  "cash",
  "bank_transfer",
  "card",
  "cheque",
];

export interface ExpenseLine {
  id: string;
  expense_id: string;
  description: string;
  account_id: string;
  amount: number;
}

export interface Expense {
  id: string;
  workspace_id: string;
  party_id: string | null;
  expense_date: string;
  description: string;
  reference: string | null;
  total_amount: number;
  currency: string;
  payment_method: string | null;
  status: ExpenseStatus;
  paid_from_account_id: string | null;
  journal_entry_id?: string;
  lines?: ExpenseLine[];
}

export interface ExpenseLineCreate {
  description: string;
  account_id: string;
  amount: number;
}

export interface ExpenseCreate {
  party_id?: string;
  expense_date: string;
  description: string;
  reference?: string;
  payment_method?: string;
  paid_from_account_id?: string;
  currency?: string;
  lines: ExpenseLineCreate[];
}

export type ExpenseUpdate = Partial<ExpenseCreate>;

export interface PostedExpense {
  expense: Expense;
  journal_entry_id: string;
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
  return response.json() as Promise<T>;
}

export async function fetchExpenses(search?: string): Promise<Expense[]> {
  const params = new URLSearchParams();
  if (search) params.set("search", search);
  const query = params.toString();
  const url = `${API_BASE}/expenses${query ? `?${query}` : ""}`;

  const response = await fetch(url, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Expense[]>(response);
}

export async function fetchExpense(id: string): Promise<Expense> {
  const response = await fetch(`${API_BASE}/expenses/${id}`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Expense>(response);
}

export async function createExpense(expense: ExpenseCreate): Promise<Expense> {
  const response = await fetch(`${API_BASE}/expenses`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(expense),
  });
  return handleResponse<Expense>(response);
}

export async function updateExpense(
  id: string,
  expense: ExpenseUpdate
): Promise<Expense> {
  const response = await fetch(`${API_BASE}/expenses/${id}`, {
    method: "PATCH",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(expense),
  });
  return handleResponse<Expense>(response);
}

export async function postExpense(id: string): Promise<PostedExpense> {
  const response = await fetch(`${API_BASE}/expenses/${id}/post`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  });
  return handleResponse<PostedExpense>(response);
}

export async function cancelExpense(id: string): Promise<Expense> {
  const response = await fetch(`${API_BASE}/expenses/${id}/cancel`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  });
  return handleResponse<Expense>(response);
}

export function useExpenses(
  search?: string,
  options?: Omit<UseQueryOptions<Expense[]>, "queryKey" | "queryFn">
) {
  return useQuery<Expense[]>({
    queryKey: [EXPENSES_KEY, { search }],
    queryFn: () => fetchExpenses(search),
    ...options,
  });
}

export function useExpense(id: string | null) {
  return useQuery<Expense>({
    queryKey: [EXPENSE_KEY, id],
    queryFn: () => fetchExpense(id as string),
    enabled: !!id,
  });
}

export function useCreateExpense() {
  const queryClient = useQueryClient();
  return useMutation<Expense, Error, ExpenseCreate>({
    mutationFn: createExpense,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [EXPENSES_KEY] });
    },
  });
}

export function useUpdateExpense() {
  const queryClient = useQueryClient();
  return useMutation<Expense, Error, { id: string; data: ExpenseUpdate }>({
    mutationFn: ({ id, data }) => updateExpense(id, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [EXPENSES_KEY] });
      queryClient.invalidateQueries({ queryKey: [EXPENSE_KEY, variables.id] });
    },
  });
}

export function usePostExpense() {
  const queryClient = useQueryClient();
  return useMutation<PostedExpense, Error, string>({
    mutationFn: postExpense,
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: [EXPENSES_KEY] });
      queryClient.invalidateQueries({ queryKey: [EXPENSE_KEY, id] });
    },
  });
}

export function useCancelExpense() {
  const queryClient = useQueryClient();
  return useMutation<Expense, Error, string>({
    mutationFn: cancelExpense,
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: [EXPENSES_KEY] });
      queryClient.invalidateQueries({ queryKey: [EXPENSE_KEY, id] });
    },
  });
}
