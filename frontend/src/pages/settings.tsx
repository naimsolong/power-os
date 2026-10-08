import { useEffect, useState } from "react";
import { Save } from "lucide-react";
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

export function SettingsPage() {
  const { data: settings, isLoading } = useLhdnSettings();
  const updateSettings = useUpdateLhdnSettings();

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
    </div>
  );
}
