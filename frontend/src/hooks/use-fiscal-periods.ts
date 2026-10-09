import {
  useQuery,
  useMutation,
  useQueryClient,
  type UseQueryOptions,
} from "@tanstack/react-query";

const API_BASE = "/api";

export interface AccountingPeriod {
  id: string;
  workspace_id: string;
  fiscal_year_id: string;
  name: string;
  start_date: string;
  end_date: string;
  is_closed: boolean;
  closed_at: string | null;
  closed_by_user_id: string | null;
}

export interface FiscalYear {
  id: string;
  workspace_id: string;
  name: string;
  start_date: string;
  end_date: string;
  is_closed: boolean;
  periods: AccountingPeriod[];
}

export interface FiscalYearCreate {
  name: string;
  start_date: string;
  end_date: string;
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
  return response.json() as Promise<T>;
}

export async function fetchFiscalYears(): Promise<FiscalYear[]> {
  const response = await fetch(`${API_BASE}/fiscal-years`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<FiscalYear[]>(response);
}

export async function createFiscalYear(
  fiscalYear: FiscalYearCreate
): Promise<FiscalYear> {
  const response = await fetch(`${API_BASE}/fiscal-years`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(fiscalYear),
  });
  return handleResponse<FiscalYear>(response);
}

export async function closeAccountingPeriod(
  id: string
): Promise<AccountingPeriod> {
  const response = await fetch(`${API_BASE}/accounting-periods/${id}/close`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  });
  return handleResponse<AccountingPeriod>(response);
}

export async function reopenAccountingPeriod(
  id: string
): Promise<AccountingPeriod> {
  const response = await fetch(`${API_BASE}/accounting-periods/${id}/reopen`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  });
  return handleResponse<AccountingPeriod>(response);
}

const FISCAL_YEARS_KEY = "fiscal-years";

export function useFiscalYears(
  options?: Omit<UseQueryOptions<FiscalYear[]>, "queryKey" | "queryFn">
) {
  return useQuery<FiscalYear[]>({
    queryKey: [FISCAL_YEARS_KEY],
    queryFn: fetchFiscalYears,
    ...options,
  });
}

export function useCreateFiscalYear() {
  const queryClient = useQueryClient();
  return useMutation<FiscalYear, Error, FiscalYearCreate>({
    mutationFn: createFiscalYear,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [FISCAL_YEARS_KEY] });
    },
  });
}

export function useCloseAccountingPeriod() {
  const queryClient = useQueryClient();
  return useMutation<AccountingPeriod, Error, string>({
    mutationFn: closeAccountingPeriod,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [FISCAL_YEARS_KEY] });
    },
  });
}

export function useReopenAccountingPeriod() {
  const queryClient = useQueryClient();
  return useMutation<AccountingPeriod, Error, string>({
    mutationFn: reopenAccountingPeriod,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [FISCAL_YEARS_KEY] });
    },
  });
}
