import { useMemo, useState } from "react";
import { BarChart3 } from "lucide-react";
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
  useTrialBalance,
  useProfitLoss,
  useBalanceSheet,
  type TrialBalanceLine,
} from "@/hooks/use-reports";
import { cn } from "@/lib/utils";

type ReportTab = "trial-balance" | "profit-loss" | "balance-sheet";

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

function ReportTabs({
  active,
  onChange,
}: {
  active: ReportTab;
  onChange: (tab: ReportTab) => void;
}) {
  const tabs: { id: ReportTab; label: string }[] = [
    { id: "trial-balance", label: "Trial Balance" },
    { id: "profit-loss", label: "Profit & Loss" },
    { id: "balance-sheet", label: "Balance Sheet" },
  ];

  return (
    <div className="flex gap-2 border-b pb-2">
      {tabs.map((tab) => (
        <Button
          key={tab.id}
          variant={active === tab.id ? "default" : "ghost"}
          size="sm"
          onClick={() => onChange(tab.id)}
        >
          {tab.label}
        </Button>
      ))}
    </div>
  );
}

function TrialBalanceReport() {
  const today = useMemo(() => new Date(), []);
  const [asOf, setAsOf] = useState<string>(formatDateInput(today));
  const [params, setParams] = useState<{ as_of: string } | null>(null);

  const { data: report, isLoading, error } = useTrialBalance(params);

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end">
        <div className="space-y-2">
          <label htmlFor="tb-as-of" className="text-sm font-medium">
            As of
          </label>
          <Input
            id="tb-as-of"
            type="date"
            value={asOf}
            onChange={(e) => setAsOf(e.target.value)}
          />
        </div>
        <Button onClick={() => setParams({ as_of: asOf })} disabled={!asOf}>
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
        <div className="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Code</TableHead>
                <TableHead>Name</TableHead>
                <TableHead>Type</TableHead>
                <TableHead className="text-right">Debit</TableHead>
                <TableHead className="text-right">Credit</TableHead>
                <TableHead className="text-right">Balance</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {report.lines.length === 0 ? (
                <TableRow>
                  <TableCell
                    colSpan={6}
                    className="h-24 text-center text-muted-foreground"
                  >
                    No accounts found.
                  </TableCell>
                </TableRow>
              ) : (
                report.lines.map((line: TrialBalanceLine) => (
                  <TableRow key={line.account_id}>
                    <TableCell className="font-medium">
                      {line.account_code}
                    </TableCell>
                    <TableCell>{line.account_name}</TableCell>
                    <TableCell className="capitalize">
                      {line.account_type}
                    </TableCell>
                    <TableCell className="text-right">
                      {formatMoney(line.debit)}
                    </TableCell>
                    <TableCell className="text-right">
                      {formatMoney(line.credit)}
                    </TableCell>
                    <TableCell
                      className={cn(
                        "text-right font-medium",
                        Number(line.balance) < 0 && "text-destructive"
                      )}
                    >
                      {formatMoney(line.balance)}
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}

function ProfitLossReport() {
  const today = useMemo(() => new Date(), []);
  const [from, setFrom] = useState<string>(formatDateInput(firstDayOfMonth(today)));
  const [to, setTo] = useState<string>(formatDateInput(lastDayOfMonth(today)));
  const [params, setParams] = useState<{ from: string; to: string } | null>(null);

  const { data: report, isLoading, error } = useProfitLoss(params);

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end">
        <div className="space-y-2">
          <label htmlFor="pl-from" className="text-sm font-medium">
            From
          </label>
          <Input
            id="pl-from"
            type="date"
            value={from}
            onChange={(e) => setFrom(e.target.value)}
          />
        </div>
        <div className="space-y-2">
          <label htmlFor="pl-to" className="text-sm font-medium">
            To
          </label>
          <Input
            id="pl-to"
            type="date"
            value={to}
            onChange={(e) => setTo(e.target.value)}
          />
        </div>
        <Button onClick={() => setParams({ from, to })} disabled={!from || !to}>
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
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <div className="rounded-md border p-4">
            <p className="text-sm text-muted-foreground">Revenue</p>
            <p className="text-lg font-semibold">
              {formatMoney(report.revenue)}
            </p>
          </div>
          <div className="rounded-md border p-4">
            <p className="text-sm text-muted-foreground">Expenses</p>
            <p className="text-lg font-semibold">
              {formatMoney(report.expenses)}
            </p>
          </div>
          <div className="rounded-md border p-4">
            <p className="text-sm text-muted-foreground">Net Profit</p>
            <p
              className={cn(
                "text-lg font-semibold",
                Number(report.net_profit) < 0 && "text-destructive"
              )}
            >
              {formatMoney(report.net_profit)}
            </p>
          </div>
        </div>
      )}
    </div>
  );
}

function BalanceSheetReport() {
  const today = useMemo(() => new Date(), []);
  const [asOf, setAsOf] = useState<string>(formatDateInput(today));
  const [params, setParams] = useState<{ as_of: string } | null>(null);

  const { data: report, isLoading, error } = useBalanceSheet(params);

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end">
        <div className="space-y-2">
          <label htmlFor="bs-as-of" className="text-sm font-medium">
            As of
          </label>
          <Input
            id="bs-as-of"
            type="date"
            value={asOf}
            onChange={(e) => setAsOf(e.target.value)}
          />
        </div>
        <Button onClick={() => setParams({ as_of: asOf })} disabled={!asOf}>
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
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <div className="rounded-md border p-4">
              <p className="text-sm text-muted-foreground">Assets</p>
              <p className="text-lg font-semibold">
                {formatMoney(report.assets)}
              </p>
            </div>
            <div className="rounded-md border p-4">
              <p className="text-sm text-muted-foreground">Liabilities</p>
              <p className="text-lg font-semibold">
                {formatMoney(report.liabilities)}
              </p>
            </div>
            <div className="rounded-md border p-4">
              <p className="text-sm text-muted-foreground">Equity Accounts</p>
              <p className="text-lg font-semibold">
                {formatMoney(report.equity)}
              </p>
            </div>
            <div className="rounded-md border p-4">
              <p className="text-sm text-muted-foreground">Retained Earnings</p>
              <p className="text-lg font-semibold">
                {formatMoney(report.retained_earnings)}
              </p>
            </div>
          </div>

          <div className="rounded-md border p-4">
            <p className="text-sm text-muted-foreground">
              Check (Assets − Liabilities − Equity)
            </p>
            <p
              className={cn(
                "text-lg font-semibold",
                Number(report.check) !== 0 && "text-destructive"
              )}
            >
              {formatMoney(report.check)}
            </p>
          </div>
        </div>
      )}
    </div>
  );
}

export function ReportsPage() {
  const [activeTab, setActiveTab] = useState<ReportTab>("trial-balance");

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">Reports</h2>
        <p className="text-muted-foreground">
          Financial statements from the general ledger.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <BarChart3 className="h-4 w-4" />
            Financial Reports
          </CardTitle>
          <CardDescription>
            Select a report and date range to view balances.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <ReportTabs active={activeTab} onChange={setActiveTab} />

          {activeTab === "trial-balance" && <TrialBalanceReport />}
          {activeTab === "profit-loss" && <ProfitLossReport />}
          {activeTab === "balance-sheet" && <BalanceSheetReport />}
        </CardContent>
      </Card>
    </div>
  );
}
