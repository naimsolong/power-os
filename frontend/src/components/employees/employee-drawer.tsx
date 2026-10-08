import { useEffect, useMemo, useState } from "react";
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
import { cn } from "@/lib/utils";
import {
  type Employee,
  type EmployeeCreate,
  type EmployeeStatus,
  type Party,
  EMPLOYEE_STATUS_OPTIONS,
} from "@/lib/api";

interface EmployeeDrawerProps {
  isOpen: boolean;
  employee: Employee | null;
  parties: Party[];
  isLoading?: boolean;
  isSaving?: boolean;
  error?: Error | null;
  onClose: () => void;
  onSave: (employee: EmployeeCreate) => void;
  onDelete?: (id: string) => void;
}

const emptyForm: EmployeeCreate = {
  party_id: "",
  employee_code: "",
  job_title: "",
  department: "",
  hire_date: "",
  status: "active",
};

export function EmployeeDrawer({
  isOpen,
  employee,
  parties,
  isLoading,
  isSaving,
  error,
  onClose,
  onSave,
  onDelete,
}: EmployeeDrawerProps) {
  const [form, setForm] = useState<EmployeeCreate>(emptyForm);

  const initialForm = useMemo<EmployeeCreate>(() => {
    if (!employee) return emptyForm;
    return {
      party_id: employee.party_id,
      employee_code: employee.employee_code,
      job_title: employee.job_title ?? "",
      department: employee.department ?? "",
      hire_date: employee.hire_date ? employee.hire_date.slice(0, 10) : "",
      status: employee.status,
    };
  }, [employee]);

  useEffect(() => {
    setForm(initialForm);
  }, [initialForm]);

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    const payload: EmployeeCreate = {
      ...form,
      job_title: form.job_title || undefined,
      department: form.department || undefined,
      hire_date: form.hire_date || undefined,
    };
    onSave(payload);
  };

  return (
    <>
      <div
        className={cn(
          "fixed inset-0 z-40 bg-black/50 transition-opacity duration-200",
          isOpen ? "opacity-100" : "pointer-events-none opacity-0"
        )}
        onClick={onClose}
        aria-hidden={!isOpen}
      />
      <div
        className={cn(
          "fixed inset-y-0 right-0 z-50 w-full max-w-lg transform bg-background shadow-xl transition-transform duration-200 ease-in-out",
          isOpen ? "translate-x-0" : "translate-x-full"
        )}
        role="dialog"
        aria-modal="true"
        aria-labelledby="employee-drawer-title"
      >
        <form onSubmit={handleSubmit} className="flex h-full flex-col">
          <Card className="flex h-full flex-col rounded-none border-0 shadow-none">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 border-b px-6 py-4">
              <CardTitle id="employee-drawer-title" className="text-lg">
                {employee ? "Edit" : "New"} Employee
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

            <CardContent className="flex-1 space-y-6 overflow-y-auto px-6 py-6">
              {error && (
                <div className="rounded-md border border-destructive bg-destructive/10 p-3 text-sm text-destructive">
                  {error.message}
                </div>
              )}
              {isLoading ? (
                <div className="text-sm text-muted-foreground">Loading...</div>
              ) : (
                <div className="space-y-4">
                  <div className="space-y-2">
                    <label
                      htmlFor="employee-party"
                      className="text-sm font-medium"
                    >
                      Person / Party
                    </label>
                    <select
                      id="employee-party"
                      value={form.party_id}
                      onChange={(e: React.ChangeEvent<HTMLSelectElement>) =>
                        setForm((prev) => ({ ...prev, party_id: e.target.value }))
                      }
                      disabled={parties.length === 0}
                      className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                    >
                      <option value="">Select a party</option>
                      {parties.map((party) => (
                        <option key={party.id} value={party.id}>
                          {party.name}
                          {party.email ? ` · ${party.email}` : ""}
                        </option>
                      ))}
                    </select>
                    {parties.length === 0 && (
                      <p className="text-xs text-muted-foreground">
                        Create a party in Contacts first.
                      </p>
                    )}
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <label
                        htmlFor="employee-code"
                        className="text-sm font-medium"
                      >
                        Employee Code
                      </label>
                      <Input
                        id="employee-code"
                        value={form.employee_code}
                        onChange={(e) =>
                          setForm((prev) => ({
                            ...prev,
                            employee_code: e.target.value,
                          }))
                        }
                        placeholder="EMP-001"
                        required
                      />
                    </div>

                    <div className="space-y-2">
                      <label
                        htmlFor="employee-status"
                        className="text-sm font-medium"
                      >
                        Status
                      </label>
                      <select
                        id="employee-status"
                        value={form.status}
                        onChange={(e: React.ChangeEvent<HTMLSelectElement>) =>
                          setForm((prev) => ({
                            ...prev,
                            status: e.target.value as EmployeeStatus,
                          }))
                        }
                        className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                      >
                        {EMPLOYEE_STATUS_OPTIONS.map((status) => (
                          <option key={status} value={status}>
                            {status.charAt(0).toUpperCase() + status.slice(1)}
                          </option>
                        ))}
                      </select>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <label
                        htmlFor="employee-job-title"
                        className="text-sm font-medium"
                      >
                        Job Title
                      </label>
                      <Input
                        id="employee-job-title"
                        value={form.job_title}
                        onChange={(e) =>
                          setForm((prev) => ({
                            ...prev,
                            job_title: e.target.value,
                          }))
                        }
                        placeholder="Software Engineer"
                      />
                    </div>

                    <div className="space-y-2">
                      <label
                        htmlFor="employee-department"
                        className="text-sm font-medium"
                      >
                        Department
                      </label>
                      <Input
                        id="employee-department"
                        value={form.department}
                        onChange={(e) =>
                          setForm((prev) => ({
                            ...prev,
                            department: e.target.value,
                          }))
                        }
                        placeholder="Engineering"
                      />
                    </div>
                  </div>

                  <div className="space-y-2">
                    <label
                      htmlFor="employee-hire-date"
                      className="text-sm font-medium"
                    >
                      Hire Date
                    </label>
                    <Input
                      id="employee-hire-date"
                      type="date"
                      value={form.hire_date}
                      onChange={(e) =>
                        setForm((prev) => ({
                          ...prev,
                          hire_date: e.target.value,
                        }))
                      }
                    />
                  </div>
                </div>
              )}
            </CardContent>

            <CardFooter className="flex justify-between border-t px-6 py-4">
              {employee && onDelete ? (
                <Button
                  type="button"
                  variant="destructive"
                  disabled={isSaving}
                  onClick={() => onDelete(employee.id)}
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
                <Button type="submit" disabled={isSaving}>
                  {isSaving ? "Saving..." : employee ? "Save" : "Create"}
                </Button>
              </div>
            </CardFooter>
          </Card>
        </form>
      </div>
    </>
  );
}
