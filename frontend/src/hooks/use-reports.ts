import {
  useQuery,
  type UseQueryOptions,
} from "@tanstack/react-query";

const API_BASE = "/api";
const REPORTS_KEY = "reports";

export interface TrialBalanceLine {
  account_id: string;
  account_code: string;
  account_name: string;
  account_type: string;
  debit: string;
  credit: string;
  balance: string;
}

export interface TrialBalanceReport {
  as_of: string;
  lines: TrialBalanceLine[];
}

export interface ProfitLossReport {
  from: string;
  to: string;
  revenue: string;
  expenses: string;
  net_profit: string;
}

export interface BalanceSheetReport {
  as_of: string;
  assets: string;
  liabilities: string;
  equity: string;
  retained_earnings: string;
  check: string;
}

export interface TrialBalanceParams {
  as_of: string;
}

export interface ProfitLossParams {
  from: string;
  to: string;
}

export interface BalanceSheetParams {
  as_of: string;
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
  return response.json() as Promise<T>;
}

export async function fetchTrialBalance(
  params: TrialBalanceParams
): Promise<TrialBalanceReport> {
  const searchParams = new URLSearchParams();
  searchParams.set("as_of", params.as_of);
  const response = await fetch(
    `${API_BASE}/reports/trial-balance?${searchParams.toString()}`,
    {
      credentials: "include",
      headers: { Accept: "application/json" },
    }
  );
  return handleResponse<TrialBalanceReport>(response);
}

export async function fetchProfitLoss(
  params: ProfitLossParams
): Promise<ProfitLossReport> {
  const searchParams = new URLSearchParams();
  searchParams.set("from", params.from);
  searchParams.set("to", params.to);
  const response = await fetch(
    `${API_BASE}/reports/profit-loss?${searchParams.toString()}`,
    {
      credentials: "include",
      headers: { Accept: "application/json" },
    }
  );
  return handleResponse<ProfitLossReport>(response);
}

export async function fetchBalanceSheet(
  params: BalanceSheetParams
): Promise<BalanceSheetReport> {
  const searchParams = new URLSearchParams();
  searchParams.set("as_of", params.as_of);
  const response = await fetch(
    `${API_BASE}/reports/balance-sheet?${searchParams.toString()}`,
    {
      credentials: "include",
      headers: { Accept: "application/json" },
    }
  );
  return handleResponse<BalanceSheetReport>(response);
}

export function useTrialBalance(
  params: TrialBalanceParams | null,
  options?: Omit<UseQueryOptions<TrialBalanceReport>, "queryKey" | "queryFn">
) {
  return useQuery<TrialBalanceReport>({
    queryKey: [REPORTS_KEY, "trial-balance", params],
    queryFn: () => fetchTrialBalance(params as TrialBalanceParams),
    enabled: !!params && !!params.as_of,
    ...options,
  });
}

export function useProfitLoss(
  params: ProfitLossParams | null,
  options?: Omit<UseQueryOptions<ProfitLossReport>, "queryKey" | "queryFn">
) {
  return useQuery<ProfitLossReport>({
    queryKey: [REPORTS_KEY, "profit-loss", params],
    queryFn: () => fetchProfitLoss(params as ProfitLossParams),
    enabled: !!params && !!params.from && !!params.to,
    ...options,
  });
}

export function useBalanceSheet(
  params: BalanceSheetParams | null,
  options?: Omit<UseQueryOptions<BalanceSheetReport>, "queryKey" | "queryFn">
) {
  return useQuery<BalanceSheetReport>({
    queryKey: [REPORTS_KEY, "balance-sheet", params],
    queryFn: () => fetchBalanceSheet(params as BalanceSheetParams),
    enabled: !!params && !!params.as_of,
    ...options,
  });
}
