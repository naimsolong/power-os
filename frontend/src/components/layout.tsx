import { NavLink, Outlet } from "react-router-dom";
import {
  LayoutDashboard,
  Users,
  Building2,
  Handshake,
  FileText,
  Briefcase,
  Sparkles,
  Settings,
  LogOut,
  User,
  BookOpen,
  Receipt,
  CalendarRange,
  NotebookPen,
  Banknote,
  FileMinus,
  Wallet,
  BarChart3,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAuth } from "@/hooks/use-auth";
import { Button } from "@/components/ui/button";

const navItems = [
  { to: "/dashboard", label: "Dashboard", icon: LayoutDashboard },
  { to: "/contacts", label: "Contacts", icon: Users },
  { to: "/companies", label: "Companies", icon: Building2 },
  { to: "/deals", label: "Deals", icon: Handshake },
  { to: "/invoices", label: "Invoices", icon: FileText },
  { to: "/bills", label: "Bills", icon: FileMinus },
  { to: "/expenses", label: "Expenses", icon: Banknote },
  { to: "/employees", label: "Employees", icon: Briefcase },
  { to: "/chart-of-accounts", label: "Chart of Accounts", icon: BookOpen },
  { to: "/journal-entries", label: "Journal Entries", icon: NotebookPen },
  { to: "/payments", label: "Payments", icon: Wallet },
  { to: "/reports", label: "Reports", icon: BarChart3 },
  { to: "/ai-assistant", label: "AI Assistant", icon: Sparkles },
  { to: "/tax-codes", label: "Tax Codes", icon: Receipt },
  { to: "/tax-reports", label: "Tax Reports", icon: FileText },
  { to: "/fiscal-periods", label: "Periods", icon: CalendarRange },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function Sidebar() {
  return (
    <aside className="w-64 border-r bg-card min-h-full flex flex-col">
      <div className="h-16 flex items-center px-6 border-b">
        <span className="text-lg font-semibold tracking-tight">Power OS</span>
      </div>
      <nav className="flex-1 p-4 space-y-1">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                isActive
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              )
            }
          >
            <item.icon className="h-4 w-4" />
            {item.label}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}

export function Layout() {
  const { user, logout } = useAuth();

  return (
    <div className="flex min-h-screen bg-background">
      <Sidebar />
      <main className="flex-1 flex flex-col min-w-0">
        <header className="h-16 border-b bg-card flex items-center justify-between px-8">
          <h1 className="text-xl font-semibold">Power OS</h1>
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <User className="h-4 w-4" />
              <span className="hidden sm:inline">
                {user?.name ?? user?.email ?? "User"}
              </span>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => logout()}
              className="gap-2"
            >
              <LogOut className="h-4 w-4" />
              <span className="hidden sm:inline">Logout</span>
            </Button>
          </div>
        </header>
        <div className="flex-1 p-8 overflow-auto">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
