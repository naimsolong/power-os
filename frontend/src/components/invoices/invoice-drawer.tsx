import { useMemo, useState } from "react";
import { Plus, Trash2, Upload, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  type InvoiceLine,
  type InvoiceStatus,
  type LhdnSubmission,
  type Party,
  INVOICE_STATUS_LABELS,
  INVOICE_STATUS_OPTIONS,
} from "@/lib/api";
import {
  type InvoiceWithCurrency,
  type InvoiceCreateRequest,
} from "@/hooks/use-invoices";
import { type TaxCode } from "@/hooks/use-tax-codes";

import { cn } from "@/lib/utils";

const SUPPORTED_CURRENCIES = ["MYR", "USD", "EUR", "SGD"];

export interface InvoiceLineForm {
  id?: string;
  description: string;
  quantity: number;
  unit_price: number;
  line_total: number;
  tax_code_id: string | null;
  tax_amount: number;
}

interface InvoiceDrawerProps {
  isOpen: boolean;
  invoiceId: string | null;
  invoice?: InvoiceWithCurrency | null;
  lines?: InvoiceLine[] | null;
  parties: Party[];
  taxCodes: TaxCode[];
  lhdnStatus?: LhdnSubmission | null;
  isLoading?: boolean;
  isSaving?: boolean;
  isSubmittingLhdn?: boolean;
  error?: Error | null;
  onClose: () => void;
  onSave: (payload: { form: InvoiceCreateRequest; lines: InvoiceLineForm[] }) => void;
  onDelete?: (id: string) => void;
  onSubmitLhdn?: (id: string) => void;
}

const emptyForm: InvoiceCreateRequest = {
  party_id: "",
  invoice_number: "",
  issue_date: "",
  due_date: "",
  status: "draft",
  currency: "MYR",
  exchange_rate: 1,
};

function buildInitialForm(
  invoice: InvoiceWithCurrency | null | undefined
): InvoiceCreateRequest {
  if (!invoice) return { ...emptyForm };
  return {
    party_id: invoice.party_id,
    invoice_number: invoice.invoice_number,
    issue_date: invoice.issue_date.slice(0, 10),
    due_date: invoice.due_date.slice(0, 10),
    status: invoice.status,
    currency: invoice.currency,
    exchange_rate: invoice.exchange_rate,
  };
}

function emptyLine(): InvoiceLineForm {
  return {
    description: "",
    quantity: 1,
    unit_price: 0,
    line_total: 0,
    tax_code_id: null,
    tax_amount: 0,
  };
}

function buildInitialLines(
  lines: InvoiceLine[] | null | undefined
): InvoiceLineForm[] {
  if (!lines || lines.length === 0) return [emptyLine()];
  return lines.map((line) => ({
    id: line.id,
    description: line.description,
    quantity: line.quantity,
    unit_price: line.unit_price,
    line_total: line.line_total,
    tax_code_id: line.tax_code_id,
    tax_amount: line.tax_amount,
  }));
}

function computeLineTotal(quantity: number, unitPrice: number): number {
  return Math.round(quantity * unitPrice * 100) / 100;
}

function buildDrawerKey(
  invoice: InvoiceWithCurrency | null | undefined,
  invoiceId: string | null,
  lines: InvoiceLine[] | null | undefined
): string {
  const identity = invoice ? invoice.id : invoiceId ?? "new";
  return `${identity}-${lines?.length ?? "null"}`;
}

function formatCurrency(value: number, currency: string): string {
  try {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: currency || "MYR",
    }).format(value);
  } catch {
    return `${currency || "MYR"} ${value.toLocaleString()}`;
  }
}

