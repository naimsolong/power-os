import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  type LhdnSettings,
  type LhdnSettingsResponse,
  fetchLhdnSettings,
  updateLhdnSettings,
} from "@/lib/api";

const LHDN_SETTINGS_KEY = "lhdn-settings";

export function useLhdnSettings() {
  return useQuery<LhdnSettingsResponse>({
    queryKey: [LHDN_SETTINGS_KEY],
    queryFn: fetchLhdnSettings,
  });
}

export function useUpdateLhdnSettings() {
  const queryClient = useQueryClient();
  return useMutation<LhdnSettingsResponse, Error, LhdnSettings>({
    mutationFn: updateLhdnSettings,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [LHDN_SETTINGS_KEY] });
    },
  });
}
