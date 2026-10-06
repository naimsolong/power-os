import { deals } from "@/data/placeholders";
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

export function DealsPage() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Deals</h2>
          <p className="text-muted-foreground">Track your pipeline.</p>
        </div>
        <Button>Add Deal</Button>
      </div>
      <div className="flex items-center gap-2">
        <Input placeholder="Search deals..." className="max-w-sm" />
      </div>
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Company</TableHead>
              <TableHead>Value</TableHead>
              <TableHead>Stage</TableHead>
              <TableHead>Probability</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {deals.map((deal) => (
              <TableRow key={deal.id}>
                <TableCell className="font-medium">{deal.name}</TableCell>
                <TableCell>{deal.company}</TableCell>
                <TableCell>${deal.value.toLocaleString()}</TableCell>
                <TableCell className="capitalize">{deal.stage.replace("-", " ")}</TableCell>
                <TableCell>{deal.probability}%</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
