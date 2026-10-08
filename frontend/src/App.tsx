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
import { LoginPage } from "@/pages/login";
import { DashboardPage } from "@/pages/dashboard";
import { ContactsPage } from "@/pages/contacts";
import { CompaniesPage } from "@/pages/companies";
import { DealsPage } from "@/pages/deals";
import { InvoicesPage } from "@/pages/invoices";
import { AiAssistantPage } from "@/pages/ai-assistant";
import { EmployeesPage } from "@/pages/employees";
import { SettingsPage } from "@/pages/settings";

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
        path: "login",
        element: <LoginPage />,
      },
      {
        element: <ProtectedRoute />,
        children: [
          {
            element: <Layout />,
            children: [
              { index: true, element: <DashboardPage /> },
              { path: "contacts", element: <ContactsPage /> },
              { path: "companies", element: <CompaniesPage /> },
              { path: "deals", element: <DealsPage /> },
              { path: "invoices", element: <InvoicesPage /> },
              { path: "employees", element: <EmployeesPage /> },
              { path: "ai-assistant", element: <AiAssistantPage /> },
              { path: "settings", element: <SettingsPage /> },
              { path: "*", element: <Navigate to="/" replace /> },
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
