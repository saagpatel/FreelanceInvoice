import { create } from "zustand";
import type { InvoiceLineItem } from "../types";

type DraftLineItem = Omit<InvoiceLineItem, "id" | "invoice_id"> & {
  source_time_entry_ids?: string[];
};

interface InvoiceBuilderStore {
  clientId: string | null;
  issueDate: string;
  dueDate: string;
  notes: string;
  taxRate: number;
  lineItems: DraftLineItem[];

  setClientId: (id: string | null) => void;
  setIssueDate: (date: string) => void;
  setDueDate: (date: string) => void;
  setNotes: (notes: string) => void;
  setTaxRate: (rate: number) => void;
  addLineItem: (item: DraftLineItem) => void;
  updateLineItem: (index: number, item: DraftLineItem) => void;
  removeLineItem: (index: number) => void;
  reset: () => void;

  subtotal: () => number;
  taxAmount: () => number;
  total: () => number;
}

const today = () => new Date().toISOString().split("T")[0];
const thirtyDaysLater = () => {
  const d = new Date();
  d.setDate(d.getDate() + 30);
  return d.toISOString().split("T")[0];
};

export const useInvoiceStore = create<InvoiceBuilderStore>()((set, get) => ({
  clientId: null,
  issueDate: today(),
  dueDate: thirtyDaysLater(),
  notes: "",
  taxRate: 0,
  lineItems: [],

  setClientId: (id) => set({ clientId: id }),
  setIssueDate: (date) => set({ issueDate: date }),
  setDueDate: (date) => set({ dueDate: date }),
  setNotes: (notes) => set({ notes }),
  setTaxRate: (rate) => set({ taxRate: rate }),

  addLineItem: (item) =>
    set((state) => ({ lineItems: [...state.lineItems, item] })),

  updateLineItem: (index, item) =>
    set((state) => {
      const items = [...state.lineItems];
      items[index] = item;
      return { lineItems: items };
    }),

  removeLineItem: (index) =>
    set((state) => ({
      lineItems: state.lineItems.filter((_, i) => i !== index),
    })),

  reset: () =>
    set({
      clientId: null,
      issueDate: today(),
      dueDate: thirtyDaysLater(),
      notes: "",
      taxRate: 0,
      lineItems: [],
    }),

  subtotal: () => get().lineItems.reduce((sum, item) => sum + item.amount, 0),

  taxAmount: () => {
    const sub = get().subtotal();
    return sub * (get().taxRate / 100);
  },

  total: () => get().subtotal() + get().taxAmount(),
}));
