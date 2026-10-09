import { useEffect, useMemo, useState } from "react";
import { Plus, Search, Send, XCircle, Trash2 } from "lucide-react";
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
import { useAccounts } from "@/hooks/use-accounts";
import { useParties } from "@/hooks/use-parties";
import {
  usePayments,
  usePayment,
  useCreatePayment,
  useUpdatePayment,
  usePostPayment,
  useCancelPayment,
  usePartyOpenInvoices,
  usePartyOpenBills,
  PAYMENT_STATUS_LABELS,
  PAYMENT_DIRECTION_LABELS,
  PAYMENT_METHOD_LABELS,
  PAYMENT_DIRECTION_OPTIONS,
  PAYMENT_METHOD_OPTIONS,
  type PaymentCreate,
  type PaymentAllocationCreate,
  type PaymentDirection,
  type PaymentMethod,
} from "@/hooks/use-payments";

interface AllocationForm {
  invoice_id?: string;
  bill_id?: string;
  amount: string;
}

function emptyAllocation(): AllocationForm {
  return { amount: "" };
}

export function PaymentsPage() {
  const [search, setSearch] = useState("");
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const [partyId, setPartyId] = useState("");
  const [bankAccountId, setBankAccountId] = useState("");
  const [paymentDate, setPaymentDate] = useState("");
  const [amount, setAmount] = useState("");
  const [currency, setCurrency] = useState("MYR");
  const [exchangeRate, setExchangeRate] = useState(1);
  const [paymentMethod, setPaymentMethod] = useState<PaymentMethod>("bank_transfer");
  const [reference, setReference] = useState("");
  const [notes, setNotes] = useState("");
  const [direction, setDirection] = useState<PaymentDirection>("received");
  const [allocations, setAllocations] = useState<AllocationForm[]>([]);

  const { data: payments, isLoading, error } = usePayments(search);
  const { data: selectedPayment, isLoading: isLoadingPayment } = usePayment(selectedId);
  const { data: parties } = useParties();
  const { data: accounts } = useAccounts();

  const { data: openInvoices } = usePartyOpenInvoices(
    direction === "received" ? partyId : null
  );
  const { data: openBills } = usePartyOpenBills(
    direction === "sent" ? partyId : null
  );

  const createPayment = useCreatePayment();
  const updatePayment = useUpdatePayment();
  const postPayment = usePostPayment();
  const cancelPayment = useCancelPayment();

  const mutationError =
    createPayment.error ||
    updatePayment.error ||
    postPayment.error ||
    cancelPayment.error;

  const bankAccounts = useMemo(
    () => accounts?.filter((a) => a.account_type === "asset") ?? [],
    [accounts]
  );

  const partyById = useMemo(() => {
    const map = new Map<string, string>();
    if (!parties) return map;
    for (const party of parties) {
      map.set(party.id, party.name);
    }
    return map;
  }, [parties]);

  useEffect(() => {
    if (isFormOpen && selectedId && selectedPayment) {
      setPartyId(selectedPayment.party_id);
      setBankAccountId(selectedPayment.bank_account_id);
      setPaymentDate(selectedPayment.payment_date.slice(0, 10));
      setAmount(String(selectedPayment.foreign_amount || selectedPayment.amount));
      setCurrency(selectedPayment.currency || "MYR");
      setExchangeRate(selectedPayment.exchange_rate ?? 1);
      setPaymentMethod(selectedPayment.payment_method);
      setReference(selectedPayment.reference ?? "");
      setNotes(selectedPayment.notes ?? "");
      setDirection(selectedPayment.direction);
      setAllocations(
        selectedPayment.allocations?.map((a) => ({
          invoice_id: a.invoice_id,
          bill_id: a.bill_id,
          amount: String(a.foreign_amount ?? a.amount),
        })) ?? [emptyAllocation()]
      );
    }
  }, [isFormOpen, selectedId, selectedPayment]);

  const resetForm = () => {
    setPartyId("");
    setBankAccountId("");
    setPaymentDate("");
    setAmount("");
    setCurrency("MYR");
    setExchangeRate(1);
    setPaymentMethod("bank_transfer");
    setReference("");
    setNotes("");
    setDirection("received");
    setAllocations([emptyAllocation()]);
  };

  const handleNew = () => {
    resetForm();
    setSelectedId(null);
    setIsFormOpen(true);
  };

  const handleEdit = (id: string) => {
    resetForm();
    setSelectedId(id);
    setIsFormOpen(true);
  };

  const handleCloseForm = () => {
    setIsFormOpen(false);
    setSelectedId(null);
    resetForm();
    createPayment.reset();
    updatePayment.reset();
    postPayment.reset();
    cancelPayment.reset();
  };

  const updateAllocation = (
    index: number,
    field: keyof AllocationForm,
    value: string
  ) => {
    setAllocations((prev) =>
      prev.map((a, i) => (i === index ? { ...a, [field]: value } : a))
    );
  };

  const updateAllocationDocument = (index: number, documentId: string) => {
    setAllocations((prev) =>
      prev.map((a, i) =>
        i === index
          ? direction === "received"
            ? { ...a, invoice_id: documentId, bill_id: undefined }
            : { ...a, bill_id: documentId, invoice_id: undefined }
          : a
      )
    );
  };

  const removeAllocation = (index: number) => {
    setAllocations((prev) => prev.filter((_, i) => i !== index));
  };

  const addAllocation = () => {
    setAllocations((prev) => [...prev, emptyAllocation()]);
  };

  const allocationTotal = useMemo(() => {
    return allocations.reduce((sum, a) => {
      return sum + (Number(a.amount) || 0);
    }, 0);
  }, [allocations]);

  const paymentAmount = Number(amount) || 0;
  const remaining = paymentAmount - allocationTotal;

  const documentOptions = useMemo(() => {
    if (direction === "received") {
      return (
        openInvoices?.map((inv) => ({
          id: inv.id,
          label: `${inv.invoice_number} — ${formatCurrency(
            inv.total_amount,
            inv.currency
          )}`,
        })) ?? []
      );
    }
    return (
      openBills?.map((bill) => ({
        id: bill.id,
        label: `${bill.bill_number} — ${formatCurrency(
          bill.total_amount,
          bill.currency
        )}`,
      })) ?? []
    );
  }, [direction, openInvoices, openBills]);

  const buildPayload = (): PaymentCreate => {
    const payload: PaymentCreate = {
      party_id: partyId,
      bank_account_id: bankAccountId,
      payment_date: paymentDate,
      amount: paymentAmount,
      currency,
      exchange_rate: exchangeRate,
      payment_method: paymentMethod,
      reference: reference || undefined,
      notes: notes || undefined,
      direction,
      allocations: allocations
        .filter((a) => {
          const docId = direction === "received" ? a.invoice_id : a.bill_id;
          return docId && Number(a.amount) > 0;
        })
        .map(
          (a): PaymentAllocationCreate => ({
            invoice_id: direction === "received" ? a.invoice_id : undefined,
            bill_id: direction === "sent" ? a.bill_id : undefined,
            amount: Number(a.amount) || 0,
          })
        ),
    };
    return payload;
  };

  const handleSave = async () => {
    const payload = buildPayload();
    try {
      if (selectedId) {
        await updatePayment.mutateAsync({ id: selectedId, data: payload });
      } else {
        await createPayment.mutateAsync(payload);
      }
      handleCloseForm();
    } catch {
      // Errors surfaced by mutation state.
    }
  };

  const handlePost = (id: string) => {
    if (confirm("Post this payment to the general ledger?")) {
      postPayment.mutate(id);
    }
  };

  const handleCancel = (id: string) => {
    if (confirm("Cancel this payment and reverse the journal entry?")) {
      cancelPayment.mutate(id);
    }
  };

  const formatCurrency = (value: number, currencyCode: string) => {
    try {
      return new Intl.NumberFormat("en-US", {
        style: "currency",
        currency: currencyCode || "MYR",
      }).format(value);
    } catch {
      return `RM${value.toLocaleString()}`;
    }
  };

  const canSave =
    partyId &&
    bankAccountId &&
    paymentDate &&
    paymentAmount > 0 &&
    allocationTotal > 0 &&
    allocationTotal <= paymentAmount;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Payments</h2>
          <p className="text-muted-foreground">
            Record customer and vendor payments and allocate them to invoices or bills.
          </p>
        </div>
        <Button onClick={handleNew}>
          <Plus className="h-4 w-4" />
          Record Payment
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Search</CardTitle>
          <CardDescription>Find payments by reference or party.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Search className="h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search payments..."
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
              <TableHead>Date</TableHead>
              <TableHead>Party</TableHead>
              <TableHead>Direction</TableHead>
              <TableHead>Amount</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Reference</TableHead>
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
                  Failed to load payments: {error.message}
                </TableCell>
              </TableRow>
            ) : !payments || payments.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={7}
                  className="h-24 text-center text-muted-foreground"
                >
                  No payments found.
                </TableCell>
              </TableRow>
            ) : (
              payments.map((payment) => (
                <TableRow
                  key={payment.id}
                  className="cursor-pointer"
                  onClick={() => handleEdit(payment.id)}
                >
                  <TableCell className="font-medium">
                    {payment.payment_date.slice(0, 10)}
                  </TableCell>
                  <TableCell>
                    {partyById.get(payment.party_id) ?? payment.party_id}
                  </TableCell>
                  <TableCell>
                    {PAYMENT_DIRECTION_LABELS[payment.direction]}
                  </TableCell>
                  <TableCell>
                    {formatCurrency(payment.foreign_amount, payment.currency)}
                    {payment.currency !== "MYR" && (
                      <span className="ml-2 text-xs text-muted-foreground">
                        ≈ {formatCurrency(payment.amount, "MYR")}
                      </span>
                    )}
                  </TableCell>
                  <TableCell className="capitalize">
                    {PAYMENT_STATUS_LABELS[payment.status]}
                  </TableCell>
                  <TableCell>{payment.reference || "—"}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-2">
                      {payment.status === "draft" && (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            handlePost(payment.id);
                          }}
                        >
                          <Send className="h-4 w-4" />
                          Post
                        </Button>
                      )}
                      {payment.status === "posted" && (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleCancel(payment.id);
                          }}
                        >
                          <XCircle className="h-4 w-4" />
                          Cancel
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

      {isFormOpen && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">
              {selectedId ? "Edit Payment" : "New Payment"}
            </CardTitle>
            <CardDescription>
              {selectedId
                ? "Update the payment details and allocations below."
                : "Record a new customer or vendor payment."}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Direction</label>
                <select
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                  value={direction}
                  onChange={(e) => {
                    setDirection(e.target.value as PaymentDirection);
                    setAllocations([emptyAllocation()]);
                  }}
                >
                  {PAYMENT_DIRECTION_OPTIONS.map((d) => (
                    <option key={d} value={d}>
                      {PAYMENT_DIRECTION_LABELS[d]}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Party</label>
                <select
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                  value={partyId}
                  onChange={(e) => {
                    setPartyId(e.target.value);
                    setAllocations([emptyAllocation()]);
                  }}
                >
                  <option value="">Select party</option>
                  {parties?.map((party) => (
                    <option key={party.id} value={party.id}>
                      {party.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Bank Account</label>
                <select
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                  value={bankAccountId}
                  onChange={(e) => setBankAccountId(e.target.value)}
                >
                  <option value="">Select account</option>
                  {bankAccounts.map((account) => (
                    <option key={account.id} value={account.id}>
                      {account.code} - {account.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Payment Date</label>
                <Input
                  type="date"
                  value={paymentDate}
                  onChange={(e) => setPaymentDate(e.target.value)}
                />
              </div>
            </div>

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Amount</label>
                <Input
                  type="number"
                  min="0"
                  step="0.01"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="0.00"
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Currency</label>
                <select
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                  value={currency}
                  onChange={(e) => {
                    const value = e.target.value;
                    setCurrency(value);
                    if (value === "MYR") setExchangeRate(1);
                  }}
                >
                  <option value="MYR">MYR</option>
                  <option value="USD">USD</option>
                  <option value="EUR">EUR</option>
                  <option value="SGD">SGD</option>
                </select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Exchange Rate</label>
                <Input
                  type="number"
                  min="0.000001"
                  step="0.000001"
                  value={exchangeRate}
                  onChange={(e) => setExchangeRate(Number(e.target.value))}
                  disabled={currency === "MYR"}
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Method</label>
                <select
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                  value={paymentMethod}
                  onChange={(e) =>
                    setPaymentMethod(e.target.value as PaymentMethod)
                  }
                >
                  {PAYMENT_METHOD_OPTIONS.map((m) => (
                    <option key={m} value={m}>
                      {PAYMENT_METHOD_LABELS[m]}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Reference</label>
                <Input
                  value={reference}
                  onChange={(e) => setReference(e.target.value)}
                  placeholder="REF-001"
                />
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">Notes</label>
              <Input
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
                placeholder="Optional notes"
              />
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm font-medium">Allocations</label>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={addAllocation}
                  disabled={!partyId}
                >
                  Add Allocation
                </Button>
              </div>
              <div className="rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[320px]">
                        {direction === "received" ? "Invoice" : "Bill"}
                      </TableHead>
                      <TableHead>Amount</TableHead>
                      <TableHead className="w-16" />
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {allocations.map((allocation, index) => {
                      const documentId =
                        direction === "received"
                          ? allocation.invoice_id
                          : allocation.bill_id;
                      return (
                        <TableRow key={index}>
                          <TableCell>
                            <select
                              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                              value={documentId || ""}
                              onChange={(e) =>
                                updateAllocationDocument(index, e.target.value)
                              }
                              disabled={!partyId}
                            >
                              <option value="">
                                Select{" "}
                                {direction === "received"
                                  ? "invoice"
                                  : "bill"}
                              </option>
                              {documentOptions.map((doc) => (
                                <option key={doc.id} value={doc.id}>
                                  {doc.label}
                                </option>
                              ))}
                            </select>
                          </TableCell>
                          <TableCell>
                            <Input
                              type="number"
                              min="0"
                              step="0.01"
                              value={allocation.amount}
                              onChange={(e) =>
                                updateAllocation(index, "amount", e.target.value)
                              }
                              placeholder="0.00"
                            />
                          </TableCell>
                          <TableCell>
                            <Button
                              type="button"
                              variant="ghost"
                              size="sm"
                              onClick={() => removeAllocation(index)}
                              disabled={allocations.length === 1}
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">
                  Allocated: {formatCurrency(allocationTotal, currency)}
                </span>
                <span
                  className={
                    remaining < 0 ? "text-destructive font-medium" : "font-medium"
                  }
                >
                  Remaining: {formatCurrency(remaining, currency)}
                </span>
              </div>
            </div>

            {selectedId && isLoadingPayment && (
              <p className="text-sm text-muted-foreground">Loading payment...</p>
            )}

            <div className="flex justify-end gap-2">
              <Button
                type="button"
                variant="outline"
                onClick={handleCloseForm}
              >
                Cancel
              </Button>
              <Button
                type="button"
                onClick={handleSave}
                disabled={
                  createPayment.isPending ||
                  updatePayment.isPending ||
                  !canSave
                }
              >
                {selectedId ? "Update Payment" : "Record Payment"}
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

export default PaymentsPage;
