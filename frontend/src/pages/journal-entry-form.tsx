import { useEffect, useMemo, useState } from "react";
import { Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardDescription,
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
import type { Account } from "@/hooks/use-accounts";
import {
  JOURNAL_ENTRY_STATUS_LABELS,
  type JournalEntry,
  type JournalEntryCreate,
  type JournalLineCreate,
} from "@/hooks/use-journal-entries";

export interface JournalEntryFormLine {
  id: string;
  account_id: string;
  description: string;
  debit: string;
  credit: string;
}

interface JournalEntryFormProps {
  entry: JournalEntry | null;
  accounts: Account[];
  isOpen: boolean;
  isSaving: boolean;
  error: Error | null;
  onClose: () => void;
  onSave: (payload: JournalEntryCreate) => void;
  onPost?: (id: string) => void;
  onCancel?: (id: string) => void;
  onDelete?: (id: string) => void;
}

function emptyLine(): JournalEntryFormLine {
  return {
    id: crypto.randomUUID(),
    account_id: "",
    description: "",
    debit: "",
    credit: "",
  };
}

function parseMoney(value: string): string | null {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const normalized = trimmed.replace(/,/g, "");
  if (!/^\d+(\.\d{0,4})?$/.test(normalized)) return null;
  return normalized;
}

function formatMoney(value: string): string {
  const num = Number(value);
  if (Number.isNaN(num)) return value;
  return num.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 4,
  });
}

function toLinePayload(line: JournalEntryFormLine): JournalLineCreate {
  return {
    account_id: line.account_id,
    description: line.description.trim() || null,
    debit: parseMoney(line.debit),
    credit: parseMoney(line.credit),
  };
}

