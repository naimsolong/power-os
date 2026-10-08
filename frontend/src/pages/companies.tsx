import { PartyList } from "@/components/parties/party-list";

export function CompaniesPage() {
  return (
    <PartyList
      title="Companies"
      singularTitle="Company"
      description="Manage your accounts."
      newLabel="New Company"
      defaultPartyType="vendor"
    />
  );
}
