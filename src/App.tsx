import { lazy, Suspense, useEffect } from "react";
import type { ComponentType } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AppShell } from "./components/layout/AppShell";
import { useTimerStore } from "./stores/timerStore";
import { useAppStore } from "./stores/appStore";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useDarkMode } from "./hooks/useDarkMode";

const DashboardPage = lazy(() =>
  import("./pages/DashboardPage").then((module) => ({
    default: module.DashboardPage,
  }))
);
const ClientsPage = lazy(() =>
  import("./pages/ClientsPage").then((module) => ({
    default: module.ClientsPage,
  }))
);
const ProjectsPage = lazy(() =>
  import("./pages/ProjectsPage").then((module) => ({
    default: module.ProjectsPage,
  }))
);
const TimeTrackingPage = lazy(() =>
  import("./pages/TimeTrackingPage").then((module) => ({
    default: module.TimeTrackingPage,
  }))
);
const InvoicesPage = lazy(() =>
  import("./pages/InvoicesPage").then((module) => ({
    default: module.InvoicesPage,
  }))
);
const InvoiceBuilderPage = lazy(() =>
  import("./pages/InvoiceBuilderPage").then((module) => ({
    default: module.InvoiceBuilderPage,
  }))
);
const EstimatesPage = lazy(() =>
  import("./pages/EstimatesPage").then((module) => ({
    default: module.EstimatesPage,
  }))
);
const SettingsPage = lazy(() =>
  import("./pages/SettingsPage").then((module) => ({
    default: module.SettingsPage,
  }))
);

function PageFallback() {
  return (
    <div className="flex items-center justify-center py-20">
      <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary-600 border-t-transparent" />
    </div>
  );
}

function LazyRoute({ component: Component }: { component: ComponentType }) {
  return (
    <Suspense fallback={<PageFallback />}>
      <Component />
    </Suspense>
  );
}

function AppContent() {
  useKeyboardShortcuts();
  useDarkMode();

  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route path="/" element={<LazyRoute component={DashboardPage} />} />
        <Route path="/clients" element={<LazyRoute component={ClientsPage} />} />
        <Route path="/projects" element={<LazyRoute component={ProjectsPage} />} />
        <Route path="/time" element={<LazyRoute component={TimeTrackingPage} />} />
        <Route
          path="/invoices/new"
          element={<LazyRoute component={InvoiceBuilderPage} />}
        />
        <Route path="/invoices" element={<LazyRoute component={InvoicesPage} />} />
        <Route path="/estimates" element={<LazyRoute component={EstimatesPage} />} />
        <Route path="/settings" element={<LazyRoute component={SettingsPage} />} />
      </Route>
    </Routes>
  );
}

function App() {
  const fetchTimerState = useTimerStore((s) => s.fetchTimerState);
  const loadSettings = useAppStore((s) => s.loadSettings);

  useEffect(() => {
    fetchTimerState();
    loadSettings();
  }, [fetchTimerState, loadSettings]);

  return (
    <BrowserRouter>
      <AppContent />
    </BrowserRouter>
  );
}

export default App;
