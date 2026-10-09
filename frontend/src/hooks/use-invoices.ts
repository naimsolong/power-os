import {
  useQuery,
  useMutation,
  useQueryClient,
  type UseQueryOptions,
} from "@tanstack/react-query";
import {
  type Invoice,
  type InvoiceCreate,
  type InvoiceUpdate,
  type InvoiceLine,
  type InvoiceLineCreate,
  type InvoiceLineUpdate,
  type LhdnSubmission,
  fetchInvoices,
  fetchInvoice,
  createInvoice,
  updateInvoice,
  deleteInvoice,
  postInvoice,
  fetchInvoiceLines,
  createInvoiceLine,
  updateInvoiceLine,
  deleteInvoiceLine,
  submitInvoiceToLhdn,
  fetchInvoiceLhdnStatus,
} from "@/lib/api";

export type InvoiceWithCurrency = Invoice & { exchange_rate: number };

export type InvoiceCreateRequest = InvoiceCreate & {
  status: Invoice["status"];
  exchange_rate: number;
};

const INVOICES_KEY = "invoices";
const INVOICE_KEY = "invoice";
const INVOICE_LINES_KEY = "invoice-lines";
const LHDN_STATUS_KEY = "lhdn-status";

export function useInvoices(
  search?: string,
  options?: Omit<UseQueryOptions<Invoice[]>, "queryKey" | "queryFn">
) {
  return useQuery<Invoice[]>({
    queryKey: [INVOICES_KEY, { search }],
    queryFn: () => fetchInvoices(search),
    ...options,
  });
}

export function useInvoice(id: string | null) {
  return useQuery<Invoice>({
    queryKey: [INVOICE_KEY, id],
    queryFn: () => fetchInvoice(id as string),
    enabled: !!id,
  });
}

export function useCreateInvoice() {
  const queryClient = useQueryClient();
  return useMutation<Invoice, Error, InvoiceCreate>({
    mutationFn: createInvoice,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [INVOICES_KEY] });
    },
  });
}

export function useUpdateInvoice() {
  const queryClient = useQueryClient();
  return useMutation<Invoice, Error, { id: string; data: InvoiceUpdate }>({
    mutationFn: ({ id, data }) => updateInvoice(id, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [INVOICES_KEY] });
      queryClient.invalidateQueries({
        queryKey: [INVOICE_KEY, variables.id],
      });
    },
  });
}

export function useDeleteInvoice() {
  const queryClient = useQueryClient();
  return useMutation<void, Error, string>({
    mutationFn: deleteInvoice,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [INVOICES_KEY] });
    },
  });
}

export function usePostInvoice() {
  const queryClient = useQueryClient();
  return useMutation<Invoice, Error, string>({
    mutationFn: postInvoice,
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: [INVOICES_KEY] });
      queryClient.invalidateQueries({ queryKey: [INVOICE_KEY, id] });
    },
  });
}

export function useInvoiceLines(id: string | null) {
  return useQuery<InvoiceLine[]>({
    queryKey: [INVOICE_LINES_KEY, id],
    queryFn: () => fetchInvoiceLines(id as string),
    enabled: !!id,
  });
}

export function useCreateInvoiceLine() {
  const queryClient = useQueryClient();
  return useMutation<InvoiceLine, Error, { id: string; data: InvoiceLineCreate }>({
    mutationFn: ({ id, data }) => createInvoiceLine(id, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: [INVOICE_LINES_KEY, variables.id],
      });
      queryClient.invalidateQueries({
        queryKey: [INVOICE_KEY, variables.id],
      });
      queryClient.invalidateQueries({ queryKey: [INVOICES_KEY] });
    },
  });
}

export function useUpdateInvoiceLine() {
  const queryClient = useQueryClient();
  return useMutation<
    InvoiceLine,
    Error,
    { id: string; lineId: string; data: InvoiceLineUpdate }
  >({
    mutationFn: ({ id, lineId, data }) => updateInvoiceLine(id, lineId, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: [INVOICE_LINES_KEY, variables.id],
      });
      queryClient.invalidateQueries({
        queryKey: [INVOICE_KEY, variables.id],
      });
      queryClient.invalidateQueries({ queryKey: [INVOICES_KEY] });
    },
  });
}

export function useDeleteInvoiceLine() {
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: string; lineId: string }>({
    mutationFn: ({ id, lineId }) => deleteInvoiceLine(id, lineId),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: [INVOICE_LINES_KEY, variables.id],
      });
      queryClient.invalidateQueries({
        queryKey: [INVOICE_KEY, variables.id],
      });
      queryClient.invalidateQueries({ queryKey: [INVOICES_KEY] });
    },
  });
}

export function useSubmitLhdn() {
  const queryClient = useQueryClient();
  return useMutation<LhdnSubmission, Error, string>({
    mutationFn: submitInvoiceToLhdn,
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: [INVOICES_KEY] });
      queryClient.invalidateQueries({ queryKey: [INVOICE_KEY, id] });
      queryClient.invalidateQueries({ queryKey: [LHDN_STATUS_KEY, id] });
    },
  });
}

export function useLhdnStatus(id: string | null) {
  return useQuery<LhdnSubmission>({
    queryKey: [LHDN_STATUS_KEY, id],
    queryFn: () => fetchInvoiceLhdnStatus(id as string),
    enabled: !!id,
  });
}
