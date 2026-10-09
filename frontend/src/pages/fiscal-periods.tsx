import { useState } from "react";
import { Lock, LockOpen, Plus, Calendar } from "lucide-react";
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
import {
  useFiscalYears,
  useCreateFiscalYear,
  useCloseAccountingPeriod,
  useReopenAccountingPeriod,
  type FiscalYearCreate,
} from "@/hooks/use-fiscal-periods";

function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString();
}

export function FiscalPeriodsPage() {
  const { data: fiscalYears, isLoading } = useFiscalYears();
  const createFiscalYear = useCreateFiscalYear();
  const closePeriod = useCloseAccountingPeriod();
  const reopenPeriod = useReopenAccountingPeriod();

  const [form, setForm] = useState<FiscalYearCreate>({
    name: "",
    start_date: "",
    end_date: "",
  });

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    createFiscalYear.mutate(form, {
      onSuccess: () => {
        setForm({ name: "", start_date: "", end_date: "" });
      },
    });
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">
          Fiscal Years & Accounting Periods
        </h2>
        <p className="text-muted-foreground">
          Manage fiscal years and close or reopen accounting periods.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <Plus className="h-4 w-4" />
            Create Fiscal Year
          </CardTitle>
          <CardDescription>
            Creates a fiscal year and auto-generates 12 monthly periods.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            {(createFiscalYear.error ||
              closePeriod.error ||
              reopenPeriod.error) && (
              <div className="rounded-md border border-destructive bg-destructive/10 p-3 text-sm text-destructive">
                {(
                  createFiscalYear.error ??
                  closePeriod.error ??
                  reopenPeriod.error
                )?.message}
              </div>
            )}

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <div className="space-y-2">
                <label htmlFor="fy-name" className="text-sm font-medium">
                  Name
                </label>
                <Input
                  id="fy-name"
                  value={form.name}
                  onChange={(e) =>
                    setForm((prev) => ({ ...prev, name: e.target.value }))
                  }
                  placeholder="e.g. Fiscal Year 2026"
                  disabled={createFiscalYear.isPending}
                />
              </div>
              <div className="space-y-2">
                <label htmlFor="fy-start" className="text-sm font-medium">
                  Start Date
                </label>
                <Input
                  id="fy-start"
                  type="date"
                  value={form.start_date}
                  onChange={(e) =>
                    setForm((prev) => ({ ...prev, start_date: e.target.value }))
                  }
                  disabled={createFiscalYear.isPending}
                />
              </div>
              <div className="space-y-2">
                <label htmlFor="fy-end" className="text-sm font-medium">
                  End Date
                </label>
                <Input
                  id="fy-end"
                  type="date"
                  value={form.end_date}
                  onChange={(e) =>
                    setForm((prev) => ({ ...prev, end_date: e.target.value }))
                  }
                  disabled={createFiscalYear.isPending}
                />
              </div>
            </div>

            <div className="flex justify-end">
              <Button
                type="submit"
                disabled={
                  createFiscalYear.isPending ||
                  !form.name ||
                  !form.start_date ||
                  !form.end_date
                }
              >
                <Plus className="h-4 w-4" />
                {createFiscalYear.isPending ? "Creating..." : "Create Fiscal Year"}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      {isLoading ? (
        <p className="text-sm text-muted-foreground">Loading fiscal years…</p>
      ) : !fiscalYears || fiscalYears.length === 0 ? (
        <p className="text-sm text-muted-foreground">No fiscal years found.</p>
      ) : (
        <div className="space-y-4">
          {fiscalYears.map((fiscalYear) => (
            <Card key={fiscalYear.id}>
              <CardHeader>
                <CardTitle className="text-base flex items-center gap-2">
                  <Calendar className="h-4 w-4" />
                  {fiscalYear.name}
                </CardTitle>
                <CardDescription>
                  {formatDate(fiscalYear.start_date)} -{" "}
                  {formatDate(fiscalYear.end_date)}
                  {fiscalYear.is_closed && (
                    <span className="ml-2 inline-flex items-center rounded-full border border-destructive bg-destructive/10 px-2 py-0.5 text-xs font-medium text-destructive">
                      Closed
                    </span>
                  )}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Period</TableHead>
                      <TableHead>Start</TableHead>
                      <TableHead>End</TableHead>
                      <TableHead>Status</TableHead>
                      <TableHead className="text-right">Actions</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {fiscalYear.periods.map((period) => (
                      <TableRow key={period.id}>
                        <TableCell className="font-medium">
                          {period.name}
                        </TableCell>
                        <TableCell>{formatDate(period.start_date)}</TableCell>
                        <TableCell>{formatDate(period.end_date)}</TableCell>
                        <TableCell>
                          {period.is_closed ? (
                            <span className="inline-flex items-center rounded-full border border-destructive bg-destructive/10 px-2 py-0.5 text-xs font-medium text-destructive">
                              Closed
                            </span>
                          ) : (
                            <span className="inline-flex items-center rounded-full border border-green-600 bg-green-600/10 px-2 py-0.5 text-xs font-medium text-green-700">
                              Open
                            </span>
                          )}
                        </TableCell>
                        <TableCell className="text-right">
                          {period.is_closed ? (
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => reopenPeriod.mutate(period.id)}
                              disabled={reopenPeriod.isPending}
                            >
                              <LockOpen className="h-4 w-4" />
                              Reopen
                            </Button>
                          ) : (
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => closePeriod.mutate(period.id)}
                              disabled={closePeriod.isPending}
                            >
                              <Lock className="h-4 w-4" />
                              Close
                            </Button>
                          )}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
