import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useQuery } from "@tanstack/react-query";
import { companies, contacts, deals, employees, invoices } from "@/data/placeholders";

function useSummary() {
  return useQuery({
    queryKey: ["summary"],
    queryFn: async () => {
      return {
        contacts: contacts.length,
        companies: companies.length,
        deals: deals.length,
        invoices: invoices.length,
        employees: employees.length,
        revenue: deals
          .filter((d) => d.stage === "closed-won")
          .reduce((sum, d) => sum + d.value, 0),
      };
    },
  });
}

export function DashboardPage() {
  const { data } = useSummary();

  const stats = [
    { label: "Contacts", value: data?.contacts ?? 0 },
    { label: "Companies", value: data?.companies ?? 0 },
    { label: "Active Deals", value: data?.deals ?? 0 },
    { label: "Invoices", value: data?.invoices ?? 0 },
    { label: "Employees", value: data?.employees ?? 0 },
    { label: "Closed Revenue", value: `$${(data?.revenue ?? 0).toLocaleString()}` },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">Dashboard</h2>
        <p className="text-muted-foreground">Overview of your business.</p>
      </div>
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {stats.map((stat) => (
          <Card key={stat.label}>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                {stat.label}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-3xl font-bold">{stat.value}</div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
