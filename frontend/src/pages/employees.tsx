import { useState } from "react";
import { Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  useCreateEmployee,
  useDeleteEmployee,
  useEmployee,
  useEmployees,
  useUpdateEmployee,
} from "@/hooks/use-employees";
import { useParties } from "@/hooks/use-parties";
import { EmployeeDrawer } from "@/components/employees/employee-drawer";
import { EmployeeList } from "@/components/employees/employee-list";
import { type Employee, type EmployeeCreate } from "@/lib/api";

export function EmployeesPage() {
  const [search, setSearch] = useState("");
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [selectedEmployeeId, setSelectedEmployeeId] = useState<string | null>(
    null
  );

  const { data: employees, isLoading: isLoadingEmployees } = useEmployees(
    undefined,
    search || undefined
  );
  const { data: parties } = useParties();
  const { data: selectedEmployee } = useEmployee(selectedEmployeeId);

  const createEmployee = useCreateEmployee();
  const updateEmployee = useUpdateEmployee();
  const deleteEmployee = useDeleteEmployee();

  const handleAdd = () => {
    setSelectedEmployeeId(null);
    setDrawerOpen(true);
  };

  const handleEdit = (employee: Employee) => {
    setSelectedEmployeeId(employee.id);
    setDrawerOpen(true);
  };

  const handleClose = () => {
    setDrawerOpen(false);
    setSelectedEmployeeId(null);
  };

  const handleSave = (payload: EmployeeCreate) => {
    if (selectedEmployeeId) {
      updateEmployee.mutate(
        { id: selectedEmployeeId, employee: payload },
        { onSuccess: handleClose }
      );
    } else {
      createEmployee.mutate(payload, { onSuccess: handleClose });
    }
  };

  const handleDelete = (id: string) => {
    const employee = employees?.find((e) => e.id === id);
    if (employee && confirm(`Delete employee ${employee.employee_code}?`)) {
      deleteEmployee.mutate(employee.id);
    }
  };

  const isSaving = createEmployee.isPending || updateEmployee.isPending;
  const error =
    createEmployee.error || updateEmployee.error || deleteEmployee.error;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Employees</h2>
          <p className="text-muted-foreground">Manage your team.</p>
        </div>
        <Button onClick={handleAdd}>
          <Plus className="mr-2 h-4 w-4" />
          Add Employee
        </Button>
      </div>

      <div className="flex items-center gap-2">
        <Input
          placeholder="Search employees..."
          className="max-w-sm"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      {isLoadingEmployees ? (
        <div className="text-sm text-muted-foreground">Loading...</div>
      ) : (
        <EmployeeList
          employees={employees ?? []}
          onEdit={handleEdit}
          onDelete={handleDelete}
        />
      )}

      <EmployeeDrawer
        isOpen={drawerOpen}
        employee={selectedEmployee ?? null}
        parties={parties ?? []}
        isLoading={selectedEmployeeId !== null && !selectedEmployee}
        isSaving={isSaving}
        error={error ?? null}
        onClose={handleClose}
        onSave={handleSave}
        onDelete={selectedEmployeeId ? handleDelete : undefined}
      />
    </div>
  );
}
