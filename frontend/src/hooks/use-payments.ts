import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseQueryOptions,
} from "@tanstack/react-query";

const API_BASE = "/api";
const PAYMENTS_KEY = "payments";
const PAYMENT_KEY = "payment";

export type PaymentStatus = "draft" | "posted" | "cancelled";
export type PaymentDirection = "received" | "sent";
export type PaymentMethod = "bank_transfer" | "cash" | "cheque" | "card";

export const PAYMENT_STATUS_LABELS: Record<PaymentStatus, string> = {
  draft: "Draft",
  posted: "Posted",
  cancelled: "Cancelled",
};

export const PAYMENT_DIRECTION_LABELS: Record<PaymentDirection, string> = {
  received: "Received",
  sent: "Sent",
};

export const PAYMENT_METHOD_LABELS: Record<PaymentMethod, string> = {
  bank_transfer: "Bank Transfer",
  cash: "Cash",
  cheque: "Cheque",
  card: "Card",
};

export const PAYMENT_STATUS_OPTIONS: PaymentStatus[] = [
  "draft",
  "posted",
  "cancelled",
];

export const PAYMENT_DIRECTION_OPTIONS: PaymentDirection[] = [
  "received",
  "sent",
];

export const PAYMENT_METHOD_OPTIONS: PaymentMethod[] = [
  "bank_transfer",
  "cash",
  "cheque",
  "card",
];

export interface PaymentAllocation {
  id: string;
  payment_id: string;
  invoice_id?: string;
  bill_id?: string;
  amount: number;
}

export interface Payment {
  id: string;
  workspace_id: string;
  party_id: string;
  bank_account_id: string;
  payment_date: string;
  amount: number;
  currency: string;
  payment_method: PaymentMethod;
  reference?: string;
  notes?: string;
  direction: PaymentDirection;
  status: PaymentStatus;
  journal_entry_id?: string;
  allocations?: PaymentAllocation[];
}

export interface PaymentAllocationCreate {
  invoice_id?: string;
  bill_id?: string;
  amount: number;
}

export interface PaymentCreate {
  party_id: string;
  bank_account_id: string;
  payment_date: string;
  amount: number;
  currency?: string;
  payment_method: PaymentMethod;
  reference?: string;
  notes?: string;
  direction: PaymentDirection;
  allocations: PaymentAllocationCreate[];
}

export type PaymentUpdate = Partial<PaymentCreate>;

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
  return response.json() as Promise<T>;
}

export async function fetchPayments(search?: string): Promise<Payment[]> {
  const params = new URLSearchParams();
  if (search) params.set("search", search);
  const query = params.toString();
  const url = `${API_BASE}/payments${query ? `?${query}` : ""}`;

  const response = await fetch(url, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Payment[]>(response);
}

export async function fetchPayment(id: string): Promise<Payment> {
  const response = await fetch(`${API_BASE}/payments/${id}`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Payment>(response);
}

export async function createPayment(payment: PaymentCreate): Promise<Payment> {
  const response = await fetch(`${API_BASE}/payments`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(payment),
  });
  return handleResponse<Payment>(response);
}

export async function updatePayment(
  id: string,
  payment: PaymentUpdate
): Promise<Payment> {
  const response = await fetch(`${API_BASE}/payments/${id}`, {
    method: "PATCH",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(payment),
  });
  return handleResponse<Payment>(response);
}

export async function postPayment(id: string): Promise<Payment> {
  const response = await fetch(`${API_BASE}/payments/${id}/post`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  });
  return handleResponse<Payment>(response);
}

export async function cancelPayment(id: string): Promise<Payment> {
  const response = await fetch(`${API_BASE}/payments/${id}/cancel`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  });
  return handleResponse<Payment>(response);
}

export interface AllocableInvoice {
  id: string;
  invoice_number: string;
  issue_date: string;
  due_date?: string;
  status: string;
  total_amount: number;
  currency: string;
}

export interface AllocableBill {
  id: string;
  bill_number: string;
  issue_date: string;
  due_date?: string;
  status: string;
  total_amount: number;
  currency: string;
}

export async function fetchPartyOpenInvoices(
  partyId: string
): Promise<AllocableInvoice[]> {
  const params = new URLSearchParams();
  params.set("party_id", partyId);
  params.set("status", "posted");
  const response = await fetch(`${API_BASE}/invoices?${params.toString()}`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<AllocableInvoice[]>(response);
}

export async function fetchPartyOpenBills(
  partyId: string
): Promise<AllocableBill[]> {
  const params = new URLSearchParams();
  params.set("party_id", partyId);
  params.set("status", "open");
  const response = await fetch(`${API_BASE}/bills?${params.toString()}`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<AllocableBill[]>(response);
}

export function usePayments(
  search?: string,
  options?: Omit<UseQueryOptions<Payment[]>, "queryKey" | "queryFn">
) {
  return useQuery<Payment[]>({
    queryKey: [PAYMENTS_KEY, { search }],
    queryFn: () => fetchPayments(search),
    ...options,
  });
}

export function usePayment(id: string | null) {
  return useQuery<Payment>({
    queryKey: [PAYMENT_KEY, id],
    queryFn: () => fetchPayment(id as string),
    enabled: !!id,
  });
}

export function useCreatePayment() {
  const queryClient = useQueryClient();
  return useMutation<Payment, Error, PaymentCreate>({
    mutationFn: createPayment,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [PAYMENTS_KEY] });
      queryClient.invalidateQueries({ queryKey: ["invoices"] });
      queryClient.invalidateQueries({ queryKey: ["bills"] });
    },
  });
}

export function useUpdatePayment() {
  const queryClient = useQueryClient();
  return useMutation<Payment, Error, { id: string; data: PaymentUpdate }>({
    mutationFn: ({ id, data }) => updatePayment(id, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [PAYMENTS_KEY] });
      queryClient.invalidateQueries({ queryKey: [PAYMENT_KEY, variables.id] });
      queryClient.invalidateQueries({ queryKey: ["invoices"] });
      queryClient.invalidateQueries({ queryKey: ["bills"] });
    },
  });
}

export function usePostPayment() {
  const queryClient = useQueryClient();
  return useMutation<Payment, Error, string>({
    mutationFn: postPayment,
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: [PAYMENTS_KEY] });
      queryClient.invalidateQueries({ queryKey: [PAYMENT_KEY, id] });
      queryClient.invalidateQueries({ queryKey: ["invoices"] });
      queryClient.invalidateQueries({ queryKey: ["bills"] });
    },
  });
}

export function useCancelPayment() {
  const queryClient = useQueryClient();
  return useMutation<Payment, Error, string>({
    mutationFn: cancelPayment,
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: [PAYMENTS_KEY] });
      queryClient.invalidateQueries({ queryKey: [PAYMENT_KEY, id] });
      queryClient.invalidateQueries({ queryKey: ["invoices"] });
      queryClient.invalidateQueries({ queryKey: ["bills"] });
    },
  });
}

export function usePartyOpenInvoices(partyId: string | null) {
  return useQuery<AllocableInvoice[]>({
    queryKey: ["invoices", "open", partyId],
    queryFn: () => fetchPartyOpenInvoices(partyId as string),
    enabled: !!partyId,
  });
}

export function usePartyOpenBills(partyId: string | null) {
  return useQuery<AllocableBill[]>({
    queryKey: ["bills", "open", partyId],
    queryFn: () => fetchPartyOpenBills(partyId as string),
    enabled: !!partyId,
  });
}
