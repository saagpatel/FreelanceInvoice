import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "../components/shared/Button";
import { DataTable } from "../components/shared/DataTable";
import { StatusBadge } from "../components/shared/Badge";
import { EmptyState } from "../components/shared/EmptyState";
import { Modal } from "../components/shared/Modal";
import { formatCurrency, formatDate } from "../lib/formatters";
import * as commands from "../lib/commands";
import { useAppStore } from "../stores/appStore";
import type { Invoice, InvoiceStatus } from "../types";

const STATUS_TABS: { label: string; value: InvoiceStatus | "all" }[] = [
  { label: "All", value: "all" },
  { label: "Draft", value: "draft" },
  { label: "Sent", value: "sent" },
  { label: "Paid", value: "paid" },
  { label: "Overdue", value: "overdue" },
  { label: "Cancelled", value: "cancelled" },
];

export function InvoicesPage() {
  const navigate = useNavigate();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [statusFilter, setStatusFilter] = useState<InvoiceStatus | "all">(
    "all",
  );
  const [loading, setLoading] = useState(true);
  const [actionBusy, setActionBusy] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [previewHtml, setPreviewHtml] = useState<string | null>(null);
  const {
    businessName,
    businessEmail,
    businessAddress,
    stripeApiKey,
    stripeSuccessUrl,
    stripeCancelUrl,
  } = useAppStore();

  const loadInvoices = useCallback(async () => {
    setLoading(true);
    try {
      const status = statusFilter === "all" ? undefined : statusFilter;
      const data = await commands.listInvoices(status);
      setInvoices(data);
    } finally {
      setLoading(false);
    }
  }, [statusFilter]);

  useEffect(() => {
    loadInvoices();
  }, [loadInvoices]);

  async function handlePreview(invoice: Invoice) {
    try {
      const html = await commands.renderInvoiceHtml(
        invoice.id,
        businessName,
        businessEmail,
        businessAddress,
      );
      setPreviewHtml(html);
    } catch (err) {
      console.error("Failed to render invoice:", err);
    }
  }

  async function handleStatusChange(invoice: Invoice, status: InvoiceStatus) {
    try {
      setActionBusy(invoice.id);
      setActionError(null);
      await commands.updateInvoiceStatus(invoice.id, status);
      loadInvoices();
    } catch (err) {
      setActionError(
        err instanceof Error ? err.message : "Failed to update invoice status",
      );
    } finally {
      setActionBusy(null);
    }
  }

  async function handleCreatePaymentLink(invoice: Invoice) {
    if (!stripeApiKey.trim()) {
      setActionError(
        "Add a Stripe API key in Settings before creating payment links.",
      );
      return;
    }
    if (invoice.total <= 0) {
      setActionError(
        "Invoice total must be greater than zero before creating a payment link.",
      );
      return;
    }
    if (!stripeSuccessUrl.trim() || !stripeCancelUrl.trim()) {
      setActionError(
        "Add Stripe success and cancel URLs in Settings before creating payment links.",
      );
      return;
    }
    try {
      setActionBusy(invoice.id);
      setActionError(null);
      await commands.createStripePaymentLink(
        invoice.id,
        stripeApiKey,
        stripeSuccessUrl,
        stripeCancelUrl,
      );
      await loadInvoices();
    } catch (err) {
      setActionError(
        err instanceof Error
          ? err.message
          : "Failed to create Stripe payment link",
      );
    } finally {
      setActionBusy(null);
    }
  }

  async function handleClearPaymentLink(invoice: Invoice) {
    try {
      setActionBusy(invoice.id);
      setActionError(null);
      await commands.setInvoicePaymentLink(invoice.id, null);
      await loadInvoices();
    } catch (err) {
      setActionError(
        err instanceof Error ? err.message : "Failed to clear payment link",
      );
    } finally {
      setActionBusy(null);
    }
  }

  async function handleDownloadPdf(invoice: Invoice) {
    try {
      setActionBusy(invoice.id);
      setActionError(null);
      const pdfBytes = await commands.exportInvoicePdf(
        invoice.id,
        businessName,
        businessEmail,
        businessAddress,
      );
      const uint8 = new Uint8Array(pdfBytes);
      const blob = new Blob([uint8], { type: "application/pdf" });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = `${invoice.invoice_number}.pdf`;
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(url);
    } catch (err) {
      setActionError(
        err instanceof Error ? err.message : "Failed to export invoice PDF",
      );
    } finally {
      setActionBusy(null);
    }
  }

  const columns = [
    {
      key: "number",
      header: "Invoice #",
      render: (inv: Invoice) => (
        <span className="font-medium">{inv.invoice_number}</span>
      ),
    },
    {
      key: "status",
      header: "Status",
      render: (inv: Invoice) => <StatusBadge status={inv.status} />,
    },
    {
      key: "date",
      header: "Date",
      render: (inv: Invoice) => formatDate(inv.issue_date),
    },
    {
      key: "due",
      header: "Due",
      render: (inv: Invoice) => formatDate(inv.due_date),
    },
    {
      key: "total",
      header: "Total",
      render: (inv: Invoice) => (
        <span className="font-medium">{formatCurrency(inv.total)}</span>
      ),
    },
    {
      key: "actions",
      header: "",
      render: (inv: Invoice) => (
        <div className="flex gap-2">
          <button
            onClick={() => handlePreview(inv)}
            className="text-xs text-primary-600 hover:text-primary-700 font-medium"
          >
            Preview
          </button>
          {inv.status === "draft" && (
            <button
              onClick={() => handleStatusChange(inv, "sent")}
              disabled={actionBusy === inv.id}
              className="text-xs text-blue-600 hover:text-blue-700 font-medium"
            >
              Mark Sent
            </button>
          )}
          {(inv.status === "draft" ||
            inv.status === "sent" ||
            inv.status === "overdue") && (
            <button
              onClick={() => handleStatusChange(inv, "cancelled")}
              disabled={actionBusy === inv.id}
              className="text-xs text-gray-600 hover:text-gray-700 font-medium"
            >
              Cancel
            </button>
          )}
          {inv.status === "sent" && (
            <button
              onClick={() => handleStatusChange(inv, "overdue")}
              disabled={actionBusy === inv.id}
              className="text-xs text-orange-600 hover:text-orange-700 font-medium"
            >
              Mark Overdue
            </button>
          )}
          {(inv.status === "sent" || inv.status === "overdue") && (
            <button
              onClick={() => handleStatusChange(inv, "paid")}
              disabled={actionBusy === inv.id}
              className="text-xs text-green-600 hover:text-green-700 font-medium"
            >
              Mark Paid
            </button>
          )}
          {inv.status !== "cancelled" && (
            <button
              onClick={() => handleDownloadPdf(inv)}
              disabled={actionBusy === inv.id}
              className="text-xs text-indigo-600 hover:text-indigo-700 font-medium"
            >
              Download PDF
            </button>
          )}
          {(inv.status === "sent" ||
            inv.status === "overdue" ||
            inv.status === "paid") &&
            (!inv.payment_link ? (
              <button
                onClick={() => handleCreatePaymentLink(inv)}
                disabled={actionBusy === inv.id || inv.total <= 0}
                className="text-xs text-cyan-700 hover:text-cyan-800 font-medium"
              >
                Create Pay Link
              </button>
            ) : (
              <>
                <a
                  href={inv.payment_link}
                  target="_blank"
                  rel="noreferrer"
                  className="text-xs text-cyan-700 hover:text-cyan-800 font-medium"
                >
                  Open Pay Link
                </a>
                <button
                  onClick={() => handleClearPaymentLink(inv)}
                  disabled={actionBusy === inv.id}
                  className="text-xs text-gray-500 hover:text-gray-700 font-medium"
                >
                  Clear Link
                </button>
              </>
            ))}
        </div>
      ),
    },
  ];

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Invoices</h1>
        <Button onClick={() => navigate("/invoices/new")}>New Invoice</Button>
      </div>

      {/* Status Tabs */}
      <div className="flex gap-1 mb-4 bg-gray-100 p-1 rounded-lg w-fit">
        {STATUS_TABS.map((tab) => (
          <button
            key={tab.value}
            onClick={() => setStatusFilter(tab.value)}
            className={`px-3 py-1.5 text-sm font-medium rounded-md transition-colors ${
              statusFilter === tab.value
                ? "bg-white text-gray-900 shadow-sm"
                : "text-gray-600 hover:text-gray-900"
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {actionError && (
        <div className="mb-4 rounded-lg border border-danger-200 bg-danger-50 p-3 text-sm text-danger-700">
          {actionError}
        </div>
      )}

      <div className="bg-white rounded-xl border border-gray-200">
        {loading ? (
          <div className="p-8 text-center text-gray-500">Loading...</div>
        ) : invoices.length === 0 ? (
          <EmptyState
            title="No invoices yet"
            description="Create your first invoice to start billing clients."
            actionLabel="New Invoice"
            onAction={() => navigate("/invoices/new")}
          />
        ) : (
          <DataTable
            columns={columns}
            data={invoices}
            keyExtractor={(inv) => inv.id}
          />
        )}
      </div>

      {/* Preview Modal */}
      {previewHtml && (
        <Modal
          title="Invoice Preview"
          isOpen={true}
          onClose={() => setPreviewHtml(null)}
          size="lg"
        >
          <iframe
            title="Invoice Preview"
            sandbox=""
            srcDoc={previewHtml}
            className="h-[70vh] w-full rounded-lg border border-gray-200 bg-white"
          />
        </Modal>
      )}
    </div>
  );
}