export function InvoiceDrawer({
  isOpen,
  invoiceId,
  invoice,
  lines,
  parties,
  taxCodes,
  lhdnStatus,
  isLoading,
  isSaving,
  isSubmittingLhdn,
  error,
  onClose,
  onSave,
  onDelete,
  onSubmitLhdn,
}: InvoiceDrawerProps) {
  const taxCodeById = useMemo(() => {
    const map = new Map<string, TaxCode>();
    for (const taxCode of taxCodes) {
      map.set(taxCode.id, taxCode);
    }
    return map;
  }, [taxCodes]);

  const [form, setForm] = useState<InvoiceCreateRequest>(() =>
    buildInitialForm(invoice)
  );
  const [lineItems, setLineItems] = useState<InvoiceLineForm[]>(() =>
    buildInitialLines(lines)
  );
  const [lastKey, setLastKey] = useState<string>(() =>
    buildDrawerKey(invoice, invoiceId, lines)
  );

  const currentKey = buildDrawerKey(invoice, invoiceId, lines);
  if (currentKey !== lastKey) {
    setLastKey(currentKey);
    setForm(buildInitialForm(invoice));
    setLineItems(buildInitialLines(lines));
  }

  const currency = form.currency ?? "MYR";
  const exchangeRate = form.exchange_rate ?? 1;

  const totalForeignAmount = lineItems.reduce(
    (sum, line) => sum + line.quantity * line.unit_price,
    0
  );
  const totalTaxAmount = lineItems.reduce((sum, line) => sum + line.tax_amount, 0);
  const totalForeignAmountWithTax = totalForeignAmount + totalTaxAmount;
  const totalMyrAmount = totalForeignAmount * exchangeRate;
  const totalMyrAmountWithTax = totalMyrAmount + totalTaxAmount;

  const computeTaxAmount = (line: InvoiceLineForm): number => {
    if (!line.tax_code_id) return 0;
    const taxCode = taxCodeById.get(line.tax_code_id);
    if (!taxCode) return 0;
    return Math.round(line.line_total * taxCode.rate * 100) / 100;
  };

  const updateLine = (
    index: number,
    field: "description" | "quantity" | "unit_price" | "tax_code_id",
    value: string | number
  ) => {
    setLineItems((prev) => {
      const next = [...prev];
      const line = { ...next[index] };

      if (field === "quantity" || field === "unit_price") {
        const numeric = typeof value === "number" ? value : Number(value);
        line[field] = Number.isNaN(numeric) ? 0 : numeric;
        line.line_total = computeLineTotal(line.quantity, line.unit_price);
      } else if (field === "description") {
        line.description = String(value);
      } else if (field === "tax_code_id") {
        line.tax_code_id = value === "" ? null : String(value);
      }
      line.tax_amount = computeTaxAmount(line);

      next[index] = line;
      return next;
    });
  };

  const addLine = () => {
    setLineItems((prev) => [...prev, emptyLine()]);
  };

  const removeLine = (index: number) => {
    setLineItems((prev) => {
      if (prev.length <= 1) return prev;
      const next = [...prev];
      next.splice(index, 1);
      return next;
    });
  };

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    const payloadLines = lineItems.map((line) => {
      const line_total = computeLineTotal(line.quantity, line.unit_price);
      const tax_amount = computeTaxAmount({ ...line, line_total });
      return {
        ...line,
        line_total,
        tax_amount,
      };
    });
    onSave({ form, lines: payloadLines });
  };

  const handlePartyChange = (value: string) => {
    setForm((prev) => ({ ...prev, party_id: value }));
  };

  const handleStatusChange = (value: string) => {
    setForm((prev) => ({ ...prev, status: value as InvoiceStatus }));
  };

  return (
    <>
      {/* Backdrop */}
      <div
        className={cn(
          "fixed inset-0 z-40 bg-black/50 transition-opacity duration-200",
          isOpen ? "opacity-100" : "pointer-events-none opacity-0"
        )}
        onClick={onClose}
        aria-hidden={!isOpen}
      />
      {/* Drawer */}
      <div
        className={cn(
          "fixed inset-y-0 right-0 z-50 w-full max-w-2xl transform bg-background shadow-xl transition-transform duration-200 ease-in-out",
          isOpen ? "translate-x-0" : "translate-x-full"
        )}
        role="dialog"
        aria-modal="true"
        aria-labelledby="invoice-drawer-title"
      >
        <form onSubmit={handleSubmit} className="flex h-full flex-col">
          <Card className="flex h-full flex-col rounded-none border-0 shadow-none">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 border-b px-6 py-4">
              <CardTitle id="invoice-drawer-title" className="text-lg">
                {invoiceId ? "Edit" : "New"} Invoice
              </CardTitle>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                onClick={onClose}
                aria-label="Close"
              >
                <X className="h-4 w-4" />
              </Button>
            </CardHeader>

            <CardContent className="flex-1 space-y-6 overflow-y-auto px-6 py-6">
              {error && (
                <div className="rounded-md border border-destructive bg-destructive/10 p-3 text-sm text-destructive">
                  {error.message}
                </div>
              )}
              {isLoading ? (
                <div className="text-sm text-muted-foreground">Loading...</div>
              ) : (
                <>
                  <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                    <div className="space-y-2">
                      <label
                        htmlFor="invoice-party"
                        className="text-sm font-medium"
                      >
                        Client
                      </label>
                      <select
                        id="invoice-party"
                        value={form.party_id}
                        onChange={(e) => handlePartyChange(e.target.value)}
                        required
                        className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                      >
                        <option value="" disabled>
                          Select a client
                        </option>
                        {parties.map((party) => (
                          <option key={party.id} value={party.id}>
                            {party.name}
                          </option>
                        ))}
                      </select>
                    </div>

                    <div className="space-y-2">
                      <label
                        htmlFor="invoice-number"
                        className="text-sm font-medium"
                      >
                        Invoice Number
                      </label>
                      <Input
                        id="invoice-number"
                        value={form.invoice_number}
                        onChange={(e) =>
                          setForm((prev) => ({
                            ...prev,
                            invoice_number: e.target.value,
                          }))
                        }
                        placeholder="INV-001"
                        required
                      />
                    </div>

                    <div className="space-y-2">
                      <label
                        htmlFor="invoice-issue-date"
                        className="text-sm font-medium"
                      >
                        Issue Date
                      </label>
                      <Input
                        id="invoice-issue-date"
                        type="date"
                        value={form.issue_date}
                        onChange={(e) =>
                          setForm((prev) => ({
                            ...prev,
                            issue_date: e.target.value,
                          }))
                        }
                        required
                      />
                    </div>

                    <div className="space-y-2">
                      <label
                        htmlFor="invoice-due-date"
                        className="text-sm font-medium"
                      >
                        Due Date
                      </label>
                      <Input
                        id="invoice-due-date"
                        type="date"
                        value={form.due_date}
                        onChange={(e) =>
                          setForm((prev) => ({
                            ...prev,
                            due_date: e.target.value,
                          }))
                        }
                        required
                      />
                    </div>

                    <div className="space-y-2">
                      <label
                        htmlFor="invoice-status"
                        className="text-sm font-medium"
                      >
                        Status
                      </label>
                      <select
                        id="invoice-status"
                        value={form.status}
                        onChange={(e) => handleStatusChange(e.target.value)}
                        className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                      >
                        {INVOICE_STATUS_OPTIONS.map((status) => (
                          <option key={status} value={status}>
                            {INVOICE_STATUS_LABELS[status]}
                          </option>
                        ))}
                      </select>
                    </div>

                    <div className="space-y-2">
                      <label
                        htmlFor="invoice-currency"
                        className="text-sm font-medium"
                      >
                        Currency
                      </label>
                      <select
                        id="invoice-currency"
                        value={form.currency}
                        onChange={(e) =>
                          setForm((prev) => ({
                            ...prev,
                            currency: e.target.value,
                            exchange_rate:
                              e.target.value === "MYR" ? 1 : prev.exchange_rate,
                          }))
                        }
                        required
                        className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                      >
                        {SUPPORTED_CURRENCIES.map((c) => (
                          <option key={c} value={c}>
                            {c}
                          </option>
                        ))}
                      </select>
                    </div>

                    <div className="space-y-2">
                      <label
                        htmlFor="invoice-exchange-rate"
                        className="text-sm font-medium"
                      >
                        Exchange Rate
                      </label>
                      <Input
                        id="invoice-exchange-rate"
                        type="number"
                        min="0.000001"
                        step="0.000001"
                        value={form.exchange_rate ?? 1}
                        onChange={(e) =>
                          setForm((prev) => ({
                            ...prev,
                            exchange_rate: Number(e.target.value),
                          }))
                        }
                        disabled={form.currency === "MYR"}
                        required
                      />
                    </div>
                  </div>

                  <div className="space-y-2">
                    <div className="flex items-center justify-between">
                      <label className="text-sm font-medium">Line Items</label>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={addLine}
                      >
                        <Plus className="h-4 w-4" />
                        Add Line
                      </Button>
                    </div>
                    <div className="rounded-md border">
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead>Description</TableHead>
                            <TableHead className="w-28">Quantity</TableHead>
                            <TableHead className="w-32">
                              Unit Price ({currency})
                            </TableHead>
                            <TableHead className="w-36">Tax Code</TableHead>
                            <TableHead className="w-28 text-right">
                              Tax ({currency})
                            </TableHead>
                            <TableHead className="w-28 text-right">
                              Total ({currency})
                            </TableHead>
                            <TableHead className="w-16" />
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {lineItems.map((line, index) => (
                            <TableRow key={index}>
                              <TableCell>
                                <Input
                                  value={line.description}
                                  onChange={(e) =>
                                    updateLine(
                                      index,
                                      "description",
                                      e.target.value
                                    )
                                  }
                                  placeholder="Item description"
                                  required
                                />
                              </TableCell>
                              <TableCell>
                                <Input
                                  type="number"
                                  min="0"
                                  step="1"
                                  value={line.quantity}
                                  onChange={(e) =>
                                    updateLine(
                                      index,
                                      "quantity",
                                      e.target.value
                                    )
                                  }
                                  required
                                />
                              </TableCell>
                              <TableCell>
                                <Input
                                  type="number"
                                  min="0"
                                  step="0.01"
                                  value={line.unit_price}
                                  onChange={(e) =>
                                    updateLine(
                                      index,
                                      "unit_price",
                                      e.target.value
                                    )
                                  }
                                  required
                                />
                              </TableCell>
                              <TableCell>
                                <select
                                  value={line.tax_code_id ?? ""}
                                  onChange={(e) =>
                                    updateLine(
                                      index,
                                      "tax_code_id",
                                      e.target.value
                                    )
                                  }
                                  className="flex h-9 w-full rounded-md border border-input bg-background px-2 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                >
                                  <option value="">No tax</option>
                                  {taxCodes.map((taxCode) => (
                                    <option key={taxCode.id} value={taxCode.id}>
                                      {taxCode.code} ({taxCode.rate * 100}%)
                                    </option>
                                  ))}
                                </select>
                              </TableCell>
                              <TableCell className="text-right font-medium">
                                {formatCurrency(line.tax_amount, currency)}
                              </TableCell>
                              <TableCell className="text-right font-medium">
                                {formatCurrency(
                                  computeLineTotal(
                                    line.quantity,
                                    line.unit_price
                                  ) + line.tax_amount,
                                  currency
                                )}
                              </TableCell>
                              <TableCell>
                                <Button
                                  type="button"
                                  variant="ghost"
                                  size="icon"
                                  onClick={() => removeLine(index)}
                                  disabled={lineItems.length <= 1}
                                  aria-label="Remove line"
                                >
                                  <Trash2 className="h-4 w-4 text-destructive" />
                                </Button>
                              </TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    </div>
                  </div>

                  {invoiceId && invoice && (
                    <div className="space-y-3 rounded-md border p-4">
                      <div className="flex items-center justify-between">
                        <label className="text-sm font-medium">
                          LHDN MyInvois
                        </label>
                        {(invoice.status === "draft" ||
                          invoice.status === "posted") &&
                          onSubmitLhdn && (
                            <Button
                              type="button"
                              variant="outline"
                              size="sm"
                              disabled={isSubmittingLhdn}
                              onClick={() => onSubmitLhdn(invoiceId)}
                            >
                              <Upload className="h-4 w-4" />
                              {isSubmittingLhdn
                                ? "Submitting..."
                                : "Submit to LHDN"}
                            </Button>
                          )}
                      </div>
                      <div className="text-sm text-muted-foreground">
                        <p>
                          Status:{" "}
                          <span className="font-medium text-foreground capitalize">
                            {lhdnStatus?.status ??
                              invoice.lhdn_status ??
                              "Not submitted"}
                          </span>
                        </p>
                        {(lhdnStatus?.lhdn_uuid ?? invoice.lhdn_uuid) && (
                          <p>
                            UUID:{" "}
                            <span className="font-medium text-foreground">
                              {lhdnStatus?.lhdn_uuid ?? invoice.lhdn_uuid}
                            </span>
                          </p>
                        )}
                        {(lhdnStatus?.error_message ?? invoice.lhdn_error) && (
                          <p className="text-destructive">
                            Error:{" "}
                            {lhdnStatus?.error_message ?? invoice.lhdn_error}
                          </p>
                        )}
                      </div>
                    </div>
                  )}

                  <div className="flex justify-end">
                    <div className="text-right space-y-1">
                      <p className="text-sm text-muted-foreground">
                        Subtotal ({currency})
                      </p>
                      <p className="text-xl font-semibold">
                        {formatCurrency(totalForeignAmount, currency)}
                      </p>
                      {totalTaxAmount > 0 && (
                        <p className="text-sm text-muted-foreground">
                          Tax ({currency}):{" "}
                          {formatCurrency(totalTaxAmount, currency)}
                        </p>
                      )}
                      <p className="text-sm text-muted-foreground">
                        Total ({currency})
                      </p>
                      <p className="text-2xl font-semibold">
                        {formatCurrency(totalForeignAmountWithTax, currency)}
                      </p>
                      {currency !== "MYR" && (
                        <p className="text-sm text-muted-foreground">
                          ≈ {formatCurrency(totalMyrAmountWithTax, "MYR")}
                        </p>
                      )}
                    </div>
                  </div>
                </>
              )}
            </CardContent>

            <CardFooter className="flex justify-between border-t px-6 py-4">
              {invoiceId && onDelete ? (
                <Button
                  type="button"
                  variant="destructive"
                  disabled={isSaving || isLoading}
                  onClick={() => onDelete(invoiceId)}
                >
                  Delete
                </Button>
              ) : (
                <div />
              )}
              <div className="flex gap-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={onClose}
                  disabled={isSaving}
                >
                  Cancel
                </Button>
                <Button type="submit" disabled={isSaving || isLoading}>
                  {isSaving ? "Saving..." : "Save"}
                </Button>
              </div>
            </CardFooter>
          </Card>
        </form>
      </div>
    </>
  );
}
