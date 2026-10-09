import { useMemo, useState } from "react";
import { Archive, Edit, Plus, Search } from "lucide-react";
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
import {
  useAccounts,
  useCreateAccount,
  useUpdateAccount,
  useArchiveAccount,
  ACCOUNT_TYPE_OPTIONS,
  ACCOUNT_TYPE_LABELS,
  type Account,
  type AccountCreate,
  type AccountType,
} from "@/hooks/use-accounts";

function getIndentLevel(account: Account, accounts: Account[]): number {
  let level = 0;
  let current = account.parent_account_id;
  const seen = new Set<string>();
  while (current) {
    if (seen.has(current)) break;
    seen.add(current);
    level += 1;
    const parent = accounts.find((a) => a.id === current);
    current = parent?.parent_account_id ?? null;
  }
  return level;
}

export function ChartOfAccountsPage() {
  const [search, setSearch] = useState("");
  const [showArchived, setShowArchived] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [selectedAccountId, setSelectedAccountId] = useState<string | null>(null);

  const { data: accounts, isLoading } = useAccounts();
  const createAccount = useCreateAccount();
  const updateAccount = useUpdateAccount();
  const archiveAccount = useArchiveAccount();

  const filteredAccounts = useMemo(() => {
    if (!accounts) return [];
    let list = accounts;
    if (!showArchived) {
      list = list.filter((a) => a.is_active);
    }
    if (search.trim()) {
      const term = search.trim().toLowerCase();
      list = list.filter(
        (a) =>
          a.code.toLowerCase().includes(term) ||
          a.name.toLowerCase().includes(term) ||
          a.account_type.toLowerCase().includes(term)
      );
    }
    return list;
  }, [accounts, search, showArchived]);

  const selectedAccount =
    selectedAccountId && accounts
      ? accounts.find((a) => a.id === selectedAccountId) ?? null
      : null;

  const handleAdd = () => {
    setSelectedAccountId(null);
    setModalOpen(true);
  };

  const handleEdit = (account: Account) => {
    setSelectedAccountId(account.id);
    setModalOpen(true);
  };

  const handleClose = () => {
    setModalOpen(false);
    setSelectedAccountId(null);
  };

  const handleSave = (payload: AccountCreate) => {
    if (selectedAccountId) {
      updateAccount.mutate(
        { id: selectedAccountId, data: payload },
        { onSuccess: handleClose }
      );
    } else {
      createAccount.mutate(payload, { onSuccess: handleClose });
    }
  };

  const handleArchive = (account: Account) => {
    if (
      confirm(
        `Archive account "${account.code} - ${account.name}"? It will remain visible in historical reports.`
      )
    ) {
      archiveAccount.mutate(account.id);
    }
  };

  const isSaving = createAccount.isPending || updateAccount.isPending;
  const error =
    createAccount.error || updateAccount.error || archiveAccount.error;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">
            Chart of Accounts
          </h2>
          <p className="text-muted-foreground">
            Manage your workspace chart of accounts.
          </p>
        </div>
        <Button onClick={handleAdd}>
          <Plus className="mr-2 h-4 w-4" />
          Add Account
        </Button>
      </div>

      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="relative max-w-sm">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="Search accounts..."
            className="pl-9"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <label className="flex items-center gap-2 text-sm text-muted-foreground">
          <input
            type="checkbox"
            checked={showArchived}
            onChange={(e) => setShowArchived(e.target.checked)}
            className="h-4 w-4 rounded border-input"
          />
          Show archived accounts
        </label>
      </div>

      {error && (
        <div className="rounded-md border border-destructive/50 bg-destructive/10 px-4 py-2 text-sm text-destructive">
          {error.message}
        </div>
      )}

      <Card>
        <CardHeader className="pb-2">
          <CardTitle>Accounts</CardTitle>
          <CardDescription>
            {showArchived
              ? "All accounts including archived ones."
              : "Active accounts only."}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="text-sm text-muted-foreground py-8">
              Loading accounts...
            </div>
          ) : filteredAccounts.length === 0 ? (
            <div className="text-sm text-muted-foreground py-8">
              No accounts found.
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Code</TableHead>
                  <TableHead>Name</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Parent</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredAccounts.map((account) => {
                  const indent = getIndentLevel(account, accounts ?? []);
                  return (
                    <TableRow
                      key={account.id}
                      className={
                        !account.is_active ? "opacity-60" : undefined
                      }
                    >
                      <TableCell>
                        <span
                          style={{ paddingLeft: `${indent * 1.5}rem` }}
                          className="inline-block"
                        >
                          {account.code}
                        </span>
                      </TableCell>
                      <TableCell>{account.name}</TableCell>
                      <TableCell>
                        {ACCOUNT_TYPE_LABELS[account.account_type]}
                      </TableCell>
                      <TableCell>
                        {account.parent_code
                          ? `${account.parent_code} - ${account.parent_name}`
                          : "—"}
                      </TableCell>
                      <TableCell>
                        {account.is_active ? (
                          <span className="inline-flex items-center rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-800">
                            Active
                          </span>
                        ) : (
                          <span className="inline-flex items-center rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-800">
                            Archived
                          </span>
                        )}
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-2">
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => handleEdit(account)}
                            aria-label="Edit account"
                          >
                            <Edit className="h-4 w-4" />
                          </Button>
                          {account.is_active && (
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => handleArchive(account)}
                              aria-label="Archive account"
                            >
                              <Archive className="h-4 w-4" />
                            </Button>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {modalOpen && (
        <AccountModal
          account={selectedAccount}
          accounts={accounts ?? []}
          isSaving={isSaving}
          error={error}
          onClose={handleClose}
          onSave={handleSave}
        />
      )}
    </div>
  );
}

interface AccountModalProps {
  account: Account | null;
  accounts: Account[];
  isSaving: boolean;
  error: Error | null;
  onClose: () => void;
  onSave: (payload: AccountCreate) => void;
}

function AccountModal({
  account,
  accounts,
  isSaving,
  error,
  onClose,
  onSave,
}: AccountModalProps) {
  const [code, setCode] = useState(account?.code ?? "");
  const [name, setName] = useState(account?.name ?? "");
  const [accountType, setAccountType] = useState<AccountType>(
    account?.account_type ?? "asset"
  );
  const [parentId, setParentId] = useState<string>(
    account?.parent_account_id ?? ""
  );

  const isEditing = account !== null;

  const parentOptions = accounts.filter(
    (a) =>
      a.is_active &&
      (!isEditing || a.id !== account.id)
  );

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave({
      code: code.trim(),
      name: name.trim(),
      account_type: accountType,
      parent_account_id: parentId || null,
    });
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>
            {isEditing ? "Edit Account" : "Add Account"}
          </CardTitle>
          <CardDescription>
            {isEditing
              ? "Update the account details below."
              : "Create a new account in your chart of accounts."}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <div className="rounded-md border border-destructive/50 bg-destructive/10 px-4 py-2 text-sm text-destructive">
                {error.message}
              </div>
            )}
            <div className="space-y-2">
              <label htmlFor="account-code" className="text-sm font-medium">
                Code
              </label>
              <Input
                id="account-code"
                value={code}
                onChange={(e) => setCode(e.target.value)}
                placeholder="e.g. 1000"
                required
              />
            </div>
            <div className="space-y-2">
              <label htmlFor="account-name" className="text-sm font-medium">
                Name
              </label>
              <Input
                id="account-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g. Cash on Hand"
                required
              />
            </div>
            <div className="space-y-2">
              <label
                htmlFor="account-type"
                className="text-sm font-medium"
              >
                Type
              </label>
              <select
                id="account-type"
                value={accountType}
                onChange={(e) =>
                  setAccountType(e.target.value as AccountType)
                }
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              >
                {ACCOUNT_TYPE_OPTIONS.map((type) => (
                  <option key={type} value={type}>
                    {ACCOUNT_TYPE_LABELS[type]}
                  </option>
                ))}
              </select>
            </div>
            <div className="space-y-2">
              <label
                htmlFor="account-parent"
                className="text-sm font-medium"
              >
                Parent Account
              </label>
              <select
                id="account-parent"
                value={parentId}
                onChange={(e) => setParentId(e.target.value)}
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              >
                <option value="">None</option>
                {parentOptions.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.code} - {a.name}
                  </option>
                ))}
              </select>
            </div>
            <div className="flex justify-end gap-2 pt-2">
              <Button
                type="button"
                variant="outline"
                onClick={onClose}
                disabled={isSaving}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={isSaving}>
                {isSaving ? "Saving..." : isEditing ? "Save" : "Create"}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
