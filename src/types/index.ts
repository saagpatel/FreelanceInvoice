export interface Client {
  id: string;
  name: string;
  email: string | null;
  company: string | null;
  address: string | null;
  phone: string | null;
  notes: string | null;
  hourly_rate: number | null;
  created_at: string;
  updated_at: string;
}

export interface CreateClient {
  name: string;
  email?: string | null;
  company?: string | null;
  address?: string | null;
  phone?: string | null;
  notes?: string | null;
  hourly_rate?: number | null;
}

export interface UpdateClient {
  name?: string | null;
  email?: string | null;
  company?: string | null;
  address?: string | null;
  phone?: string | null;
  notes?: string | null;
  hourly_rate?: number | null;
}

export type ProjectStatus = "active" | "completed" | "archived" | "on_hold";

export interface Project {
  id: string;
  client_id: string;
  name: string;
  description: string | null;
  status: ProjectStatus;
  hourly_rate: number | null;
  budget_hours: number | null;
  created_at: string;
  updated_at: string;
}

export interface CreateProject {
  client_id: string;
  name: string;
  description?: string | null;
  status?: ProjectStatus;
  hourly_rate?: number | null;
  budget_hours?: number | null;
}

export interface UpdateProject {
  name?: string | null;
  description?: string | null;
  status?: ProjectStatus;
  hourly_rate?: number | null;
  budget_hours?: number | null;
}

export interface TimeEntry {
  id: string;
  project_id: string;
  description: string | null;
  start_time: string;
  end_time: string;
  duration_secs: number;
  is_billable: boolean;
  is_manual: boolean;
  invoice_id: string | null;
  created_at: string;
}

export type InvoiceStatus = "draft" | "sent" | "paid" | "overdue" | "cancelled";

export interface Invoice {
  id: string;
  invoice_number: string;
  client_id: string;
  status: InvoiceStatus;
  issue_date: string;
  due_date: string;
  subtotal: number;
  tax_rate: number | null;
  tax_amount: number;
  total: number;
  notes: string | null;
  payment_link: string | null;
  created_at: string;
  updated_at: string;
}

export interface InvoiceLineItem {
  id: string;
  invoice_id: string;
  description: string;
  quantity: number;
  unit_price: number;
  amount: number;
  sort_order: number;
}

export interface DraftInvoiceLineItemInput {
  description: string;
  quantity: number;
  unit_price: number;
  sort_order: number;
  source_time_entry_ids?: string[];
}

export interface CreateInvoiceDraftAtomicInput {
  client_id: string;
  issue_date: string;
  due_date: string;
  notes?: string | null;
  tax_rate?: number | null;
  line_items: DraftInvoiceLineItemInput[];
}

export interface Estimate {
  id: string;
  project_description: string;
  conservative_hours: number;
  realistic_hours: number;
  optimistic_hours: number;
  confidence_score: number;
  risk_flags: string[];
  similar_projects: unknown[];
  reasoning: string | null;
  raw_response: string | null;
  created_at: string;
}

export interface TimerState {
  is_running: boolean;
  is_paused: boolean;
  project_id: string | null;
  project_name: string | null;
  description: string | null;
  elapsed_secs: number;
  start_time: string | null;
}

export interface CreateManualTimeEntryInput {
  project_id: string;
  description?: string | null;
  start_time: string;
  end_time: string;
  is_billable?: boolean;
}

export interface UpdateManualTimeEntryInput {
  description?: string | null;
  start_time?: string;
  end_time?: string;
  is_billable?: boolean;
}

export interface ActiveTimer {
  id: number;
  project_id: string;
  description: string | null;
  start_time: string;
  accumulated_secs: number;
  is_paused: boolean;
}

export interface AppSetting {
  key: string;
  value: string;
}

export type Tier = "free" | "pro" | "premium";

export interface DashboardSummary {
  total_revenue: number;
  outstanding_amount: number;
  hours_this_week: number;
  hours_this_month: number;
  active_projects: number;
  pending_invoices: number;
}

export interface RevenueByClient {
  client_name: string;
  total_revenue: number;
  total_hours: number;
  effective_rate: number;
}

export interface HoursByProject {
  project_name: string;
  total_hours: number;
  billable_hours: number;
}

export interface MonthlyRevenue {
  month: string;
  revenue: number;
}
