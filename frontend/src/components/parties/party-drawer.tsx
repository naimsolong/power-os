import { useState } from "react";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  type Party,
  type PartyCreate,
  type PartyType,
  PARTY_TYPE_LABELS,
  PARTY_TYPE_OPTIONS,
} from "@/lib/api";
import { cn } from "@/lib/utils";

interface PartyDrawerProps {
  isOpen: boolean;
  partyId: string | null;
  defaultPartyType: PartyType;
  title: string;
  onClose: () => void;
  onSave: (party: PartyCreate) => void;
  onDelete?: (id: string) => void;
  isLoading?: boolean;
  isSaving?: boolean;
  error?: Error | null;
  party?: Party | null;
}

const emptyForm: PartyCreate = {
  name: "",
  email: "",
  phone: "",
  address: "",
  party_type: "customer",
};

function buildInitialForm(
  party: Party | null | undefined,
  defaultPartyType: PartyType
): PartyCreate {
  if (party) {
    return {
      name: party.name,
      email: party.email ?? "",
      phone: party.phone ?? "",
      address: party.address ?? "",
      party_type: party.party_type,
    };
  }
  return { ...emptyForm, party_type: defaultPartyType };
}

export function PartyDrawer({
  isOpen,
  partyId,
  defaultPartyType,
  title,
  onClose,
  onSave,
  onDelete,
  isLoading,
  isSaving,
  error,
  party,
}: PartyDrawerProps) {
  const [form, setForm] = useState<PartyCreate>(() =>
    buildInitialForm(party, defaultPartyType)
  );
  const [lastPartyKey, setLastPartyKey] = useState<string>(() =>
    party ? party.id : partyId ?? "new"
  );

  // Reset form whenever the underlying party identity changes, including
  // when async data finishes loading after the drawer opens.
  const currentPartyKey = party ? party.id : partyId ?? "new";
  if (currentPartyKey !== lastPartyKey) {
    setLastPartyKey(currentPartyKey);
    setForm(buildInitialForm(party, defaultPartyType));
  }

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    onSave(form);
  };

  return (
    <>
      {/* Backdrop */}
      <div
        className={cn(
          "fixed inset-0 z-40 bg-black/50 transition-opacity duration-200",
          isOpen ? "opacity-100" : "pointer-events-none opacity-0"
        )}
        onClick={onClose}
        aria-hidden={!isOpen}
      />
      {/* Drawer */}
      <div
        className={cn(
          "fixed inset-y-0 right-0 z-50 w-full max-w-md transform bg-background shadow-xl transition-transform duration-200 ease-in-out",
          isOpen ? "translate-x-0" : "translate-x-full"
        )}
        role="dialog"
        aria-modal="true"
        aria-labelledby="party-drawer-title"
      >
        <form onSubmit={handleSubmit} className="flex h-full flex-col">
          <Card className="flex h-full flex-col rounded-none border-0 shadow-none">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 border-b px-6 py-4">
              <CardTitle id="party-drawer-title" className="text-lg">
                {partyId ? "Edit" : "New"} {title}
              </CardTitle>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                onClick={onClose}
                aria-label="Close"
              >
                <X className="h-4 w-4" />
              </Button>
            </CardHeader>

            <CardContent className="flex-1 space-y-4 overflow-y-auto px-6 py-6">
              {error && (
                <div className="rounded-md border border-destructive bg-destructive/10 p-3 text-sm text-destructive">
                  {error.message}
                </div>
              )}
              {isLoading ? (
                <div className="text-sm text-muted-foreground">Loading...</div>
              ) : (
                <>
                  <div className="space-y-2">
                    <label
                      htmlFor="party-name"
                      className="text-sm font-medium"
                    >
                      Name
                    </label>
                    <Input
                      id="party-name"
                      value={form.name}
                      onChange={(e) =>
                        setForm((prev) => ({ ...prev, name: e.target.value }))
                      }
                      placeholder="Name"
                      required
                    />
                  </div>

                  <div className="space-y-2">
                    <label
                      htmlFor="party-email"
                      className="text-sm font-medium"
                    >
                      Email
                    </label>
                    <Input
                      id="party-email"
                      type="email"
                      value={form.email}
                      onChange={(e) =>
                        setForm((prev) => ({ ...prev, email: e.target.value }))
                      }
                      placeholder="Email"
                    />
                  </div>

                  <div className="space-y-2">
                    <label
                      htmlFor="party-phone"
                      className="text-sm font-medium"
                    >
                      Phone
                    </label>
                    <Input
                      id="party-phone"
                      value={form.phone}
                      onChange={(e) =>
                        setForm((prev) => ({ ...prev, phone: e.target.value }))
                      }
                      placeholder="Phone"
                    />
                  </div>

                  <div className="space-y-2">
                    <label
                      htmlFor="party-address"
                      className="text-sm font-medium"
                    >
                      Address
                    </label>
                    <Input
                      id="party-address"
                      value={form.address}
                      onChange={(e) =>
                        setForm((prev) => ({
                          ...prev,
                          address: e.target.value,
                        }))
                      }
                      placeholder="Address"
                    />
                  </div>

                  <div className="space-y-2">
                    <label
                      htmlFor="party-type"
                      className="text-sm font-medium"
                    >
                      Type
                    </label>
                    <select
                      id="party-type"
                      value={form.party_type}
                      onChange={(e) =>
                        setForm((prev) => ({
                          ...prev,
                          party_type: e.target.value as PartyType,
                        }))
                      }
                      className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                    >
                      {PARTY_TYPE_OPTIONS.map((type) => (
                        <option key={type} value={type}>
                          {PARTY_TYPE_LABELS[type]}
                        </option>
                      ))}
                    </select>
                  </div>
                </>
              )}
            </CardContent>

            <CardFooter className="flex justify-between border-t px-6 py-4">
              {partyId && onDelete ? (
                <Button
                  type="button"
                  variant="destructive"
                  disabled={isSaving || isLoading}
                  onClick={() => onDelete(partyId)}
                >
                  Delete
                </Button>
              ) : (
                <div />
              )}
              <div className="flex gap-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={onClose}
                  disabled={isSaving}
                >
                  Cancel
                </Button>
                <Button type="submit" disabled={isSaving || isLoading}>
                  {isSaving ? "Saving..." : "Save"}
                </Button>
              </div>
            </CardFooter>
          </Card>
        </form>
      </div>
    </>
  );
}
