import { useMemo, useState } from "react";
import { Plus, Search } from "lucide-react";
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
import { PartyDrawer } from "@/components/parties/party-drawer";
import {
  useParties,
  useParty,
  useCreateParty,
  useUpdateParty,
  useDeleteParty,
} from "@/hooks/use-parties";
import {
  type PartyCreate,
  type PartyType,
  PARTY_TYPE_LABELS,
  PARTY_TYPE_OPTIONS,
} from "@/lib/api";

interface PartyListProps {
  title: string;
  singularTitle: string;
  description: string;
  newLabel: string;
  defaultPartyType: PartyType;
}

export function PartyList({
  title,
  singularTitle,
  description,
  newLabel,
  defaultPartyType,
}: PartyListProps) {
  const [partyType, setPartyType] = useState<PartyType | undefined>(
    defaultPartyType
  );
  const [search, setSearch] = useState("");
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [selectedPartyId, setSelectedPartyId] = useState<string | null>(null);

  const { data: parties, isLoading, error } = useParties(partyType, search);
  const { data: selectedParty, isLoading: isLoadingParty } = useParty(
    selectedPartyId
  );
  const createParty = useCreateParty();
  const updateParty = useUpdateParty();
  const deleteParty = useDeleteParty();

  const mutationError =
    createParty.error || updateParty.error || deleteParty.error;

  const filteredParties = useMemo(() => {
    if (!parties) return [];
    if (!search.trim()) return parties;
    const term = search.toLowerCase();
    return parties.filter(
      (party) =>
        party.name.toLowerCase().includes(term) ||
        (party.email ?? "").toLowerCase().includes(term) ||
        (party.phone ?? "").toLowerCase().includes(term) ||
        (party.address ?? "").toLowerCase().includes(term) ||
        (party.tin ?? "").toLowerCase().includes(term)
    );
  }, [parties, search]);

  const handleNew = () => {
    setSelectedPartyId(null);
    setDrawerOpen(true);
  };

  const handleEdit = (id: string) => {
    setSelectedPartyId(id);
    setDrawerOpen(true);
  };

  const handleCloseDrawer = () => {
    setDrawerOpen(false);
    setSelectedPartyId(null);
    createParty.reset();
    updateParty.reset();
    deleteParty.reset();
  };

  const handleSave = (form: PartyCreate) => {
    if (selectedPartyId) {
      updateParty.mutate(
        { id: selectedPartyId, data: form },
        { onSuccess: handleCloseDrawer }
      );
    } else {
      createParty.mutate(form, { onSuccess: handleCloseDrawer });
    }
  };

  const handleDelete = (id: string) => {
    if (confirm("Are you sure you want to delete this party?")) {
      deleteParty.mutate(id, { onSuccess: handleCloseDrawer });
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">{title}</h2>
          <p className="text-muted-foreground">{description}</p>
        </div>
        <Button onClick={handleNew}>
          <Plus className="h-4 w-4" />
          {newLabel}
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Filter</CardTitle>
          <CardDescription>Search and filter by party type.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-2">
            <Search className="h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search by name, email, phone, or address..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="max-w-md"
            />
          </div>
          <div className="flex flex-wrap gap-2">
            <Button
              type="button"
              variant={partyType === undefined ? "default" : "outline"}
              size="sm"
              onClick={() => setPartyType(undefined)}
            >
              All
            </Button>
            {PARTY_TYPE_OPTIONS.map((type) => (
              <Button
                key={type}
                type="button"
                variant={partyType === type ? "default" : "outline"}
                size="sm"
                onClick={() => setPartyType(type)}
              >
                {PARTY_TYPE_LABELS[type]}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>TIN</TableHead>
              <TableHead>Email</TableHead>
              <TableHead>Phone</TableHead>
              <TableHead>Address</TableHead>
              <TableHead>Type</TableHead>
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
                  Failed to load parties: {error.message}
                </TableCell>
              </TableRow>
            ) : filteredParties.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={6}
                  className="h-24 text-center text-muted-foreground"
                >
                  No parties found.
                </TableCell>
              </TableRow>
            ) : (
              filteredParties.map((party) => (
                <TableRow
                  key={party.id}
                  className="cursor-pointer"
                  onClick={() => handleEdit(party.id)}
                >
                  <TableCell className="font-medium">{party.name}</TableCell>
                  <TableCell>{party.tin ?? "—"}</TableCell>
                  <TableCell>{party.email ?? "—"}</TableCell>
                  <TableCell>{party.phone ?? "—"}</TableCell>
                  <TableCell>{party.address ?? "—"}</TableCell>
                  <TableCell className="capitalize">
                    {PARTY_TYPE_LABELS[party.party_type]}
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <PartyDrawer
        isOpen={drawerOpen}
        partyId={selectedPartyId}
        defaultPartyType={defaultPartyType}
        title={singularTitle}
        party={selectedParty}
        isLoading={!!selectedPartyId && isLoadingParty}
        isSaving={createParty.isPending || updateParty.isPending}
        error={mutationError}
        onClose={handleCloseDrawer}
        onSave={handleSave}
        onDelete={handleDelete}
      />
    </div>
  );
}
