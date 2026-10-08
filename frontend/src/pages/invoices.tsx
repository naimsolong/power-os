import { useMemo, useState } from "react";
import { Plus, Search, Send, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { InvoiceDrawer } from "@/components/invoices/invoice-drawer";
import { useParties } from "@/hooks/use-parties";
import {
  useInvoices,
  useInvoice,
  useCreateInvoice,
  useUpdateInvoice,
  useDeleteInvoice,
  usePostInvoice,
  useInvoiceLines,
  useSubmitLhdn,
  useLhdnStatus,
} from "@/hooks/use-invoices";
import {
  INVOICE_STATUS_LABELS,
  type InvoiceCreate,
  type InvoiceLineCreate,
  deleteInvoiceLine,
  createInvoiceLine,
} from "@/lib/api";
import type { InvoiceLineForm } from "@/components/invoices/invoice-drawer";

export function InvoicesPage() {
  const [search, setSearch] = useState("");
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [selectedInvoiceId, setSelectedInvoiceId] = useState<string | null>(
    null
  );

  const { data: invoices, isLoading, error } = useInvoices(search);
  const { data: selectedInvoice, isLoading: isLoadingInvoice } = useInvoice(
    selectedInvoiceId
  );
  const { data: selectedLines, isLoading: isLoadingLines } = useInvoiceLines(
    selectedInvoiceId
  );
  const { data: parties } = useParties("customer");

  const createInvoice = useCreateInvoice();
  const updateInvoice = useUpdateInvoice();
  const deleteInvoice = useDeleteInvoice();
  const postInvoice = usePostInvoice();
  const submitLhdn = useSubmitLhdn();
  const { data: selectedLhdnStatus } = useLhdnStatus(selectedInvoiceId);

  const mutationError =
    createInvoice.error ||
    updateInvoice.error ||
    deleteInvoice.error ||
    postInvoice.error ||
    submitLhdn.error;

  const partyById = useMemo(() => {
    const map = new Map<string, string>();
    if (!parties) return map;
    for (const party of parties) {
      map.set(party.id, party.name);
    }
    return map;
  }, [parties]);

  const handleNew = () => {
    setSelectedInvoiceId(null);
    setDrawerOpen(true);
  };

  const handleEdit = (id: string) => {
    setSelectedInvoiceId(id);
    setDrawerOpen(true);
  };

  const handleCloseDrawer = () => {
    setDrawerOpen(false);
    setSelectedInvoiceId(null);
    createInvoice.reset();
    updateInvoice.reset();
    deleteInvoice.reset();
    postInvoice.reset();
  };

  const saveLines = async (invoiceId: string, lines: InvoiceLineForm[]) => {
    if (lines.length === 0) return;
    const payload: InvoiceLineCreate[] = lines.map((line) => ({
      description: line.description,
      quantity: line.quantity,
      unit_price: line.unit_price,
      line_total: line.line_total,
    }));
    await Promise.all(
      payload.map((line) => createInvoiceLine(invoiceId, line))
    );
  };

  const handleSave = async (payload: {
    form: InvoiceCreate;
    lines: InvoiceLineForm[];
  }) => {
    try {
      if (selectedInvoiceId) {
        await updateInvoice.mutateAsync({
          id: selectedInvoiceId,
          data: payload.form,
        });
        if (selectedLines && selectedLines.length > 0) {
          await Promise.all(
            selectedLines.map((line) =>
              deleteInvoiceLine(selectedInvoiceId, line.id)
            )
          );
        }
        await saveLines(selectedInvoiceId, payload.lines);
      } else {
        const created = await createInvoice.mutateAsync(payload.form);
        await saveLines(created.id, payload.lines);
      }
      handleCloseDrawer();
    } catch {
      // Errors are surfaced by mutation state.
    }
  };

  const handleDelete = (id: string) => {
    if (confirm("Are you sure you want to delete this invoice?")) {
      deleteInvoice.mutate(id, { onSuccess: handleCloseDrawer });
    }
  };

  const handlePost = (id: string) => {
    if (confirm("Post this draft invoice?")) {
      postInvoice.mutate(id);
    }
  };

  const handleSubmitLhdn = (id: string) => {
    if (confirm("Submit this invoice to LHDN MyInvois?")) {
      submitLhdn.mutate(id);
    }
  };

  const formatCurrency = (value: number, currency: string) => {
    try {
      return new Intl.NumberFormat("en-US", {
        style: "currency",
        currency: currency || "USD",
      }).format(value);
    } catch {
      return `$${value.toLocaleString()}`;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Invoices</h2>
          <p className="text-muted-foreground">Manage billing and payments.</p>
        </div>
        <Button onClick={handleNew}>
          <Plus className="h-4 w-4" />
          Create Invoice
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Search</CardTitle>
          <CardDescription>Find invoices by invoice number.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Search className="h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search invoices..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="max-w-md"
            />
          </div>
        </CardContent>
      </Card>

      {mutationError && (
        <div className="rounded-md border border-destructive bg-destructive/10 p-3 text-sm text-destructive">
          {mutationError.message}
        </div>
      )}

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Number</TableHead>
              <TableHead>Client</TableHead>
              <TableHead>Amount</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>LHDN Status</TableHead>
              <TableHead>Due Date</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell
                  colSpan={7}
                  className="h-24 text-center text-muted-foreground"
                >
                  Loading...
                </TableCell>
              </TableRow>
            ) : error ? (
              <TableRow>
                <TableCell
                  colSpan={7}
                  className="h-24 text-center text-destructive"
                >
                  Failed to load invoices: {error.message}
                </TableCell>
              </TableRow>
            ) : !invoices || invoices.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={7}
                  className="h-24 text-center text-muted-foreground"
                >
                  No invoices found.
                </TableCell>
              </TableRow>
            ) : (
              invoices.map((invoice) => (
                <TableRow
                  key={invoice.id}
                  className="cursor-pointer"
                  onClick={() => handleEdit(invoice.id)}
                >
                  <TableCell className="font-medium">
                    {invoice.invoice_number}
                  </TableCell>
                  <TableCell>
                    {partyById.get(invoice.party_id) ?? invoice.party_id}
                  </TableCell>
                  <TableCell>
                    {formatCurrency(
                      invoice.total_amount,
                      invoice.currency
                    )}
                  </TableCell>
                  <TableCell className="capitalize">
                    {INVOICE_STATUS_LABELS[invoice.status]}
                  </TableCell>
                  <TableCell className="capitalize">
                    {invoice.lhdn_status ?? "—"}
                  </TableCell>
                  <TableCell>
                    {invoice.due_date
                      ? invoice.due_date.slice(0, 10)
                      : "—"}
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-2">
                      {invoice.status === "draft" && (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            handlePost(invoice.id);
                          }}
                        >
                          <Send className="h-4 w-4" />
                          Post
                        </Button>
                      )}
                      {(invoice.status === "draft" || invoice.status === "posted") && (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleSubmitLhdn(invoice.id);
                          }}
                        >
                          <Upload className="h-4 w-4" />
                          Submit LHDN
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <InvoiceDrawer
        isOpen={drawerOpen}
        invoiceId={selectedInvoiceId}
        invoice={selectedInvoice}
        lines={selectedLines}
        parties={parties ?? []}
        lhdnStatus={selectedLhdnStatus}
        isLoading={!!selectedInvoiceId && (isLoadingInvoice || isLoadingLines)}
        isSaving={
          createInvoice.isPending ||
          updateInvoice.isPending ||
          deleteInvoice.isPending
        }
        isSubmittingLhdn={submitLhdn.isPending}
        error={mutationError}
        onClose={handleCloseDrawer}
        onSave={handleSave}
        onDelete={handleDelete}
        onSubmitLhdn={handleSubmitLhdn}
      />
    </div>
  );
}
