import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseQueryOptions,
} from "@tanstack/react-query";

const API_BASE = "/api";
const BILLS_KEY = "bills";
const BILL_KEY = "bill";

export type BillStatus =
  | "draft"
  | "open"
  | "partial"
  | "paid"
  | "overdue"
  | "cancelled";

export const BILL_STATUS_LABELS: Record<BillStatus, string> = {
  draft: "Draft",
  open: "Open",
  partial: "Partial",
  paid: "Paid",
  overdue: "Overdue",
  cancelled: "Cancelled",
};

export const BILL_STATUS_OPTIONS: BillStatus[] = [
  "draft",
  "open",
  "partial",
  "paid",
  "overdue",
  "cancelled",
];

export interface BillLine {
  id: string;
  bill_id: string;
  description: string;
  account_id: string;
  quantity: number;
  unit_price: number;
  amount: number;
}

export interface Bill {
  id: string;
  workspace_id: string;
  party_id: string;
  bill_number: string;
  issue_date: string;
  due_date: string;
  status: BillStatus;
  total_amount: number;
  currency: string;
  journal_entry_id?: string;
  lines?: BillLine[];
}

export interface BillLineCreate {
  description: string;
  account_id: string;
  quantity: number;
  unit_price: number;
}

export interface BillCreate {
  party_id: string;
  bill_number: string;
  issue_date: string;
  due_date?: string;
  currency?: string;
  lines: BillLineCreate[];
}

export type BillUpdate = Partial<BillCreate>;

export interface PostedBill {
  bill: Bill;
  journal_entry_id: string;
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
  return response.json() as Promise<T>;
}

export async function fetchBills(search?: string): Promise<Bill[]> {
  const params = new URLSearchParams();
  if (search) params.set("search", search);
  const query = params.toString();
  const url = `${API_BASE}/bills${query ? `?${query}` : ""}`;

  const response = await fetch(url, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Bill[]>(response);
}

export async function fetchBill(id: string): Promise<Bill> {
  const response = await fetch(`${API_BASE}/bills/${id}`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Bill>(response);
}

export async function createBill(bill: BillCreate): Promise<Bill> {
  const response = await fetch(`${API_BASE}/bills`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(bill),
  });
  return handleResponse<Bill>(response);
}

export async function updateBill(
  id: string,
  bill: BillUpdate
): Promise<Bill> {
  const response = await fetch(`${API_BASE}/bills/${id}`, {
    method: "PATCH",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(bill),
  });
  return handleResponse<Bill>(response);
}

export async function postBill(id: string): Promise<PostedBill> {
  const response = await fetch(`${API_BASE}/bills/${id}/post`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  });
  return handleResponse<PostedBill>(response);
}

export async function cancelBill(id: string): Promise<Bill> {
  const response = await fetch(`${API_BASE}/bills/${id}/cancel`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  });
  return handleResponse<Bill>(response);
}

export function useBills(
  search?: string,
  options?: Omit<UseQueryOptions<Bill[]>, "queryKey" | "queryFn">
) {
  return useQuery<Bill[]>({
    queryKey: [BILLS_KEY, { search }],
    queryFn: () => fetchBills(search),
    ...options,
  });
}

export function useBill(id: string | null) {
  return useQuery<Bill>({
    queryKey: [BILL_KEY, id],
    queryFn: () => fetchBill(id as string),
    enabled: !!id,
  });
}

export function useCreateBill() {
  const queryClient = useQueryClient();
  return useMutation<Bill, Error, BillCreate>({
    mutationFn: createBill,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [BILLS_KEY] });
    },
  });
}

export function useUpdateBill() {
  const queryClient = useQueryClient();
  return useMutation<Bill, Error, { id: string; data: BillUpdate }>({
    mutationFn: ({ id, data }) => updateBill(id, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [BILLS_KEY] });
      queryClient.invalidateQueries({ queryKey: [BILL_KEY, variables.id] });
    },
  });
}

export function usePostBill() {
  const queryClient = useQueryClient();
  return useMutation<PostedBill, Error, string>({
    mutationFn: postBill,
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: [BILLS_KEY] });
      queryClient.invalidateQueries({ queryKey: [BILL_KEY, id] });
    },
  });
}

export function useCancelBill() {
  const queryClient = useQueryClient();
  return useMutation<Bill, Error, string>({
    mutationFn: cancelBill,
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: [BILLS_KEY] });
      queryClient.invalidateQueries({ queryKey: [BILL_KEY, id] });
    },
  });
}
