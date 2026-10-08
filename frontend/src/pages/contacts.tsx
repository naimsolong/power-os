import { PartyList } from "@/components/parties/party-list";

export function ContactsPage() {
  return (
    <PartyList
      title="Contacts"
      singularTitle="Contact"
      description="Manage your contacts."
      newLabel="New Contact"
      defaultPartyType="customer"
    />
  );
}
