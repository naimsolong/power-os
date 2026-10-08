import {
  useQuery,
  useMutation,
  useQueryClient,
  type UseQueryOptions,
} from "@tanstack/react-query";
import {
  type Party,
  type PartyCreate,
  type PartyUpdate,
  type PartyType,
  fetchParties,
  fetchParty,
  createParty,
  updateParty,
  deleteParty,
} from "@/lib/api";

const PARTIES_KEY = "parties";
const PARTY_KEY = "party";

export function useParties(
  partyType?: PartyType,
  search?: string,
  options?: Omit<UseQueryOptions<Party[]>, "queryKey" | "queryFn">
) {
  return useQuery<Party[]>({
    queryKey: [PARTIES_KEY, { partyType, search }],
    queryFn: () => fetchParties(partyType, search),
    ...options,
  });
}

export function useParty(id: string | null) {
  return useQuery<Party>({
    queryKey: [PARTY_KEY, id],
    queryFn: () => fetchParty(id as string),
    enabled: !!id,
  });
}

export function useCreateParty() {
  const queryClient = useQueryClient();
  return useMutation<Party, Error, PartyCreate>({
    mutationFn: createParty,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [PARTIES_KEY] });
    },
  });
}

export function useUpdateParty() {
  const queryClient = useQueryClient();
  return useMutation<Party, Error, { id: string; data: PartyUpdate }>({
    mutationFn: ({ id, data }) => updateParty(id, data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: [PARTIES_KEY] });
      queryClient.invalidateQueries({ queryKey: [PARTY_KEY, variables.id] });
    },
  });
}

export function useDeleteParty() {
  const queryClient = useQueryClient();
  return useMutation<void, Error, string>({
    mutationFn: deleteParty,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [PARTIES_KEY] });
    },
  });
}
