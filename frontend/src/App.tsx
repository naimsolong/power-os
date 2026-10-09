import { type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createBrowserRouter,
  Navigate,
  Outlet,
  RouterProvider,
} from "react-router-dom";
import { AuthProvider } from "@/contexts/auth-provider";
import { ProtectedRoute } from "@/components/protected-route";
import { Layout } from "@/components/layout";
import { ForgotPasswordPage } from "@/pages/forgot-password";
import { LoginPage } from "@/pages/login";
import { ResetPasswordPage } from "@/pages/reset-password";
import { DashboardPage } from "@/pages/dashboard";
import { ContactsPage } from "@/pages/contacts";
import { CompaniesPage } from "@/pages/companies";
import { DealsPage } from "@/pages/deals";
import { InvoicesPage } from "@/pages/invoices";
import { AiAssistantPage } from "@/pages/ai-assistant";
import { ChartOfAccountsPage } from "@/pages/chart-of-accounts";
import { EmployeesPage } from "@/pages/employees";
import { FiscalPeriodsPage } from "@/pages/fiscal-periods";
import { LandingPage } from "@/pages/landing";
import { SettingsPage } from "@/pages/settings";
import { TaxCodesPage } from "@/pages/tax-codes";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,
    },
  },
});

function Root({ children }: { children: ReactNode }) {
  return (
    <AuthProvider>
      {children}
    </AuthProvider>
  );
}

const router = createBrowserRouter([
  {
    path: "/",
    element: (
      <Root>
        <Outlet />
      </Root>
    ),
    children: [
      {
        index: true,
        element: <LandingPage />,
      },
      {
        path: "login",
        element: <LoginPage />,
      },
      {
        path: "forgot-password",
        element: <ForgotPasswordPage />,
      },
      {
        path: "reset-password",
        element: <ResetPasswordPage />,
      },
      {
        element: <ProtectedRoute />,
        children: [
          {
            element: <Layout />,
            children: [
              { path: "dashboard", element: <DashboardPage /> },
              { path: "contacts", element: <ContactsPage /> },
              { path: "companies", element: <CompaniesPage /> },
              { path: "deals", element: <DealsPage /> },
              { path: "invoices", element: <InvoicesPage /> },
              { path: "employees", element: <EmployeesPage /> },
              { path: "chart-of-accounts", element: <ChartOfAccountsPage /> },
              { path: "ai-assistant", element: <AiAssistantPage /> },
              { path: "tax-codes", element: <TaxCodesPage /> },
              { path: "fiscal-periods", element: <FiscalPeriodsPage /> },
              { path: "settings", element: <SettingsPage /> },
              { index: true, element: <Navigate to="/dashboard" replace /> },
              { path: "*", element: <Navigate to="/dashboard" replace /> },
            ],
          },
        ],
      },
    ],
  },
]);

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}

export default App;