export function JournalEntryForm({
  entry,
  accounts,
  isOpen,
  isSaving,
  error,
  onClose,
  onSave,
  onPost,
  onCancel,
  onDelete,
}: JournalEntryFormProps) {
  const isEditing = entry !== null;
  const isPosted = entry?.status === "posted";
  const isCancelled = entry?.status === "cancelled";
  const isReadOnly = isPosted || isCancelled;

  const [entryDate, setEntryDate] = useState(entry?.entry_date.slice(0, 10) ?? "");
  const [reference, setReference] = useState(entry?.reference ?? "");
  const [description, setDescription] = useState(entry?.description ?? "");
  const [lines, setLines] = useState<JournalEntryFormLine[]>(() => {
    if (entry && entry.lines.length > 0) {
      return entry.lines.map((line) => ({
        id: crypto.randomUUID(),
        account_id: line.account_id,
        description: line.description ?? "",
        debit: line.debit || "",
        credit: line.credit || "",
      }));
    }
    return [emptyLine(), emptyLine()];
  });
  const [localError, setLocalError] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) return;
    setEntryDate(entry?.entry_date.slice(0, 10) ?? "");
    setReference(entry?.reference ?? "");
    setDescription(entry?.description ?? "");
    setLines(
      entry && entry.lines.length > 0
        ? entry.lines.map((line) => ({
            id: crypto.randomUUID(),
            account_id: line.account_id,
            description: line.description ?? "",
            debit: line.debit || "",
            credit: line.credit || "",
          }))
        : [emptyLine(), emptyLine()]
    );
    setLocalError(null);
  }, [isOpen, entry]);

  const activeAccounts = useMemo(
    () => accounts.filter((a) => a.is_active),
    [accounts]
  );

  const { totalDebit, totalCredit, isBalanced } = useMemo(() => {
    let debit = 0;
    let credit = 0;
    for (const line of lines) {
      const d = Number(parseMoney(line.debit) ?? 0);
      const c = Number(parseMoney(line.credit) ?? 0);
      debit += d;
      credit += c;
    }
    return {
      totalDebit: debit,
      totalCredit: credit,
      isBalanced: debit === credit && debit > 0,
    };
  }, [lines]);

  const updateLine = (
    id: string,
    field: keyof JournalEntryFormLine,
    value: string
  ) => {
    setLines((prev) =>
      prev.map((line) => (line.id === id ? { ...line, [field]: value } : line))
    );
    setLocalError(null);
  };

  const addLine = () => {
    setLines((prev) => [...prev, emptyLine()]);
    setLocalError(null);
  };

  const removeLine = (id: string) => {
    setLines((prev) => prev.filter((line) => line.id !== id));
    setLocalError(null);
  };

  const validate = (): JournalEntryCreate | null => {
    if (!entryDate) {
      setLocalError("Entry date is required.");
      return null;
    }

    if (lines.length < 2) {
      setLocalError("At least two lines are required.");
      return null;
    }

    const payloadLines: JournalLineCreate[] = [];
    let debitTotal = 0;
    let creditTotal = 0;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (!line.account_id) {
        setLocalError(`Line ${i + 1}: account is required.`);
        return null;
      }

      const debit = Number(parseMoney(line.debit) ?? 0);
      const credit = Number(parseMoney(line.credit) ?? 0);

      if (debit > 0 && credit > 0) {
        setLocalError(
          `Line ${i + 1}: a line cannot have both debit and credit.`
        );
        return null;
      }

      if (debit === 0 && credit === 0) {
        setLocalError(
          `Line ${i + 1}: a line must have either debit or credit greater than zero.`
        );
        return null;
      }

      debitTotal += debit;
      creditTotal += credit;

      payloadLines.push(toLinePayload(line));
    }

    if (debitTotal !== creditTotal) {
      setLocalError("Total debits must equal total credits.");
      return null;
    }

    return {
      entry_date: entryDate,
      reference: reference.trim() || null,
      description: description.trim() || null,
      status: isEditing ? undefined : "draft",
      lines: payloadLines,
    };
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const payload = validate();
    if (!payload) return;
    onSave(payload);
  };

  const displayError = localError || error?.message;

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <Card className="w-full max-w-4xl max-h-[90vh] overflow-y-auto">
        <CardHeader>
          <CardTitle>
            {isEditing
              ? `Edit Journal Entry${
                  entry ? ` (${JOURNAL_ENTRY_STATUS_LABELS[entry.status]})` : ""
                }`
              : "New Journal Entry"}
          </CardTitle>
          <CardDescription>
            {isReadOnly
              ? "This journal entry is posted and cannot be edited."
              : "Create a balanced double-entry journal."}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-6">
            {displayError && (
              <div className="rounded-md border border-destructive/50 bg-destructive/10 px-4 py-2 text-sm text-destructive">
                {displayError}
              </div>
            )}

            <div className="grid gap-4 sm:grid-cols-3">
              <div className="space-y-2">
                <label htmlFor="entry-date" className="text-sm font-medium">
                  Entry Date
                </label>
                <Input
                  id="entry-date"
                  type="date"
                  value={entryDate}
                  onChange={(e) => setEntryDate(e.target.value)}
                  disabled={isReadOnly || isSaving}
                  required
                />
              </div>
              <div className="space-y-2">
                <label htmlFor="reference" className="text-sm font-medium">
                  Reference
                </label>
                <Input
                  id="reference"
                  value={reference}
                  onChange={(e) => setReference(e.target.value)}
                  placeholder="e.g. ADJ-001"
                  disabled={isReadOnly || isSaving}
                />
              </div>
              <div className="space-y-2">
                <label htmlFor="description" className="text-sm font-medium">
                  Description
                </label>
                <Input
                  id="description"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="e.g. Opening balances"
                  disabled={isReadOnly || isSaving}
                />
              </div>
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="text-sm font-medium">Journal Lines</label>
                {!isReadOnly && (
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={addLine}
                    disabled={isSaving}
                  >
                    <Plus className="h-4 w-4" />
                    Add Line
                  </Button>
                )}
              </div>

              <div className="rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[240px]">Account</TableHead>
                      <TableHead>Description</TableHead>
                      <TableHead className="w-[140px]">Debit</TableHead>
                      <TableHead className="w-[140px]">Credit</TableHead>
                      {!isReadOnly && (
                        <TableHead className="w-[60px]"></TableHead>
                      )}
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {lines.map((line, index) => (
                      <TableRow key={line.id}>
                        <TableCell>
                          <select
                            value={line.account_id}
                            onChange={(e) =>
                              updateLine(line.id, "account_id", e.target.value)
                            }
                            disabled={isReadOnly || isSaving}
                            className="flex h-9 w-full rounded-md border border-input bg-transparent px-2 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                            required
                          >
                            <option value="">Select account</option>
                            {activeAccounts.map((account) => (
                              <option key={account.id} value={account.id}>
                                {account.code} - {account.name}
                              </option>
                            ))}
                          </select>
                        </TableCell>
                        <TableCell>
                          <Input
                            value={line.description}
                            onChange={(e) =>
                              updateLine(line.id, "description", e.target.value)
                            }
                            placeholder="Line description"
                            disabled={isReadOnly || isSaving}
                          />
                        </TableCell>
                        <TableCell>
                          <Input
                            value={line.debit}
                            onChange={(e) =>
                              updateLine(line.id, "debit", e.target.value)
                            }
                            placeholder="0.00"
                            disabled={isReadOnly || isSaving}
                            inputMode="decimal"
                          />
                        </TableCell>
                        <TableCell>
                          <Input
                            value={line.credit}
                            onChange={(e) =>
                              updateLine(line.id, "credit", e.target.value)
                            }
                            placeholder="0.00"
                            disabled={isReadOnly || isSaving}
                            inputMode="decimal"
                          />
                        </TableCell>
                        {!isReadOnly && (
                          <TableCell>
                            <Button
                              type="button"
                              variant="ghost"
                              size="icon"
                              onClick={() => removeLine(line.id)}
                              disabled={lines.length <= 2 || isSaving}
                              aria-label={`Remove line ${index + 1}`}
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </TableCell>
                        )}
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>

              <div className="flex items-center justify-between rounded-md border bg-muted/50 px-4 py-2 text-sm">
                <div className="flex gap-6">
                  <span>
                    Total Debit:{" "}
                    <strong className="tabular-nums">
                      {formatMoney(totalDebit.toString())}
                    </strong>
                  </span>
                  <span>
                    Total Credit:{" "}
                    <strong className="tabular-nums">
                      {formatMoney(totalCredit.toString())}
                    </strong>
                  </span>
                </div>
                <span
                  className={
                    isBalanced ? "text-green-600" : "text-destructive"
                  }
                >
                  {isBalanced ? "Balanced" : "Unbalanced"}
                </span>
              </div>
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <Button
                type="button"
                variant="outline"
                onClick={onClose}
                disabled={isSaving}
              >
                {isReadOnly ? "Close" : "Cancel"}
              </Button>

              {isEditing && entry?.status === "draft" && onDelete && (
                <Button
                  type="button"
                  variant="destructive"
                  onClick={() => onDelete(entry.id)}
                  disabled={isSaving}
                >
                  Delete
                </Button>
              )}

              {isEditing && entry?.status === "draft" && onPost && (
                <Button
                  type="button"
                  variant="secondary"
                  onClick={() => onPost(entry.id)}
                  disabled={isSaving}
                >
                  Post
                </Button>
              )}

              {isEditing && entry?.status === "posted" && onCancel && (
                <Button
                  type="button"
                  variant="secondary"
                  onClick={() => onCancel(entry.id)}
                  disabled={isSaving}
                >
                  Cancel
                </Button>
              )}

              {!isReadOnly && (
                <Button type="submit" disabled={isSaving}>
                  {isSaving ? "Saving..." : isEditing ? "Save" : "Create"}
                </Button>
              )}
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
