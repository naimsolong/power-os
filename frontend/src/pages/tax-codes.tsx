import { useMemo, useState } from "react";
import { Pencil, Plus, Search, Trash2 } from "lucide-react";
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
  useTaxCodes,
  useCreateTaxCode,
  useUpdateTaxCode,
  useDeleteTaxCode,
  TAX_TYPE_LABELS,
  TAX_TYPE_OPTIONS,
  type TaxCode,
  type TaxCodeCreate,
  type TaxType,
} from "@/hooks/use-tax-codes";

interface FormState {
  code: string;
  description: string;
  rate: string;
  tax_type: TaxType;
}

function emptyForm(): FormState {
  return {
    code: "",
    description: "",
    rate: "",
    tax_type: "sst",
  };
}

function formFromTaxCode(taxCode: TaxCode): FormState {
  return {
    code: taxCode.code,
    description: taxCode.description,
    rate: taxCode.rate.toString(),
    tax_type: taxCode.tax_type,
  };
}

export function TaxCodesPage() {
  const [search, setSearch] = useState("");
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [form, setForm] = useState<FormState>(emptyForm());
  const [formError, setFormError] = useState<string | null>(null);

  const { data: taxCodes, isLoading, error } = useTaxCodes();
  const createTaxCode = useCreateTaxCode();
  const updateTaxCode = useUpdateTaxCode();
  const deleteTaxCode = useDeleteTaxCode();

  const selectedTaxCode = useMemo(() => {
    if (!selectedId || !taxCodes) return null;
    return taxCodes.find((tc) => tc.id === selectedId) ?? null;
  }, [selectedId, taxCodes]);

  const filteredTaxCodes = useMemo(() => {
    if (!taxCodes) return [];
    if (!search.trim()) return taxCodes;
    const term = search.toLowerCase();
    return taxCodes.filter(
      (tc) =>
        tc.code.toLowerCase().includes(term) ||
        tc.description.toLowerCase().includes(term)
    );
  }, [taxCodes, search]);

  const handleNew = () => {
    setSelectedId(null);
    setForm(emptyForm());
    setFormError(null);
    setDrawerOpen(true);
  };

  const handleEdit = (taxCode: TaxCode) => {
    setSelectedId(taxCode.id);
    setForm(formFromTaxCode(taxCode));
    setFormError(null);
    setDrawerOpen(true);
  };

  const handleCloseDrawer = () => {
    setDrawerOpen(false);
    setSelectedId(null);
    setForm(emptyForm());
    setFormError(null);
    createTaxCode.reset();
    updateTaxCode.reset();
  };

  const validateForm = (): TaxCodeCreate | null => {
    const code = form.code.trim();
    const description = form.description.trim();
    const rate = parseFloat(form.rate);

    if (!code) {
      setFormError("Code is required.");
      return null;
    }
    if (!description) {
      setFormError("Description is required.");
      return null;
    }
    if (Number.isNaN(rate) || rate < 0) {
      setFormError("Rate must be a number greater than or equal to 0.");
      return null;
    }

    return {
      code,
      description,
      rate,
      tax_type: form.tax_type,
    };
  };

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setFormError(null);

    const payload = validateForm();
    if (!payload) return;

    try {
      if (selectedId) {
        await updateTaxCode.mutateAsync({ id: selectedId, data: payload });
      } else {
        await createTaxCode.mutateAsync(payload);
      }
      handleCloseDrawer();
    } catch (err) {
      setFormError(
        err instanceof Error ? err.message : "An unexpected error occurred."
      );
    }
  };

  const handleDelete = (id: string) => {
    if (confirm("Are you sure you want to delete this tax code?")) {
      deleteTaxCode.mutate(id);
    }
  };

  const formatRate = (rate: number) => {
    return `${rate.toLocaleString(undefined, {
      minimumFractionDigits: 0,
      maximumFractionDigits: 2,
    })}%`;
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Tax Codes</h2>
          <p className="text-muted-foreground">
            Manage SST and service tax codes.
          </p>
        </div>
        <Button onClick={handleNew}>
          <Plus className="h-4 w-4" />
          New Tax Code
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Search</CardTitle>
          <CardDescription>Find tax codes by code or description.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Search className="h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search tax codes..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="max-w-md"
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Tax Codes</CardTitle>
          <CardDescription>
            Active tax codes for the current workspace.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <p className="text-sm text-muted-foreground">Loading tax codes...</p>
          ) : error ? (
            <p className="text-sm text-destructive">
              Failed to load tax codes: {error.message}
            </p>
          ) : filteredTaxCodes.length === 0 ? (
            <p className="text-sm text-muted-foreground">No tax codes found.</p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Code</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead className="text-right">Rate</TableHead>
                  <TableHead className="w-24 text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredTaxCodes.map((taxCode) => (
                  <TableRow key={taxCode.id}>
                    <TableCell className="font-medium">{taxCode.code}</TableCell>
                    <TableCell>{taxCode.description}</TableCell>
                    <TableCell>{TAX_TYPE_LABELS[taxCode.tax_type]}</TableCell>
                    <TableCell className="text-right">
                      {formatRate(taxCode.rate)}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-2">
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleEdit(taxCode)}
                        >
                          <Pencil className="h-4 w-4" />
                          <span className="sr-only">Edit</span>
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleDelete(taxCode.id)}
                        >
                          <Trash2 className="h-4 w-4" />
                          <span className="sr-only">Delete</span>
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {drawerOpen && (
        <div className="fixed inset-0 z-50 flex justify-end">
          <div
            className="absolute inset-0 bg-black/50"
            onClick={handleCloseDrawer}
            role="presentation"
          />
          <div className="relative z-10 w-full max-w-md bg-background shadow-xl border-l h-full overflow-auto p-6">
            <h3 className="text-lg font-semibold mb-4">
              {selectedTaxCode ? "Edit Tax Code" : "New Tax Code"}
            </h3>
            <form onSubmit={handleSubmit} className="space-y-4">
              {(formError ||
                createTaxCode.error ||
                updateTaxCode.error) && (
                <div className="rounded-md border border-destructive bg-destructive/10 p-3 text-sm text-destructive">
                  {formError ||
                    (createTaxCode.error?.message ??
                      updateTaxCode.error?.message)}
                </div>
              )}

              <div className="space-y-2">
                <label htmlFor="code" className="text-sm font-medium">
                  Code
                </label>
                <Input
                  id="code"
                  value={form.code}
                  onChange={(e) =>
                    setForm((prev) => ({ ...prev, code: e.target.value }))
                  }
                  placeholder="e.g. SR"
                  disabled={createTaxCode.isPending || updateTaxCode.isPending}
                />
              </div>

              <div className="space-y-2">
                <label htmlFor="description" className="text-sm font-medium">
                  Description
                </label>
                <Input
                  id="description"
                  value={form.description}
                  onChange={(e) =>
                    setForm((prev) => ({
                      ...prev,
                      description: e.target.value,
                    }))
                  }
                  placeholder="e.g. Standard Rate"
                  disabled={createTaxCode.isPending || updateTaxCode.isPending}
                />
              </div>

              <div className="space-y-2">
                <label htmlFor="tax-type" className="text-sm font-medium">
                  Tax Type
                </label>
                <select
                  id="tax-type"
                  value={form.tax_type}
                  onChange={(e) =>
                    setForm((prev) => ({
                      ...prev,
                      tax_type: e.target.value as TaxType,
                    }))
                  }
                  disabled={createTaxCode.isPending || updateTaxCode.isPending}
                  className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                >
                  {TAX_TYPE_OPTIONS.map((type) => (
                    <option key={type} value={type}>
                      {TAX_TYPE_LABELS[type]}
                    </option>
                  ))}
                </select>
              </div>

              <div className="space-y-2">
                <label htmlFor="rate" className="text-sm font-medium">
                  Rate (%)
                </label>
                <Input
                  id="rate"
                  type="number"
                  min={0}
                  step="0.01"
                  value={form.rate}
                  onChange={(e) =>
                    setForm((prev) => ({ ...prev, rate: e.target.value }))
                  }
                  placeholder="e.g. 10"
                  disabled={createTaxCode.isPending || updateTaxCode.isPending}
                />
              </div>

              <div className="flex justify-end gap-2 pt-4">
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleCloseDrawer}
                  disabled={createTaxCode.isPending || updateTaxCode.isPending}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  disabled={createTaxCode.isPending || updateTaxCode.isPending}
                >
                  {createTaxCode.isPending || updateTaxCode.isPending
                    ? "Saving..."
                    : "Save"}
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
