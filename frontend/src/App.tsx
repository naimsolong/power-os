import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createBrowserRouter,
  Navigate,
  RouterProvider,
} from "react-router-dom";
import { Layout } from "@/components/layout";
import { DashboardPage } from "@/pages/dashboard";
import { ContactsPage } from "@/pages/contacts";
import { CompaniesPage } from "@/pages/companies";
import { DealsPage } from "@/pages/deals";
import { InvoicesPage } from "@/pages/invoices";
import { EmployeesPage } from "@/pages/employees";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,
    },
  },
});

const router = createBrowserRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      { index: true, element: <DashboardPage /> },
      { path: "contacts", element: <ContactsPage /> },
      { path: "companies", element: <CompaniesPage /> },
      { path: "deals", element: <DealsPage /> },
      { path: "invoices", element: <InvoicesPage /> },
      { path: "employees", element: <EmployeesPage /> },
      { path: "*", element: <Navigate to="/" replace /> },
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
