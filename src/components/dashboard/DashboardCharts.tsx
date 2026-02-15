import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
} from "recharts";
import { formatCurrency } from "../../lib/formatters";
import type { MonthlyRevenue, RevenueByClient } from "../../types";

const COLORS = [
  "#6366f1",
  "#8b5cf6",
  "#a855f7",
  "#d946ef",
  "#ec4899",
  "#f43f5e",
];

interface DashboardChartsProps {
  monthlyRevenue: MonthlyRevenue[];
  revenueByClient: RevenueByClient[];
}

export function DashboardCharts({
  monthlyRevenue,
  revenueByClient,
}: DashboardChartsProps) {
  return (
    <>
      <div className="bg-white rounded-xl border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Monthly Revenue
        </h2>
        {monthlyRevenue.length === 0 ? (
          <p className="text-sm text-gray-500 text-center py-12">
            No revenue data yet. Complete and mark invoices as paid.
          </p>
        ) : (
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={monthlyRevenue}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="month" tick={{ fontSize: 12 }} />
              <YAxis tick={{ fontSize: 12 }} />
              <Tooltip formatter={(value) => formatCurrency(Number(value))} />
              <Bar dataKey="revenue" fill="#6366f1" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        )}
      </div>

      <div className="bg-white rounded-xl border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Revenue by Client
        </h2>
        {revenueByClient.length === 0 ? (
          <p className="text-sm text-gray-500 text-center py-12">
            No client revenue data yet.
          </p>
        ) : (
          <ResponsiveContainer width="100%" height={250}>
            <PieChart>
              <Pie
                data={revenueByClient}
                dataKey="total_revenue"
                nameKey="client_name"
                cx="50%"
                cy="50%"
                outerRadius={80}
                label={({ name, percent }) =>
                  `${name} (${((percent ?? 0) * 100).toFixed(0)}%)`
                }
              >
                {revenueByClient.map((_, index) => (
                  <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip formatter={(value) => formatCurrency(Number(value))} />
            </PieChart>
          </ResponsiveContainer>
        )}
      </div>
    </>
  );
}
