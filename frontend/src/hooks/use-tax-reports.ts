import {
  useQuery,
  type UseQueryOptions,
} from "@tanstack/react-query";

const API_BASE = "/api";
const TAX_REPORTS_KEY = "tax-reports";

export interface Sst02ReportLine {
  tax_code_id: string;
  code: string;
  description: string;
  rate: string;
  taxable_amount: string;
  tax_amount: string;
}

export interface Sst02Report {
  from: string;
  to: string;
  total_taxable_amount: string;
  total_tax_amount: string;
  lines: Sst02ReportLine[];
}

export interface Sst02ReportParams {
  from: string;
  to: string;
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
  return response.json() as Promise<T>;
}

export async function fetchSst02Report(params: Sst02ReportParams): Promise<Sst02Report> {
  const searchParams = new URLSearchParams();
  searchParams.set("from", params.from);
  searchParams.set("to", params.to);
  const response = await fetch(`${API_BASE}/tax-reports/sst-02?${searchParams.toString()}`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<Sst02Report>(response);
}

export function useSst02Report(
  params: Sst02ReportParams | null,
  options?: Omit<UseQueryOptions<Sst02Report>, "queryKey" | "queryFn">
) {
  return useQuery<Sst02Report>({
    queryKey: [TAX_REPORTS_KEY, "sst-02", params],
    queryFn: () => fetchSst02Report(params as Sst02ReportParams),
    enabled: !!params && !!params.from && !!params.to,
    ...options,
  });
}
