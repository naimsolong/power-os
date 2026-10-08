import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import {
  type EmployeeStatus,
  type EmployeeUpdate,
  createEmployee,
  deleteEmployee,
  fetchEmployee,
  fetchEmployees,
  updateEmployee,
} from "@/lib/api";

export function useEmployees(status?: EmployeeStatus, search?: string) {
  return useQuery({
    queryKey: ["employees", status, search],
    queryFn: () => fetchEmployees(status, search),
  });
}

export function useEmployee(id: string | null) {
  return useQuery({
    queryKey: ["employee", id],
    queryFn: () => fetchEmployee(id!),
    enabled: id !== null,
  });
}

export function useCreateEmployee() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: createEmployee,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employees"] });
    },
  });
}

export function useUpdateEmployee() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, employee }: { id: string; employee: EmployeeUpdate }) =>
      updateEmployee(id, employee),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["employees"] });
      queryClient.invalidateQueries({ queryKey: ["employee", variables.id] });
    },
  });
}

export function useDeleteEmployee() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: deleteEmployee,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["employees"] });
    },
  });
}
