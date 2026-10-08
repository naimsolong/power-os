import { useEffect, useState } from "react";
import { Link, Navigate } from "react-router-dom";
import { useAuth } from "@/hooks/use-auth";
import { Button } from "@/components/ui/button";
import {
  Brain,
  Briefcase,
  Code,
  FileText,
  Landmark,
  Shield,
  Users,
  Zap,
} from "lucide-react";

const features = [
  {
    icon: Users,
    title: "CRM",
    description:
      "Track contacts, companies, deals, and pipelines in one place.",
  },
  {
    icon: FileText,
    title: "Finance + LHDN e-Invoice",
    description:
      "Create invoices, post double-entry journals, and submit to LHDN MyInvois sandbox.",
  },
  {
    icon: Briefcase,
    title: "People",
    description:
      "Employee directory linked to the same party record as customers and vendors.",
  },
  {
    icon: Brain,
    title: "AI Assistant",
    description:
      "Ask plain-language questions across your business. Read-only tools, fully logged.",
  },
];

const values = [
  {
    icon: Shield,
    title: "Compliance-first",
    description: "LHDN e-Invoice, SST, EPF, SOCSO, and PCB are core features.",
  },
  {
    icon: Landmark,
    title: "Malaysian-focused",
    description:
      "Built for Malaysian SMEs: local tax rules, Malay/English UI, and MYR by default.",
  },
  {
    icon: Zap,
    title: "Self-hostable",
    description:
      "One Docker Compose file on a single VPS. Your data stays on your server.",
  },
];

export function LandingPage() {
  const { user, isLoading } = useAuth();
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    window.addEventListener("scroll", onScroll);
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <span className="text-muted-foreground">Loading…</span>
      </div>
    );
  }

  if (user) {
    return <Navigate to="/dashboard" replace />;
  }

  return (
    <div className="min-h-screen bg-background">
      <header
        className={`sticky top-0 z-50 border-b transition-colors ${
          scrolled ? "bg-background/95 backdrop-blur" : "bg-background"
        }`}
      >
        <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-4 sm:px-6">
          <Link to="/" className="flex items-center gap-2">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
              <Zap className="h-5 w-5" />
            </div>
            <span className="text-lg font-semibold tracking-tight">
              Power OS
            </span>
          </Link>
          <nav className="flex items-center gap-4">
            <a
              href="https://github.com/naimsolong/power-os"
              target="_blank"
              rel="noreferrer"
              className="hidden items-center gap-2 text-sm text-muted-foreground hover:text-foreground sm:flex"
            >
              <Code className="h-4 w-4" />
              GitHub
            </a>
            <Button asChild variant="ghost" size="sm">
              <Link to="/login">Sign in</Link>
            </Button>
            <Button asChild size="sm">
              <Link to="/login">Get started</Link>
            </Button>
          </nav>
        </div>
      </header>

      <main>
        <section className="relative overflow-hidden border-b px-4 py-24 sm:px-6 lg:py-32">
          <div className="mx-auto max-w-4xl text-center">
            <h1 className="text-4xl font-bold tracking-tight text-foreground sm:text-6xl">
              One login. One record. One AI assistant.
            </h1>
            <p className="mx-auto mt-6 max-w-2xl text-lg text-muted-foreground">
              Power OS is the open-source business operating system for
              Malaysian SMEs. CRM, finance, people, and compliance—on your own
              server.
            </p>
            <div className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row">
              <Button asChild size="lg" className="min-w-[10rem]">
                <Link to="/login">Get started free</Link>
              </Button>
              <Button asChild variant="outline" size="lg" className="min-w-[10rem]">
                <a
                  href="https://github.com/naimsolong/power-os"
                  target="_blank"
                  rel="noreferrer"
                >
                  <Code className="mr-2 h-4 w-4" />
                  View on GitHub
                </a>
              </Button>
            </div>
            <p className="mt-4 text-xs text-muted-foreground">
              MIT licensed. Self-host with Docker Compose.
            </p>
          </div>
        </section>

        <section className="px-4 py-20 sm:px-6">
          <div className="mx-auto max-w-6xl">
            <div className="mb-12 text-center">
              <h2 className="text-3xl font-semibold tracking-tight">
                Four pillars, one workspace
              </h2>
              <p className="mt-4 text-muted-foreground">
                Stop switching apps. Power OS connects sales, finance, people,
                and AI around a single record.
              </p>
            </div>
            <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
              {features.map((feature) => (
                <div
                  key={feature.title}
                  className="rounded-xl border bg-card p-6 shadow-sm transition-shadow hover:shadow-md"
                >
                  <div className="mb-4 flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
                    <feature.icon className="h-5 w-5" />
                  </div>
                  <h3 className="text-lg font-semibold">{feature.title}</h3>
                  <p className="mt-2 text-sm text-muted-foreground">
                    {feature.description}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="border-y bg-muted/50 px-4 py-20 sm:px-6">
          <div className="mx-auto max-w-6xl">
            <div className="mb-12 text-center">
              <h2 className="text-3xl font-semibold tracking-tight">
                Built for Malaysian businesses
              </h2>
            </div>
            <div className="grid gap-6 sm:grid-cols-3">
              {values.map((value) => (
                <div key={value.title} className="text-center">
                  <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-background shadow-sm">
                    <value.icon className="h-6 w-6 text-primary" />
                  </div>
                  <h3 className="text-lg font-semibold">{value.title}</h3>
                  <p className="mt-2 text-sm text-muted-foreground">
                    {value.description}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="px-4 py-20 sm:px-6">
          <div className="mx-auto max-w-3xl rounded-2xl bg-primary p-8 text-center text-primary-foreground sm:p-12">
            <h2 className="text-3xl font-semibold tracking-tight">
              Ready to run your business?
            </h2>
            <p className="mx-auto mt-4 max-w-xl text-primary-foreground/90">
              Install Power OS on your own server in minutes. No credit card.
              No vendor lock-in.
            </p>
            <div className="mt-8 flex flex-col items-center justify-center gap-4 sm:flex-row">
              <Button
                asChild
                size="lg"
                variant="secondary"
                className="min-w-[10rem]"
              >
                <Link to="/login">Start using Power OS</Link>
              </Button>
              <Button
                asChild
                size="lg"
                variant="outline"
                className="min-w-[10rem] border-primary-foreground/30 text-primary-foreground hover:bg-primary-foreground/10"
              >
                <a
                  href="https://github.com/naimsolong/power-os#readme"
                  target="_blank"
                  rel="noreferrer"
                >
                  Read the docs
                </a>
              </Button>
            </div>
          </div>
        </section>
      </main>

      <footer className="border-t px-4 py-10 sm:px-6">
        <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 sm:flex-row">
          <div className="flex items-center gap-2">
            <Zap className="h-5 w-5 text-primary" />
            <span className="font-semibold tracking-tight">Power OS</span>
          </div>
          <p className="text-sm text-muted-foreground">
            © {new Date().getFullYear()} Power OS Contributors. MIT License.
          </p>
          <div className="flex items-center gap-6">
            <a
              href="https://github.com/naimsolong/power-os"
              target="_blank"
              rel="noreferrer"
              className="text-sm text-muted-foreground hover:text-foreground"
            >
              GitHub
            </a>
            <Link
              to="/login"
              className="text-sm text-muted-foreground hover:text-foreground"
            >
              Sign in
            </Link>
          </div>
        </div>
      </footer>
    </div>
  );
}
