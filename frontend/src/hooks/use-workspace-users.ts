import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  fetchWorkspaceUsers,
  removeWorkspaceUser,
  updateWorkspaceUserRole,
  type WorkspaceRole,
  type WorkspaceUser,
} from "@/lib/api";

const QUERY_KEY = ["workspace-users"];

export function useWorkspaceUsers() {
  return useQuery<WorkspaceUser[]>({
    queryKey: QUERY_KEY,
    queryFn: fetchWorkspaceUsers,
  });
}

export function useUpdateWorkspaceUserRole() {
  const queryClient = useQueryClient();

  return useMutation<WorkspaceUser, Error, { userId: string; role: WorkspaceRole }>({
    mutationFn: ({ userId, role }) => updateWorkspaceUserRole(userId, role),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEY });
    },
  });
}

export function useRemoveWorkspaceUser() {
  const queryClient = useQueryClient();

  return useMutation<void, Error, string>({
    mutationFn: removeWorkspaceUser,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEY });
    },
  });
}
