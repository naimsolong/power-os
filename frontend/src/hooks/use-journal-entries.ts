import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseQueryOptions,
} from "@tanstack/react-query";

const API_BASE = "/api";
const JOURNAL_ENTRIES_KEY = "journal-entries";
const JOURNAL_ENTRY_KEY = "journal-entry";

export type JournalEntryStatus = "draft" | "posted" | "cancelled";

export const JOURNAL_ENTRY_STATUS_LABELS: Record<JournalEntryStatus, string> = {
  draft: "Draft",
  posted: "Posted",
  cancelled: "Cancelled",
};

export const JOURNAL_ENTRY_STATUS_OPTIONS: JournalEntryStatus[] = [
  "draft",
  "posted",
  "cancelled",
];

export interface JournalLine {
  id: string;
  journal_entry_id: string;
  account_id: string;
  party_id: string | null;
  description: string | null;
  debit: string;
  credit: string;
}

export interface JournalEntry {
  id: string;
  workspace_id: string;
  entry_date: string;
  reference: string | null;
  description: string | null;
  status: JournalEntryStatus;
  lines: JournalLine[];
}

export interface JournalEntryCancelResponse {
  original_entry: JournalEntry;
  reversing_entry: JournalEntry;
}

export interface JournalLineCreate {
  account_id: string;
  party_id?: string | null;
  description?: string | null;
  debit?: string | null;
  credit?: string | null;
}

export interface JournalEntryCreate {
  entry_date: string;
  reference?: string | null;
  description?: string | null;
  status?: JournalEntryStatus;
  lines: JournalLineCreate[];
}

export type JournalEntryUpdate = Partial<JournalEntryCreate>;

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
  return response.json() as Promise<T>;
}

export async function fetchJournalEntries(
  from?: string,
  to?: string,
  status?: JournalEntryStatus
): Promise<JournalEntry[]> {
  const params = new URLSearchParams();
  if (from) params.set("from", from);
  if (to) params.set("to", to);
  if (status) params.set("status", status);
  const query = params.toString();
  const url = `${API_BASE}/journal-entries${query ? `?${query}` : ""}`;

  const response = await fetch(url, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<JournalEntry[]>(response);
}

export async function fetchJournalEntry(id: string): Promise<JournalEntry> {
  const response = await fetch(`${API_BASE}/journal-entries/${id}`, {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return handleResponse<JournalEntry>(response);
}

export async function createJournalEntry(
  entry: JournalEntryCreate
): Promise<JournalEntry> {
  const response = await fetch(`${API_BASE}/journal-entries`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(entry),
  });
  return handleResponse<JournalEntry>(response);
}

export async function updateJournalEntry(
  id: string,
  entry: JournalEntryUpdate
): Promise<JournalEntry> {
  const response = await fetch(`${API_BASE}/journal-entries/${id}`, {
    method: "PATCH",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify(entry),
  });
  return handleResponse<JournalEntry>(response);
}

export async function deleteJournalEntry(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/journal-entries/${id}`, {
    method: "DELETE",
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  if (!response.ok) {
    const text = await response.text().catch(() => "Unknown error");
    throw new Error(`HTTP ${response.status}: ${text}`);
  }
}

export async function postJournalEntry(id: string): Promise<JournalEntry> {
  const response = await fetch(`${API_BASE}/journal-entries/${id}/post`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  });
  return handleResponse<JournalEntry>(response);
}

export async function cancelJournalEntry(
  id: string
): Promise<JournalEntryCancelResponse> {
  const response = await fetch(`${API_BASE}/journal-entries/${id}/cancel`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
  });
  return handleResponse<JournalEntryCancelResponse>(response);
}

export function useJournalEntries(
  from?: string,
  to?: string,
  status?: JournalEntryStatus,
  options?: Omit<UseQueryOptions<JournalEntry[]>, "queryKey" | "queryFn">
) {
  return useQuery<JournalEntry[]>({
    queryKey: [JOURNAL_ENTRIES_KEY, { from, to, status }],
    queryFn: () => fetchJournalEntries(from, to, status),
    ...options,
  });
}

export function useJournalEntry(id: string | null) {
  return useQuery<JournalEntry>({
    queryKey: [JOURNAL_ENTRY_KEY, id],
    queryFn: () => fetchJournalEntry(id as string),
    enabled: id !== null,
  });
}

export function useCreateJournalEntry() {
  const queryClient = useQueryClient();
  return useMutation<JournalEntry, Error, JournalEntryCreate>({
    mutationFn: createJournalEntry,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [JOURNAL_ENTRIES_KEY] });
    },
  });
}

export function useUpdateJournalEntry() {
  const queryClient = useQueryClient();
  return useMutation<JournalEntry, Error, { id: string; data: JournalEntryUpdate }>({
    mutationFn: ({ id, data }) => updateJournalEntry(id, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [JOURNAL_ENTRIES_KEY] });
      queryClient.invalidateQueries({
        queryKey: [JOURNAL_ENTRY_KEY, variables.id],
      });
    },
  });
}

export function useDeleteJournalEntry() {
  const queryClient = useQueryClient();
  return useMutation<void, Error, string>({
    mutationFn: deleteJournalEntry,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [JOURNAL_ENTRIES_KEY] });
    },
  });
}

export function usePostJournalEntry() {
  const queryClient = useQueryClient();
  return useMutation<JournalEntry, Error, string>({
    mutationFn: postJournalEntry,
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: [JOURNAL_ENTRIES_KEY] });
      queryClient.invalidateQueries({ queryKey: [JOURNAL_ENTRY_KEY, id] });
    },
  });
}

export function useCancelJournalEntry() {
  const queryClient = useQueryClient();
  return useMutation<JournalEntryCancelResponse, Error, string>({
    mutationFn: cancelJournalEntry,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [JOURNAL_ENTRIES_KEY] });
    },
  });
}
