import { useMemo, useState } from "react";
import { Eye, Plus, RotateCcw, Search, Send, Trash2 } from "lucide-react";
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
import {
  useJournalEntries,
  useJournalEntry,
  useCreateJournalEntry,
  useUpdateJournalEntry,
  useDeleteJournalEntry,
  usePostJournalEntry,
  useCancelJournalEntry,
  JOURNAL_ENTRY_STATUS_LABELS,
  type JournalEntry,
  type JournalEntryCreate,
  type JournalEntryStatus,
} from "@/hooks/use-journal-entries";
import { JournalEntryForm } from "./journal-entry-form";

function formatAmount(value: string): string {
  const num = Number(value);
  if (Number.isNaN(num)) return value;
  return num.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

export function JournalEntriesPage() {
  const [search, setSearch] = useState("");
  const [statusFilter, setStatusFilter] = useState<JournalEntryStatus | "">("");
  const [formOpen, setFormOpen] = useState(false);
  const [selectedEntryId, setSelectedEntryId] = useState<string | null>(null);

  const { data: entries, isLoading, error } = useJournalEntries(
    undefined,
    undefined,
    (statusFilter as JournalEntryStatus) || undefined
  );
  const { data: selectedEntry, isLoading: isLoadingEntry } = useJournalEntry(
    selectedEntryId
  );
  const { data: accounts } = useAccounts();

  const createEntry = useCreateJournalEntry();
  const updateEntry = useUpdateJournalEntry();
  const deleteEntry = useDeleteJournalEntry();
  const postEntry = usePostJournalEntry();
  const cancelEntry = useCancelJournalEntry();

  const mutationError =
    createEntry.error ||
    updateEntry.error ||
    deleteEntry.error ||
    postEntry.error ||
    cancelEntry.error;

  const filteredEntries = useMemo(() => {
    if (!entries) return [];
    if (!search.trim()) return entries;
    const term = search.trim().toLowerCase();
    return entries.filter(
      (entry) =>
        (entry.reference ?? "").toLowerCase().includes(term) ||
        (entry.description ?? "").toLowerCase().includes(term) ||
        entry.entry_date.includes(term)
    );
  }, [entries, search]);

  const handleNew = () => {
    setSelectedEntryId(null);
    setFormOpen(true);
  };

  const handleEdit = (id: string) => {
    setSelectedEntryId(id);
    setFormOpen(true);
  };

  const handleCloseForm = () => {
    setFormOpen(false);
    setSelectedEntryId(null);
    createEntry.reset();
    updateEntry.reset();
    deleteEntry.reset();
    postEntry.reset();
    cancelEntry.reset();
  };

  const handleSave = (payload: JournalEntryCreate) => {
    if (selectedEntryId) {
      updateEntry.mutate(
        { id: selectedEntryId, data: payload },
        { onSuccess: handleCloseForm }
      );
    } else {
      createEntry.mutate(payload, { onSuccess: handleCloseForm });
    }
  };

  const handleDelete = (id: string) => {
    if (confirm("Are you sure you want to delete this draft journal entry?")) {
      deleteEntry.mutate(id, { onSuccess: handleCloseForm });
    }
  };

  const handlePost = (id: string) => {
    if (confirm("Post this journal entry?")) {
      postEntry.mutate(id, { onSuccess: handleCloseForm });
    }
  };

  const handleCancel = (id: string) => {
    if (
      confirm(
        "Cancel this posted journal entry? A reversing entry will be created."
      )
    ) {
      cancelEntry.mutate(id, { onSuccess: handleCloseForm });
    }
  };

  const statusBadge = (status: JournalEntry["status"]) => {
    const styles: Record<JournalEntry["status"], string> = {
      draft:
        "inline-flex items-center rounded-full bg-yellow-100 px-2 py-0.5 text-xs font-medium text-yellow-800",
      posted:
        "inline-flex items-center rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-800",
      cancelled:
        "inline-flex items-center rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-800",
    };
    return (
      <span className={styles[status]}>
        {JOURNAL_ENTRY_STATUS_LABELS[status]}
      </span>
    );
  };

  const entryAmount = (entry: JournalEntry) => {
    const debit = entry.lines.reduce(
      (sum, line) => sum + Number(line.debit || 0),
      0
    );
    return formatAmount(debit.toString());
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">
            Journal Entries
          </h2>
          <p className="text-muted-foreground">
            Post and manage manual journal entries.
          </p>
        </div>
        <Button onClick={handleNew}>
          <Plus className="h-4 w-4" />
          New Journal Entry
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Filters</CardTitle>
          <CardDescription>Search and filter journal entries.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col gap-4 sm:flex-row sm:items-center">
            <div className="relative max-w-md flex-1">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="Search reference, description, or date..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                className="pl-9"
              />
            </div>
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value as JournalEntryStatus | "")}
              className="flex h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            >
              <option value="">All statuses</option>
              {(
                ["draft", "posted", "cancelled"] as JournalEntryStatus[]
              ).map((status) => (
                <option key={status} value={status}>
                  {JOURNAL_ENTRY_STATUS_LABELS[status]}
                </option>
              ))}
            </select>
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
              <TableHead>Reference</TableHead>
              <TableHead>Description</TableHead>
              <TableHead>Amount</TableHead>
              <TableHead>Status</TableHead>
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
                  Failed to load journal entries: {error.message}
                </TableCell>
              </TableRow>
            ) : filteredEntries.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={6}
                  className="h-24 text-center text-muted-foreground"
                >
                  No journal entries found.
                </TableCell>
              </TableRow>
            ) : (
              filteredEntries.map((entry) => (
                <TableRow key={entry.id}>
                  <TableCell>{entry.entry_date.slice(0, 10)}</TableCell>
                  <TableCell className="font-medium">
                    {entry.reference ?? "—"}
                  </TableCell>
                  <TableCell>{entry.description ?? "—"}</TableCell>
                  <TableCell className="tabular-nums">
                    {entryAmount(entry)}
                  </TableCell>
                  <TableCell>{statusBadge(entry.status)}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-2">
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        onClick={() => handleEdit(entry.id)}
                        aria-label="View journal entry"
                      >
                        <Eye className="h-4 w-4" />
                      </Button>
                      {entry.status === "draft" && (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() => handlePost(entry.id)}
                        >
                          <Send className="h-4 w-4" />
                          Post
                        </Button>
                      )}
                      {entry.status === "posted" && (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() => handleCancel(entry.id)}
                        >
                          <RotateCcw className="h-4 w-4" />
                          Cancel
                        </Button>
                      )}
                      {entry.status === "draft" && (
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          onClick={() => handleDelete(entry.id)}
                          aria-label="Delete journal entry"
                        >
                          <Trash2 className="h-4 w-4" />
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

      <JournalEntryForm
        entry={selectedEntry ?? null}
        accounts={accounts ?? []}
        isOpen={formOpen}
        isSaving={
          createEntry.isPending ||
          updateEntry.isPending ||
          deleteEntry.isPending ||
          postEntry.isPending ||
          cancelEntry.isPending ||
          (!!selectedEntryId && isLoadingEntry)
        }
        error={mutationError}
        onClose={handleCloseForm}
        onSave={handleSave}
        onPost={handlePost}
        onCancel={handleCancel}
        onDelete={handleDelete}
      />
    </div>
  );
}
