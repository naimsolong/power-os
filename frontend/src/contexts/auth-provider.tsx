import { useCallback, useMemo, useState, type ReactNode } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { fetchMe, loginUser, logoutUser, type AuthUser, type LoginRequest } from "@/lib/api";
import { AuthContext } from "@/contexts/auth-context";

export function AuthProvider({ children }: { children: ReactNode }) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [authError, setAuthError] = useState<string | null>(null);

  const {
    data: user,
    isLoading,
    refetch,
  } = useQuery<AuthUser | null>({
    queryKey: ["auth", "me"],
    queryFn: async () => {
      try {
        return await fetchMe();
      } catch (error) {
        if (error instanceof Error) {
          if (error.message.toLowerCase().includes("unauthorized")) {
            return null;
          }
        }
        return null;
      }
    },
    retry: false,
    staleTime: Infinity,
  });

  const loginMutation = useMutation<AuthUser, Error, LoginRequest>({
    mutationFn: loginUser,
    onSuccess: async () => {
      setAuthError(null);
      await refetch();
      navigate("/dashboard", { replace: true });
    },
    onError: (error) => {
      setAuthError(error.message);
    },
  });

  const logoutMutation = useMutation<void, Error>({
    mutationFn: logoutUser,
    onSuccess: () => {
      queryClient.setQueryData(["auth", "me"], null);
      queryClient.removeQueries({ queryKey: ["auth", "me"] });
      setAuthError(null);
      navigate("/login", { replace: true });
    },
    onError: (error) => {
      setAuthError(error.message);
    },
  });

  const login = useCallback(
    async (credentials: LoginRequest) => {
      setAuthError(null);
      await loginMutation.mutateAsync(credentials);
    },
    [loginMutation]
  );

  const logout = useCallback(async () => {
    setAuthError(null);
    await logoutMutation.mutateAsync();
  }, [logoutMutation]);

  const value = useMemo(
    () => ({
      user: user ?? null,
      isLoading,
      isAuthenticated: !!user,
      login,
      logout,
      error: authError,
    }),
    [user, isLoading, login, logout, authError]
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
