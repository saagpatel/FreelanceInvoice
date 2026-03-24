import { useCallback, useEffect, useState } from "react";
import type { Client, InvoiceLineItem, TimeEntry } from "../types";
import {
  listClients,
  getClient,
  getUninvoicedEntries,
  listProjectsByClient,
  createInvoiceDraftAtomic,
} from "../lib/commands";
import { formatHours } from "../lib/formatters";
import { useInvoiceStore } from "../stores/invoiceStore";
import { useAppStore } from "../stores/appStore";
import { Button } from "../components/shared/Button";
import { Input, TextArea } from "../components/shared/Input";
import { Select } from "../components/shared/Select";
import { LineItemRow } from "../components/invoices/LineItemRow";
import { InvoicePreview } from "../components/invoices/InvoicePreview";

type LineItemData = Omit<InvoiceLineItem, "id" | "invoice_id"> & {
  source_time_entry_ids?: string[];
};

export function InvoiceBuilderPage() {
  const [clients, setClients] = useState<Client[]>([]);
  const [selectedClient, setSelectedClient] = useState<Client | null>(null);
  const [loadingClients, setLoadingClients] = useState(true);
  const [loadingEntries, setLoadingEntries] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [showPreview, setShowPreview] = useState(false);

  const {
    clientId,
    issueDate,
    dueDate,
    notes,
    taxRate,
    lineItems,
    setClientId,
    setIssueDate,
    setDueDate,
    setNotes,
    setTaxRate,
    addLineItem: addStoreLineItem,
    updateLineItem,
    removeLineItem,
    reset,
    subtotal,
    taxAmount,
    total,
  } = useInvoiceStore();

  const { businessName, businessEmail, businessAddress } = useAppStore();

  // Load clients on mount
  const fetchClients = useCallback(async () => {
    setLoadingClients(true);
    try {
      const data = await listClients();
      setClients(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load clients");
    } finally {
      setLoadingClients(false);
    }
  }, []);

  useEffect(() => {
    fetchClients();
  }, [fetchClients]);

  // Load selected client details and uninvoiced entries when clientId changes
  useEffect(() => {
    if (!clientId) {
      setSelectedClient(null);
      return;
    }

    let cancelled = false;

    async function loadClientData(id: string) {
      setLoadingEntries(true);
      setError(null);

      try {
        const [client, entries, projects] = await Promise.all([
          getClient(id),
          getUninvoicedEntries(id),
          listProjectsByClient(id),
        ]);

        if (cancelled) return;

        setSelectedClient(client);

        // Build a project name lookup
        const projectNames = new Map(projects.map((p) => [p.id, p.name]));

        // Convert uninvoiced time entries to line items
        if (entries.length > 0) {
          const timeLineItems: LineItemData[] = entries.map(
            (entry: TimeEntry, idx: number) => {
              const hours = parseFloat((entry.duration_secs / 3600).toFixed(2));
              const projectName =
                projectNames.get(entry.project_id) ?? "Project";
              const description = entry.description
                ? `${projectName} - ${entry.description}`
                : `${projectName} (${formatHours(entry.duration_secs)})`;
              const rate = client.hourly_rate ?? 100;

              return {
                description,
                quantity: hours,
                unit_price: rate,
                amount: parseFloat((hours * rate).toFixed(2)),
                sort_order: idx,
                source_time_entry_ids: [entry.id],
              };
            },
          );

          // Replace existing line items with auto-populated ones
          // Reset first then add each
          useInvoiceStore.setState({ lineItems: timeLineItems });
        }
      } catch (err) {
        if (!cancelled) {
          setError(
            err instanceof Error ? err.message : "Failed to load client data",
          );
        }
      } finally {
        if (!cancelled) {
          setLoadingEntries(false);
        }
      }
    }

    loadClientData(clientId);

    return () => {
      cancelled = true;
    };
  }, [clientId]);

  function handleClientChange(e: React.ChangeEvent<HTMLSelectElement>) {
    const value = e.target.value;
    setClientId(value || null);
    if (!value) {
      useInvoiceStore.setState({ lineItems: [] });
    }
  }

  function handleAddManualItem() {
    addStoreLineItem({
      description: "",
      quantity: 1,
      unit_price: 0,
      amount: 0,
      sort_order: lineItems.length,
      source_time_entry_ids: [],
    });
  }

  function handleUpdateItem(index: number, item: LineItemData) {
    updateLineItem(index, {
      ...item,
      source_time_entry_ids: lineItems[index]?.source_time_entry_ids ?? [],
    });
  }

  async function handleSaveDraft() {
    if (!clientId) {
      setError("Please select a client before saving.");
      return;
    }
    if (lineItems.length === 0) {
      setError("Add at least one line item before saving.");
      return;
    }

    setSaving(true);
    setError(null);
    setSuccess(null);

    try {
      const invoice = await createInvoiceDraftAtomic({
        client_id: clientId,
        issue_date: issueDate,
        due_date: dueDate,
        notes: notes || null,
        tax_rate: taxRate > 0 ? taxRate : null,
        line_items: lineItems.map((item, index) => ({
          description: item.description,
          quantity: item.quantity,
          unit_price: item.unit_price,
          sort_order: index,
          source_time_entry_ids: item.source_time_entry_ids ?? [],
        })),
      });

      setSuccess(`Invoice #${invoice.invoice_number} saved as draft.`);
      reset();
      setSelectedClient(null);
      setShowPreview(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save invoice");
    } finally {
      setSaving(false);
    }
  }

  const clientOptions = clients.map((c) => ({
    value: c.id,
    label: c.company ? `${c.name} (${c.company})` : c.name,
  }));

  if (loadingClients) {
    return (
      <div className="flex items-center justify-center py-20">
        <div className="animate-spin h-8 w-8 border-4 border-primary-600 border-t-transparent rounded-full" />
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-gray-900">New Invoice</h1>
        <div className="flex items-center gap-3">
          <Button
            variant="secondary"
            onClick={() => setShowPreview(!showPreview)}
          >
            {showPreview ? "Edit" : "Preview"}
          </Button>
          <Button onClick={handleSaveDraft} loading={saving}>
            Save as Draft
          </Button>
        </div>
      </div>

      {error && (
        <div className="bg-danger-50 border border-danger-200 rounded-lg p-4 mb-4">
          <p className="text-sm text-danger-700">{error}</p>
        </div>
      )}

      {success && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-4 mb-4">
          <p className="text-sm text-green-700">{success}</p>
        </div>
      )}

      {showPreview ? (
        <InvoicePreview
          businessName={businessName}
          businessEmail={businessEmail}
          businessAddress={businessAddress}
          client={selectedClient}
          lineItems={lineItems}
          issueDate={issueDate}
          dueDate={dueDate}
          subtotal={subtotal()}
          taxRate={taxRate}
          taxAmount={taxAmount()}
          total={total()}
          notes={notes}
        />
      ) : (
        <div className="space-y-6">
          {/* Client Selection */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-4">Client</h2>
            <Select
              label="Select Client"
              options={clientOptions}
              value={clientId ?? ""}
              onChange={handleClientChange}
              placeholder="Choose a client..."
            />
            {loadingEntries && (
              <p className="text-sm text-gray-500 mt-2">
                Loading uninvoiced time entries...
              </p>
            )}
          </div>

          {/* Dates */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-4">Dates</h2>
            <div className="grid grid-cols-2 gap-4">
              <Input
                label="Issue Date"
                type="date"
                value={issueDate}
                onChange={(e) => setIssueDate(e.target.value)}
              />
              <Input
                label="Due Date"
                type="date"
                value={dueDate}
                onChange={(e) => setDueDate(e.target.value)}
              />
            </div>
          </div>

          {/* Line Items */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold text-gray-900">
                Line Items
              </h2>
              <Button
                variant="secondary"
                size="sm"
                onClick={handleAddManualItem}
              >
                + Add Item
              </Button>
            </div>

            {lineItems.length === 0 ? (
              <p className="text-sm text-gray-500 py-4 text-center">
                {clientId
                  ? "No uninvoiced time entries found. Add items manually."
                  : "Select a client to auto-populate from uninvoiced time entries, or add items manually."}
              </p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-gray-200">
                      <th className="text-left text-xs font-semibold text-gray-500 uppercase tracking-wide pb-2">
                        Description
                      </th>
                      <th className="text-right text-xs font-semibold text-gray-500 uppercase tracking-wide pb-2 w-24">
                        Qty
                      </th>
                      <th className="text-right text-xs font-semibold text-gray-500 uppercase tracking-wide pb-2 w-32">
                        Rate
                      </th>
                      <th className="text-right text-xs font-semibold text-gray-500 uppercase tracking-wide pb-2 w-32">
                        Amount
                      </th>
                      <th className="w-12" />
                    </tr>
                  </thead>
                  <tbody>
                    {lineItems.map((item, idx) => (
                      <LineItemRow
                        key={idx}
                        item={item}
                        index={idx}
                        onUpdate={handleUpdateItem}
                        onRemove={removeLineItem}
                      />
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>

          {/* Tax & Totals */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-4">Totals</h2>
            <div className="max-w-xs ml-auto space-y-3">
              <Input
                label="Tax Rate (%)"
                type="number"
                value={taxRate}
                onChange={(e) => setTaxRate(parseFloat(e.target.value) || 0)}
                min={0}
                max={100}
                step={0.1}
              />
              <div className="border-t border-gray-200 pt-3 space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600">Subtotal</span>
                  <span className="font-medium text-gray-900">
                    {new Intl.NumberFormat("en-US", {
                      style: "currency",
                      currency: "USD",
                    }).format(subtotal())}
                  </span>
                </div>
                {taxRate > 0 && (
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-600">Tax ({taxRate}%)</span>
                    <span className="font-medium text-gray-900">
                      {new Intl.NumberFormat("en-US", {
                        style: "currency",
                        currency: "USD",
                      }).format(taxAmount())}
                    </span>
                  </div>
                )}
                <div className="flex justify-between text-base font-bold border-t border-gray-200 pt-2">
                  <span className="text-gray-900">Total</span>
                  <span className="text-gray-900">
                    {new Intl.NumberFormat("en-US", {
                      style: "currency",
                      currency: "USD",
                    }).format(total())}
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* Notes */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-4">Notes</h2>
            <TextArea
              label="Invoice Notes"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder="Payment terms, thank you message, etc."
              rows={4}
            />
          </div>
        </div>
      )}
    </div>
  );
}
