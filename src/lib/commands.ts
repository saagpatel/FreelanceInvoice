import { invoke } from "@tauri-apps/api/core";
import type {
  Client,
  CreateClient,
  UpdateClient,
  Project,
  CreateProject,
  UpdateProject,
  TimeEntry,
  TimerState,
  ActiveTimer,
  Invoice,
  InvoiceLineItem,
  CreateInvoiceDraftAtomicInput,
  Estimate,
  AppSetting,
  DashboardSummary,
  RevenueByClient,
  HoursByProject,
  MonthlyRevenue,
  CreateManualTimeEntryInput,
  UpdateManualTimeEntryInput,
} from "../types";

// Clients
export const createClient = (input: CreateClient) =>
  invoke<Client>("create_client", { input });
export const getClient = (id: string) => invoke<Client>("get_client", { id });
export const listClients = () => invoke<Client[]>("list_clients");
export const updateClient = (id: string, input: UpdateClient) =>
  invoke<Client>("update_client", { id, input });
export const deleteClient = (id: string) =>
  invoke<void>("delete_client", { id });

// Projects
export const createProject = (input: CreateProject) =>
  invoke<Project>("create_project", { input });
export const listProjects = (status?: string) =>
  invoke<Project[]>("list_projects", { status: status ?? null });
export const listProjectsByClient = (clientId: string) =>
  invoke<Project[]>("list_projects_by_client", { clientId });
export const updateProject = (id: string, input: UpdateProject) =>
  invoke<Project>("update_project", { id, input });
export const deleteProject = (id: string) =>
  invoke<void>("delete_project", { id });

// Timer
export const startTimer = (projectId: string, description?: string) =>
  invoke<ActiveTimer>("start_timer", {
    projectId,
    description: description ?? null,
  });
export const stopTimer = () => invoke<TimeEntry>("stop_timer");
export const pauseTimer = () => invoke<ActiveTimer>("pause_timer");
export const resumeTimer = () => invoke<ActiveTimer>("resume_timer");
export const getTimerState = () => invoke<TimerState>("get_timer_state");
export const listTimeEntries = (projectId: string) =>
  invoke<TimeEntry[]>("list_time_entries", { projectId });
export const deleteTimeEntry = (id: string) =>
  invoke<void>("delete_time_entry", { id });
export const createManualTimeEntry = (input: CreateManualTimeEntryInput) =>
  invoke<TimeEntry>("create_manual_time_entry", { input });
export const updateManualTimeEntry = (
  id: string,
  input: UpdateManualTimeEntryInput,
) => invoke<TimeEntry>("update_manual_time_entry", { id, input });

// Invoices
export const createInvoice = (
  clientId: string,
  issueDate: string,
  dueDate: string,
  notes?: string,
  taxRate?: number,
) =>
  invoke<Invoice>("create_invoice", {
    clientId,
    issueDate,
    dueDate,
    notes: notes ?? null,
    taxRate: taxRate ?? null,
  });
export const listInvoices = (status?: string) =>
  invoke<Invoice[]>("list_invoices", { status: status ?? null });
export const updateInvoiceStatus = (id: string, status: string) =>
  invoke<Invoice>("update_invoice_status", { id, status });
export const addLineItem = (
  invoiceId: string,
  description: string,
  quantity: number,
  unitPrice: number,
  sortOrder: number,
) =>
  invoke<InvoiceLineItem>("add_line_item", {
    invoiceId,
    description,
    quantity,
    unitPrice,
    sortOrder,
  });
export const getUninvoicedEntries = (clientId: string) =>
  invoke<TimeEntry[]>("get_uninvoiced_entries", { clientId });
export const createInvoiceDraftAtomic = (
  input: CreateInvoiceDraftAtomicInput,
) => invoke<Invoice>("create_invoice_draft_atomic", { input });
export const setInvoicePaymentLink = (
  invoiceId: string,
  paymentLink?: string | null,
) =>
  invoke<Invoice>("set_invoice_payment_link", {
    invoiceId,
    paymentLink: paymentLink ?? null,
  });
export const createStripePaymentLink = (
  invoiceId: string,
  stripeApiKey: string,
  successUrl?: string,
  cancelUrl?: string,
) =>
  invoke<Invoice>("create_stripe_payment_link", {
    invoiceId,
    stripeApiKey,
    successUrl: successUrl ?? null,
    cancelUrl: cancelUrl ?? null,
  });

// Estimates
export const listEstimates = () => invoke<Estimate[]>("list_estimates");

// PDF
export const renderInvoiceHtml = (
  invoiceId: string,
  businessName: string,
  businessEmail: string,
  businessAddress: string,
) =>
  invoke<string>("render_invoice_html", {
    invoiceId,
    businessName,
    businessEmail,
    businessAddress,
  });
export const exportInvoicePdf = (
  invoiceId: string,
  businessName: string,
  businessEmail: string,
  businessAddress: string,
) =>
  invoke<number[]>("export_invoice_pdf", {
    invoiceId,
    businessName,
    businessEmail,
    businessAddress,
  });
// AI Estimation
export const runAiEstimate = (apiKey: string, projectDescription: string) =>
  invoke<Estimate>("run_ai_estimate", { apiKey, projectDescription });

// Dashboard
export const getDashboardSummary = () =>
  invoke<DashboardSummary>("get_dashboard_summary", {});
export const getRevenueByClient = () =>
  invoke<RevenueByClient[]>("get_revenue_by_client", {});
export const getHoursByProject = (days?: number) =>
  invoke<HoursByProject[]>("get_hours_by_project", { days: days ?? null });
export const getMonthlyRevenue = (months?: number) =>
  invoke<MonthlyRevenue[]>("get_monthly_revenue", { months: months ?? null });

// Settings
export const setSetting = (key: string, value: string) =>
  invoke<void>("set_setting", { key, value });
export const getAllSettings = () => invoke<AppSetting[]>("get_all_settings");
