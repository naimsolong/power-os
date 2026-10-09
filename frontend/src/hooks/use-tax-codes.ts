import {
  useQuery,
  useMutation,
  useQueryClient,
  type UseQueryOptions,
} from "@tanstack/react-query";

const API_BASE = "/api";
const TAX_CODES_KEY = "tax-codes";
const TAX_CODE_KEY = "tax-code";

export type TaxType = "sst" | "service_tax";

export const TAX_TYPE_LABELS: Record<TaxType, string> = {
  sst: "SST",
  service_tax: "Service Tax",
};

export const TAX_TYPE_OPTIONS: TaxType[] = ["sst", "service_tax"];

export interface TaxCode {
  id: string;
  workspace_id: string;
  code: string;
  description: string;
  rate: number;
  tax_type: TaxType;
  is_active: boolean;
}

export interface TaxCodeCreate {
  code: string;
  description: string;
  rate: number;
  tax_type: TaxType;
}

export type TaxCodeUpdate = Partial<TaxCodeCreate> & {
  is_active?: boolean;
};

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
  return response.json() as Promise<T>;
}

export async function fetchTaxCodes(): Promise<TaxCode[]> {
  const response = await fetch(`${API_BASE}/tax-codes`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<TaxCode[]>(response);
}

export async function fetchTaxCode(id: string): Promise<TaxCode> {
  const response = await fetch(`${API_BASE}/tax-codes/${id}`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<TaxCode>(response);
}

export async function createTaxCode(taxCode: TaxCodeCreate): Promise<TaxCode> {
  const response = await fetch(`${API_BASE}/tax-codes`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(taxCode),
  });
  return handleResponse<TaxCode>(response);
}

export async function updateTaxCode(
  id: string,
  taxCode: TaxCodeUpdate
): Promise<TaxCode> {
  const response = await fetch(`${API_BASE}/tax-codes/${id}`, {
    method: "PATCH",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(taxCode),
  });
  return handleResponse<TaxCode>(response);
}

export async function deleteTaxCode(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/tax-codes/${id}`, {
    method: "DELETE",
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
}

export function useTaxCodes(
  options?: Omit<UseQueryOptions<TaxCode[]>, "queryKey" | "queryFn">
) {
  return useQuery<TaxCode[]>({
    queryKey: [TAX_CODES_KEY],
    queryFn: fetchTaxCodes,
    ...options,
  });
}

export function useTaxCode(id: string | null) {
  return useQuery<TaxCode>({
    queryKey: [TAX_CODE_KEY, id],
    queryFn: () => fetchTaxCode(id as string),
    enabled: !!id,
  });
}

export function useCreateTaxCode() {
  const queryClient = useQueryClient();
  return useMutation<TaxCode, Error, TaxCodeCreate>({
    mutationFn: createTaxCode,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [TAX_CODES_KEY] });
    },
  });
}

export function useUpdateTaxCode() {
  const queryClient = useQueryClient();
  return useMutation<TaxCode, Error, { id: string; data: TaxCodeUpdate }>({
    mutationFn: ({ id, data }) => updateTaxCode(id, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [TAX_CODES_KEY] });
      queryClient.invalidateQueries({
        queryKey: [TAX_CODE_KEY, variables.id],
      });
    },
  });
}

export function useDeleteTaxCode() {
  const queryClient = useQueryClient();
  return useMutation<void, Error, string>({
    mutationFn: deleteTaxCode,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [TAX_CODES_KEY] });
    },
  });
}
