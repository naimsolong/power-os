import { useEffect, useMemo, useState } from "react";
import { Plus, Search, Send, XCircle } from "lucide-react";
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
  useBills,
  useBill,
  useCreateBill,
  useUpdateBill,
  usePostBill,
  useCancelBill,
  BILL_STATUS_LABELS,
  type BillCreate,
  type BillLineCreate,
} from "@/hooks/use-bills";

interface BillLineForm {
  description: string;
  account_id: string;
  quantity: string;
  unit_price: string;
}

function emptyLine(): BillLineForm {
  return {
    description: "",
    account_id: "",
    quantity: "1",
    unit_price: "0",
  };
}

export function BillsPage() {
  const [search, setSearch] = useState("");
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const [partyId, setPartyId] = useState("");
  const [billNumber, setBillNumber] = useState("");
  const [issueDate, setIssueDate] = useState("");
  const [dueDate, setDueDate] = useState("");
  const [currency, setCurrency] = useState("MYR");
  const [exchangeRate, setExchangeRate] = useState(1);
  const [lines, setLines] = useState<BillLineForm[]>([emptyLine()]);

  const { data: bills, isLoading, error } = useBills(search);
  const { data: selectedBill, isLoading: isLoadingBill } = useBill(selectedId);
  const { data: parties } = useParties("vendor");
  const { data: accounts } = useAccounts();

  const createBill = useCreateBill();
  const updateBill = useUpdateBill();
  const postBill = usePostBill();
  const cancelBill = useCancelBill();

  const mutationError =
    createBill.error || updateBill.error || postBill.error || cancelBill.error;

  const partyById = useMemo(() => {
    const map = new Map<string, string>();
    if (!parties) return map;
    for (const party of parties) {
      map.set(party.id, party.name);
    }
    return map;
  }, [parties]);

  useEffect(() => {
    if (isFormOpen && selectedId && selectedBill) {
      setPartyId(selectedBill.party_id);
      setBillNumber(selectedBill.bill_number);
      setIssueDate(selectedBill.issue_date.slice(0, 10));
      setDueDate(
        selectedBill.due_date ? selectedBill.due_date.slice(0, 10) : ""
      );
      setCurrency(selectedBill.currency || "MYR");
      setExchangeRate(selectedBill.exchange_rate ?? 1);
      setLines(
        selectedBill.lines?.map((line) => ({
          description: line.description,
          account_id: line.account_id,
          quantity: String(line.quantity),
          unit_price: String(line.foreign_unit_price ?? line.unit_price),
        })) ?? [emptyLine()]
      );
    }
  }, [isFormOpen, selectedId, selectedBill]);

  const resetForm = () => {
    setPartyId("");
    setBillNumber("");
    setIssueDate("");
    setDueDate("");
    setCurrency("MYR");
    setExchangeRate(1);
    setLines([emptyLine()]);
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
    createBill.reset();
    updateBill.reset();
    postBill.reset();
    cancelBill.reset();
  };

  const updateLine = (index: number, field: keyof BillLineForm, value: string) => {
    setLines((prev) =>
      prev.map((line, i) => (i === index ? { ...line, [field]: value } : line))
    );
  };

  const removeLine = (index: number) => {
    setLines((prev) => prev.filter((_, i) => i !== index));
  };

  const addLine = () => {
    setLines((prev) => [...prev, emptyLine()]);
  };

  const total = useMemo(() => {
    return lines.reduce((sum, line) => {
      const qty = Number(line.quantity) || 0;
      const price = Number(line.unit_price) || 0;
      return sum + qty * price;
    }, 0);
  }, [lines]);

  const buildPayload = (): BillCreate => ({
    party_id: partyId,
    bill_number: billNumber,
    issue_date: issueDate,
    due_date: dueDate || undefined,
    currency,
    exchange_rate: exchangeRate,
    lines: lines
      .filter((line) => line.description.trim() && line.account_id)
      .map(
        (line): BillLineCreate => ({
          description: line.description,
          account_id: line.account_id,
          quantity: Number(line.quantity) || 0,
          unit_price: Number(line.unit_price) || 0,
        })
      ),
  });

  const handleSave = async () => {
    const payload = buildPayload();
    try {
      if (selectedId) {
        await updateBill.mutateAsync({ id: selectedId, data: payload });
      } else {
        await createBill.mutateAsync(payload);
      }
      handleCloseForm();
    } catch {
      // Errors surfaced by mutation state.
    }
  };

  const handlePost = (id: string) => {
    if (confirm("Post this draft bill?")) {
      postBill.mutate(id);
    }
  };

  const handleCancel = (id: string) => {
    if (confirm("Cancel this bill and reverse the journal entry?")) {
      cancelBill.mutate(id);
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

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Bills</h2>
          <p className="text-muted-foreground">
            Manage vendor bills and accounts payable.
          </p>
        </div>
        <Button onClick={handleNew}>
          <Plus className="h-4 w-4" />
          Create Bill
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Search</CardTitle>
          <CardDescription>Find bills by bill number.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Search className="h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search bills..."
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
              <TableHead>Vendor</TableHead>
              <TableHead>Amount</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Due Date</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell
                  colSpan={6}
                  className="h-24 text-center text-muted-foreground"
                >
                  Loading...
                </TableCell>
              </TableRow>
            ) : error ? (
              <TableRow>
                <TableCell
                  colSpan={6}
                  className="h-24 text-center text-destructive"
                >
                  Failed to load bills: {error.message}
                </TableCell>
              </TableRow>
            ) : !bills || bills.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={6}
                  className="h-24 text-center text-muted-foreground"
                >
                  No bills found.
                </TableCell>
              </TableRow>
            ) : (
              bills.map((bill) => (
                <TableRow
                  key={bill.id}
                  className="cursor-pointer"
                  onClick={() => handleEdit(bill.id)}
                >
                  <TableCell className="font-medium">
                    {bill.bill_number}
                  </TableCell>
                  <TableCell>
                    {partyById.get(bill.party_id) ?? bill.party_id}
                  </TableCell>
                  <TableCell>
                    {formatCurrency(bill.total_amount, bill.currency)}
                  </TableCell>
                  <TableCell className="capitalize">
                    {BILL_STATUS_LABELS[bill.status]}
                  </TableCell>
                  <TableCell>
                    {bill.due_date ? bill.due_date.slice(0, 10) : "—"}
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-2">
                      {bill.status === "draft" && (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            handlePost(bill.id);
                          }}
                        >
                          <Send className="h-4 w-4" />
                          Post
                        </Button>
                      )}
                      {bill.status !== "draft" && bill.status !== "cancelled" && (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleCancel(bill.id);
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
              {selectedId ? "Edit Bill" : "New Bill"}
            </CardTitle>
            <CardDescription>
              {selectedId
                ? "Update the bill details and lines below."
                : "Create a new vendor bill."}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Vendor</label>
                <select
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                  value={partyId}
                  onChange={(e) => setPartyId(e.target.value)}
                >
                  <option value="">Select vendor</option>
                  {parties?.map((party) => (
                    <option key={party.id} value={party.id}>
                      {party.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Bill Number</label>
                <Input
                  value={billNumber}
                  onChange={(e) => setBillNumber(e.target.value)}
                  placeholder="BILL-001"
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Issue Date</label>
                <Input
                  type="date"
                  value={issueDate}
                  onChange={(e) => setIssueDate(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Due Date</label>
                <Input
                  type="date"
                  value={dueDate}
                  onChange={(e) => setDueDate(e.target.value)}
                />
              </div>
            </div>

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
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
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm font-medium">Lines</label>
                <Button type="button" variant="outline" size="sm" onClick={addLine}>
                  Add Line
                </Button>
              </div>
              <div className="rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Description</TableHead>
                      <TableHead>Account</TableHead>
                      <TableHead>Quantity</TableHead>
                      <TableHead>Unit Price</TableHead>
                      <TableHead>Amount</TableHead>
                      <TableHead className="w-16" />
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {lines.map((line, index) => {
                      const amount =
                        (Number(line.quantity) || 0) *
                        (Number(line.unit_price) || 0);
                      return (
                        <TableRow key={index}>
                          <TableCell>
                            <Input
                              value={line.description}
                              onChange={(e) =>
                                updateLine(index, "description", e.target.value)
                              }
                              placeholder="Description"
                            />
                          </TableCell>
                          <TableCell>
                            <select
                              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                              value={line.account_id}
                              onChange={(e) =>
                                updateLine(index, "account_id", e.target.value)
                              }
                            >
                              <option value="">Select account</option>
                              {accounts?.map((account) => (
                                <option key={account.id} value={account.id}>
                                  {account.code} - {account.name}
                                </option>
                              ))}
                            </select>
                          </TableCell>
                          <TableCell>
                            <Input
                              type="number"
                              min="0"
                              step="0.01"
                              value={line.quantity}
                              onChange={(e) =>
                                updateLine(index, "quantity", e.target.value)
                              }
                            />
                          </TableCell>
                          <TableCell>
                            <Input
                              type="number"
                              min="0"
                              step="0.01"
                              value={line.unit_price}
                              onChange={(e) =>
                                updateLine(index, "unit_price", e.target.value)
                              }
                            />
                          </TableCell>
                          <TableCell>
                            {formatCurrency(amount, currency)}
                          </TableCell>
                          <TableCell>
                            <Button
                              type="button"
                              variant="ghost"
                              size="sm"
                              onClick={() => removeLine(index)}
                              disabled={lines.length === 1}
                            >
                              Remove
                            </Button>
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </div>
              <div className="flex justify-end text-sm font-medium">
                Total: {formatCurrency(total, currency)}
              </div>
            </div>

            {selectedId && isLoadingBill && (
              <p className="text-sm text-muted-foreground">Loading bill...</p>
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
                  createBill.isPending ||
                  updateBill.isPending ||
                  !partyId ||
                  !billNumber ||
                  !issueDate
                }
              >
                {selectedId ? "Update Bill" : "Create Bill"}
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
