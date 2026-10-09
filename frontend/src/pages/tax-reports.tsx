import { useMemo, useState } from "react";
import { FileText } from "lucide-react";
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
import { useSst02Report } from "@/hooks/use-tax-reports";

function formatDateInput(date: Date): string {
  return date.toISOString().slice(0, 10);
}

function firstDayOfMonth(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

function lastDayOfMonth(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth() + 1, 0);
}

function formatMoney(value: string | number): string {
  const numeric = typeof value === "number" ? value : Number(value);
  return new Intl.NumberFormat("en-MY", {
    style: "currency",
    currency: "MYR",
  }).format(numeric);
}

export function TaxReportsPage() {
  const today = useMemo(() => new Date(), []);
  const [from, setFrom] = useState<string>(formatDateInput(firstDayOfMonth(today)));
  const [to, setTo] = useState<string>(formatDateInput(lastDayOfMonth(today)));
  const [params, setParams] = useState<{ from: string; to: string } | null>(null);

  const { data: report, isLoading, error } = useSst02Report(params);

  const handleRun = () => {
    setParams({ from, to });
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">Tax Reports</h2>
        <p className="text-muted-foreground">SST-02 prep report by tax code.</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <FileText className="h-4 w-4" />
            SST-02 Report
          </CardTitle>
          <CardDescription>
            Select a date range to aggregate posted invoice tax amounts.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex flex-col gap-4 sm:flex-row sm:items-end">
            <div className="space-y-2">
              <label htmlFor="sst-from" className="text-sm font-medium">
                From
              </label>
              <Input
                id="sst-from"
                type="date"
                value={from}
                onChange={(e) => setFrom(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <label htmlFor="sst-to" className="text-sm font-medium">
                To
              </label>
              <Input
                id="sst-to"
                type="date"
                value={to}
                onChange={(e) => setTo(e.target.value)}
              />
            </div>
            <Button onClick={handleRun} disabled={!from || !to}>
              Run Report
            </Button>
          </div>

          {error && (
            <div className="rounded-md border border-destructive bg-destructive/10 p-3 text-sm text-destructive">
              {error.message}
            </div>
          )}

          {isLoading && (
            <div className="text-sm text-muted-foreground">Loading...</div>
          )}

          {report && (
            <div className="space-y-4">
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
                <div className="rounded-md border p-4">
                  <p className="text-sm text-muted-foreground">Taxable Amount</p>
                  <p className="text-lg font-semibold">
                    {formatMoney(report.total_taxable_amount)}
                  </p>
                </div>
                <div className="rounded-md border p-4">
                  <p className="text-sm text-muted-foreground">Tax Amount</p>
                  <p className="text-lg font-semibold">
                    {formatMoney(report.total_tax_amount)}
                  </p>
                </div>
                <div className="rounded-md border p-4">
                  <p className="text-sm text-muted-foreground">Total (incl. tax)</p>
                  <p className="text-lg font-semibold">
                    {formatMoney(
                      Number(report.total_taxable_amount) +
                        Number(report.total_tax_amount)
                    )}
                  </p>
                </div>
              </div>

              <div className="rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Code</TableHead>
                      <TableHead>Description</TableHead>
                      <TableHead className="text-right">Rate</TableHead>
                      <TableHead className="text-right">Taxable Amount</TableHead>
                      <TableHead className="text-right">Tax Amount</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {report.lines.length === 0 ? (
                      <TableRow>
                        <TableCell
                          colSpan={5}
                          className="h-24 text-center text-muted-foreground"
                        >
                          No taxable lines found for this period.
                        </TableCell>
                      </TableRow>
                    ) : (
                      report.lines.map((line) => (
                        <TableRow key={line.tax_code_id}>
                          <TableCell className="font-medium">{line.code}</TableCell>
                          <TableCell>{line.description}</TableCell>
                          <TableCell className="text-right">
                            {(Number(line.rate) * 100).toFixed(0)}%
                          </TableCell>
                          <TableCell className="text-right">
                            {formatMoney(line.taxable_amount)}
                          </TableCell>
                          <TableCell className="text-right">
                            {formatMoney(line.tax_amount)}
                          </TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
