import { useEffect, useState } from "react";
import { Save, Trash2, Users } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

import {
  useLhdnSettings,
  useUpdateLhdnSettings,
} from "@/hooks/use-lhdn-settings";
import {
  useRemoveWorkspaceUser,
  useUpdateWorkspaceUserRole,
  useWorkspaceUsers,
} from "@/hooks/use-workspace-users";
import { useAuth } from "@/hooks/use-auth";
import {
  WORKSPACE_ROLE_LABELS,
  WORKSPACE_ROLE_OPTIONS,
  type WorkspaceRole,
} from "@/lib/api";

export function SettingsPage() {
  const { user } = useAuth();
  const { data: settings, isLoading } = useLhdnSettings();
  const updateSettings = useUpdateLhdnSettings();
  const { data: users, isLoading: isLoadingUsers } = useWorkspaceUsers();
  const updateRole = useUpdateWorkspaceUserRole();
  const removeUser = useRemoveWorkspaceUser();

  const canManageUsers =
    user?.role === "owner" || user?.role === "admin";

  const [form, setForm] = useState({
    lhdn_client_id: "",
    lhdn_client_secret: "",
    lhdn_tin: "",
    lhdn_sandbox: true,
  });

  useEffect(() => {
    if (settings) {
      setForm((prev) => ({
        ...prev,
        lhdn_client_id: settings.lhdn_client_id ?? "",
        lhdn_tin: settings.lhdn_tin ?? "",
        lhdn_sandbox: settings.lhdn_sandbox,
      }));
    }
  }, [settings]);

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    updateSettings.mutate(form);
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">Settings</h2>
        <p className="text-muted-foreground">
          Configure workspace integrations.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">LHDN MyInvois</CardTitle>
          <CardDescription>
            Enter your LHDN sandbox or production OAuth credentials and TIN.
            Credentials are stored encrypted at rest in a production deployment.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            {updateSettings.error && (
              <div className="rounded-md border border-destructive bg-destructive/10 p-3 text-sm text-destructive">
                {updateSettings.error.message}
              </div>
            )}
            {updateSettings.isSuccess && (
              <div className="rounded-md border border-green-600 bg-green-600/10 p-3 text-sm text-green-700">
                LHDN settings saved.
              </div>
            )}

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <div className="space-y-2">
                <label
                  htmlFor="lhdn-client-id"
                  className="text-sm font-medium"
                >
                  Client ID
                </label>
                <Input
                  id="lhdn-client-id"
                  value={form.lhdn_client_id}
                  onChange={(e) =>
                    setForm((prev) => ({
                      ...prev,
                      lhdn_client_id: e.target.value,
                    }))
                  }
                  placeholder="LHDN client ID"
                  disabled={isLoading}
                />
              </div>

              <div className="space-y-2">
                <label
                  htmlFor="lhdn-client-secret"
                  className="text-sm font-medium"
                >
                  Client Secret
                </label>
                <Input
                  id="lhdn-client-secret"
                  type="password"
                  value={form.lhdn_client_secret}
                  onChange={(e) =>
                    setForm((prev) => ({
                      ...prev,
                      lhdn_client_secret: e.target.value,
                    }))
                  }
                  placeholder="LHDN client secret"
                  disabled={isLoading}
                />
              </div>

              <div className="space-y-2">
                <label htmlFor="lhdn-tin" className="text-sm font-medium">
                  TIN
                </label>
                <Input
                  id="lhdn-tin"
                  value={form.lhdn_tin}
                  onChange={(e) =>
                    setForm((prev) => ({ ...prev, lhdn_tin: e.target.value }))
                  }
                  placeholder="e.g. C1234567890"
                  disabled={isLoading}
                />
              </div>

              <div className="flex items-center gap-3 pt-6">
                <input
                  id="lhdn-sandbox"
                  type="checkbox"
                  checked={form.lhdn_sandbox}
                  onChange={(e) =>
                    setForm((prev) => ({
                      ...prev,
                      lhdn_sandbox: e.target.checked,
                    }))
                  }
                  className="h-4 w-4 rounded border-input"
                  disabled={isLoading}
                />
                <label htmlFor="lhdn-sandbox" className="text-sm font-medium">
                  Use sandbox (pre-production)
                </label>
              </div>
            </div>

            <div className="flex justify-end">
              <Button
                type="submit"
                disabled={updateSettings.isPending || isLoading}
              >
                <Save className="h-4 w-4" />
                {updateSettings.isPending ? "Saving..." : "Save Settings"}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <Users className="h-4 w-4" />
            Workspace Users
          </CardTitle>
          <CardDescription>
            Manage roles for people with access to this workspace.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoadingUsers ? (
            <p className="text-sm text-muted-foreground">Loading users…</p>
          ) : !users || users.length === 0 ? (
            <p className="text-sm text-muted-foreground">No users found.</p>
          ) : (
            <div className="divide-y">
              {users.map((workspaceUser) => (
                <div
                  key={workspaceUser.user_id}
                  className="flex items-center justify-between py-3"
                >
                  <div>
                    <p className="text-sm font-medium">
                      {workspaceUser.name ?? workspaceUser.email}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {workspaceUser.email}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    {canManageUsers && workspaceUser.user_id !== user?.user_id ? (
                      <>
                        <select
                          value={workspaceUser.role}
                          onChange={(event) =>
                            updateRole.mutate({
                              userId: workspaceUser.user_id,
                              role: event.target.value as WorkspaceRole,
                            })
                          }
                          disabled={updateRole.isPending}
                          className="h-9 rounded-md border border-input bg-background px-2 text-sm"
                        >
                          {WORKSPACE_ROLE_OPTIONS.map((role) => (
                            <option key={role} value={role}>
                              {WORKSPACE_ROLE_LABELS[role]}
                            </option>
                          ))}
                        </select>
                        <Button
                          variant="ghost"
                          size="icon"
                          title="Remove user"
                          onClick={() => removeUser.mutate(workspaceUser.user_id)}
                          disabled={removeUser.isPending}
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      </>
                    ) : (
                      <span className="text-sm text-muted-foreground">
                        {WORKSPACE_ROLE_LABELS[workspaceUser.role]}
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
