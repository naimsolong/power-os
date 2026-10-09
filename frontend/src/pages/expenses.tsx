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
  useExpenses,
  useExpense,
  useCreateExpense,
  useUpdateExpense,
  usePostExpense,
  useCancelExpense,
  EXPENSE_STATUS_LABELS,
  PAYMENT_METHOD_OPTIONS,
  type ExpenseCreate,
  type ExpenseLineCreate,
} from "@/hooks/use-expenses";

interface ExpenseLineForm {
  description: string;
  account_id: string;
  amount: string;
}

function emptyLine(): ExpenseLineForm {
  return {
    description: "",
    account_id: "",
    amount: "0",
  };
}

export function ExpensesPage() {
  const [search, setSearch] = useState("");
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const [partyId, setPartyId] = useState("");
  const [expenseDate, setExpenseDate] = useState("");
  const [description, setDescription] = useState("");
  const [reference, setReference] = useState("");
  const [paymentMethod, setPaymentMethod] = useState("");
  const [paidFromAccountId, setPaidFromAccountId] = useState("");
  const [currency, setCurrency] = useState("MYR");
  const [lines, setLines] = useState<ExpenseLineForm[]>([emptyLine()]);

  const { data: expenses, isLoading, error } = useExpenses(search);
  const { data: selectedExpense, isLoading: isLoadingExpense } =
    useExpense(selectedId);
  const { data: parties } = useParties("vendor");
  const { data: accounts } = useAccounts();

  const createExpense = useCreateExpense();
  const updateExpense = useUpdateExpense();
  const postExpense = usePostExpense();
  const cancelExpense = useCancelExpense();

  const mutationError =
    createExpense.error ||
    updateExpense.error ||
    postExpense.error ||
    cancelExpense.error;

  const partyById = useMemo(() => {
    const map = new Map<string, string>();
    if (!parties) return map;
    for (const party of parties) {
      map.set(party.id, party.name);
    }
    return map;
  }, [parties]);

  useEffect(() => {
    if (isFormOpen && selectedId && selectedExpense) {
      setPartyId(selectedExpense.party_id ?? "");
      setExpenseDate(selectedExpense.expense_date.slice(0, 10));
      setDescription(selectedExpense.description);
      setReference(selectedExpense.reference ?? "");
      setPaymentMethod(selectedExpense.payment_method ?? "");
      setPaidFromAccountId(selectedExpense.paid_from_account_id ?? "");
      setCurrency(selectedExpense.currency || "MYR");
      setLines(
        selectedExpense.lines?.map((line) => ({
          description: line.description,
          account_id: line.account_id,
          amount: String(line.amount),
        })) ?? [emptyLine()]
      );
    }
  }, [isFormOpen, selectedId, selectedExpense]);

  const resetForm = () => {
    setPartyId("");
    setExpenseDate("");
    setDescription("");
    setReference("");
    setPaymentMethod("");
    setPaidFromAccountId("");
    setCurrency("MYR");
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
    createExpense.reset();
    updateExpense.reset();
    postExpense.reset();
    cancelExpense.reset();
  };

  const updateLine = (
    index: number,
    field: keyof ExpenseLineForm,
    value: string
  ) => {
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
      return sum + (Number(line.amount) || 0);
    }, 0);
  }, [lines]);

  const buildPayload = (): ExpenseCreate => ({
    party_id: partyId || undefined,
    expense_date: expenseDate,
    description,
    reference: reference || undefined,
    payment_method: paymentMethod || undefined,
    paid_from_account_id: paidFromAccountId || undefined,
    currency,
    lines: lines
      .filter((line) => line.description.trim() && line.account_id)
      .map(
        (line): ExpenseLineCreate => ({
          description: line.description,
          account_id: line.account_id,
          amount: Number(line.amount) || 0,
        })
      ),
  });

  const handleSave = async () => {
    const payload = buildPayload();
    try {
      if (selectedId) {
        await updateExpense.mutateAsync({ id: selectedId, data: payload });
      } else {
        await createExpense.mutateAsync(payload);
      }
      handleCloseForm();
    } catch {
      // Errors surfaced by mutation state.
    }
  };

  const handlePost = (id: string) => {
    if (confirm("Post this draft expense?")) {
      postExpense.mutate(id);
    }
  };

  const handleCancel = (id: string) => {
    if (confirm("Cancel this expense and reverse the journal entry?")) {
      cancelExpense.mutate(id);
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
          <h2 className="text-2xl font-semibold tracking-tight">Expenses</h2>
          <p className="text-muted-foreground">
            Record direct expenses and cash payments.
          </p>
        </div>
        <Button onClick={handleNew}>
          <Plus className="h-4 w-4" />
          Record Expense
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Search</CardTitle>
          <CardDescription>
            Find expenses by description or reference.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Search className="h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search expenses..."
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
              <TableHead>Description</TableHead>
              <TableHead>Vendor</TableHead>
              <TableHead>Amount</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Date</TableHead>
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
                  Failed to load expenses: {error.message}
                </TableCell>
              </TableRow>
            ) : !expenses || expenses.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={6}
                  className="h-24 text-center text-muted-foreground"
                >
                  No expenses found.
                </TableCell>
              </TableRow>
            ) : (
              expenses.map((expense) => (
                <TableRow
                  key={expense.id}
                  className="cursor-pointer"
                  onClick={() => handleEdit(expense.id)}
                >
                  <TableCell className="font-medium">
                    {expense.description}
                  </TableCell>
                  <TableCell>
                    {expense.party_id
                      ? partyById.get(expense.party_id) ?? expense.party_id
                      : "—"}
                  </TableCell>
                  <TableCell>
                    {formatCurrency(expense.total_amount, expense.currency)}
                  </TableCell>
                  <TableCell className="capitalize">
                    {EXPENSE_STATUS_LABELS[expense.status]}
                  </TableCell>
                  <TableCell>{expense.expense_date.slice(0, 10)}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-2">
                      {expense.status === "draft" && (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            handlePost(expense.id);
                          }}
                        >
                          <Send className="h-4 w-4" />
                          Post
                        </Button>
                      )}
                      {expense.status === "posted" && (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleCancel(expense.id);
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
              {selectedId ? "Edit Expense" : "New Expense"}
            </CardTitle>
            <CardDescription>
              {selectedId
                ? "Update the expense details and lines below."
                : "Record a new direct expense."}
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
                  <option value="">Optional vendor</option>
                  {parties?.map((party) => (
                    <option key={party.id} value={party.id}>
                      {party.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Date</label>
                <Input
                  type="date"
                  value={expenseDate}
                  onChange={(e) => setExpenseDate(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Payment Method</label>
                <select
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                  value={paymentMethod}
                  onChange={(e) => setPaymentMethod(e.target.value)}
                >
                  <option value="">Unpaid / On Account</option>
                  {PAYMENT_METHOD_OPTIONS.map((method) => (
                    <option key={method} value={method}>
                      {method.replace("_", " ")}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Currency</label>
                <Input
                  value={currency}
                  onChange={(e) => setCurrency(e.target.value)}
                  placeholder="MYR"
                />
              </div>
            </div>

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <div className="space-y-2">
                <label className="text-sm font-medium">Description</label>
                <Input
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Expense description"
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Reference</label>
                <Input
                  value={reference}
                  onChange={(e) => setReference(e.target.value)}
                  placeholder="Receipt or transaction reference"
                />
              </div>
            </div>

            {paymentMethod && (
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Paid From</label>
                  <select
                    className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                    value={paidFromAccountId}
                    onChange={(e) => setPaidFromAccountId(e.target.value)}
                  >
                    <option value="">Select cash/bank account</option>
                    {accounts
                      ?.filter(
                        (account) =>
                          account.account_type === "asset" &&
                          (account.code.startsWith("1000") ||
                            account.code.startsWith("1100") ||
                            account.code.startsWith("1200"))
                      )
                      .map((account) => (
                        <option key={account.id} value={account.id}>
                          {account.code} - {account.name}
                        </option>
                      ))}
                  </select>
                </div>
              </div>
            )}

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm font-medium">Lines</label>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={addLine}
                >
                  Add Line
                </Button>
              </div>
              <div className="rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Description</TableHead>
                      <TableHead>Account</TableHead>
                      <TableHead>Amount</TableHead>
                      <TableHead className="w-16" />
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {lines.map((line, index) => (
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
                            value={line.amount}
                            onChange={(e) =>
                              updateLine(index, "amount", e.target.value)
                            }
                          />
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
                    ))}
                  </TableBody>
                </Table>
              </div>
              <div className="flex justify-end text-sm font-medium">
                Total: {formatCurrency(total, currency)}
              </div>
            </div>

            {selectedId && isLoadingExpense && (
              <p className="text-sm text-muted-foreground">Loading expense...</p>
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
                  createExpense.isPending ||
                  updateExpense.isPending ||
                  !description ||
                  !expenseDate
                }
              >
                {selectedId ? "Update Expense" : "Record Expense"}
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
