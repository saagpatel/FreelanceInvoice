import { useEffect, useState } from "react";
import { Button } from "../components/shared/Button";
import { Input } from "../components/shared/Input";
import { TextArea } from "../components/shared/Input";
import { Badge } from "../components/shared/Badge";
import { useAppStore } from "../stores/appStore";

export function SettingsPage() {
  const store = useAppStore();
  const [tier, setTier] = useState(store.tier);
  const [businessName, setBusinessName] = useState("");
  const [businessEmail, setBusinessEmail] = useState("");
  const [businessAddress, setBusinessAddress] = useState("");
  const [defaultRate, setDefaultRate] = useState("");
  const [claudeKey, setClaudeKey] = useState("");
  const [stripeKey, setStripeKey] = useState("");
  const [stripeSuccessUrl, setStripeSuccessUrl] = useState("");
  const [stripeCancelUrl, setStripeCancelUrl] = useState("");
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setTier(store.tier);
    setBusinessName(store.businessName);
    setBusinessEmail(store.businessEmail);
    setBusinessAddress(store.businessAddress);
    setDefaultRate(String(store.defaultHourlyRate));
    setClaudeKey(store.claudeApiKey);
    setStripeKey(store.stripeApiKey);
    setStripeSuccessUrl(store.stripeSuccessUrl);
    setStripeCancelUrl(store.stripeCancelUrl);
  }, [
    store.tier,
    store.businessName,
    store.businessEmail,
    store.businessAddress,
    store.defaultHourlyRate,
    store.claudeApiKey,
    store.stripeApiKey,
    store.stripeSuccessUrl,
    store.stripeCancelUrl,
  ]);

  const handleSave = async () => {
    setSaving(true);
    try {
      await store.saveSetting("tier", tier);
      await store.saveSetting("business_name", businessName);
      await store.saveSetting("business_email", businessEmail);
      await store.saveSetting("business_address", businessAddress);
      await store.saveSetting("default_hourly_rate", defaultRate);
      await store.saveSetting("claude_api_key", claudeKey);
      await store.saveSetting("stripe_api_key", stripeKey);
      await store.saveSetting("stripe_success_url", stripeSuccessUrl);
      await store.saveSetting("stripe_cancel_url", stripeCancelUrl);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="max-w-2xl">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">Settings</h1>

      {/* Tier */}
      <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">Plan</h2>
            <p className="text-sm text-gray-500">
              Stripe payment links require Premium
            </p>
          </div>
          <Badge variant={tier === "free" ? "default" : "success"}>
            {tier.toUpperCase()}
          </Badge>
        </div>
        <div className="mt-4 flex gap-2">
          {(["free", "pro", "premium"] as const).map((option) => (
            <button
              key={option}
              onClick={() => setTier(option)}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                tier === option
                  ? "bg-primary-600 text-white"
                  : "bg-gray-100 text-gray-700 hover:bg-gray-200"
              }`}
            >
              {option.toUpperCase()}
            </button>
          ))}
        </div>
      </div>

      {/* Business Info */}
      <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Business Information
        </h2>
        <div className="space-y-4">
          <Input
            label="Business Name"
            value={businessName}
            onChange={(e) => setBusinessName(e.target.value)}
            placeholder="Your Business LLC"
          />
          <Input
            label="Email"
            type="email"
            value={businessEmail}
            onChange={(e) => setBusinessEmail(e.target.value)}
            placeholder="hello@yourbusiness.com"
          />
          <TextArea
            label="Address"
            value={businessAddress}
            onChange={(e) => setBusinessAddress(e.target.value)}
            placeholder="123 Main St, City, State 12345"
          />
          <Input
            label="Default Hourly Rate ($)"
            type="number"
            value={defaultRate}
            onChange={(e) => setDefaultRate(e.target.value)}
            placeholder="100"
          />
        </div>
      </div>

      {/* Appearance */}
      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Appearance
        </h2>
        <div className="flex gap-3">
          {(["light", "dark", "system"] as const).map((option) => (
            <button
              key={option}
              onClick={() => store.saveSetting("theme", option)}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                store.theme === option
                  ? "bg-primary-600 text-white"
                  : "bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
              }`}
            >
              {option.charAt(0).toUpperCase() + option.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* API Keys */}
      <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">API Keys</h2>
        <div className="space-y-4">
          <Input
            label="Claude API Key"
            type="password"
            value={claudeKey}
            onChange={(e) => setClaudeKey(e.target.value)}
            placeholder="sk-ant-..."
          />
          <p className="text-xs text-gray-500">
            Required for AI project estimation. Get your key from
            console.anthropic.com
          </p>
          <Input
            label="Stripe API Key"
            type="password"
            value={stripeKey}
            onChange={(e) => setStripeKey(e.target.value)}
            placeholder="sk_live_..."
          />
          <Input
            label="Stripe success URL"
            type="url"
            value={stripeSuccessUrl}
            onChange={(e) => setStripeSuccessUrl(e.target.value)}
            placeholder="https://yourapp.com/payments/success"
          />
          <Input
            label="Stripe cancel URL"
            type="url"
            value={stripeCancelUrl}
            onChange={(e) => setStripeCancelUrl(e.target.value)}
            placeholder="https://yourapp.com/payments/cancel"
          />
          <p className="text-xs text-gray-500">
            Used for Stripe payment-link generation on sent/overdue invoices.
            Premium tier and valid return URLs required.
          </p>
        </div>
      </div>

      <div className="flex items-center gap-3">
        <Button onClick={handleSave} loading={saving}>
          Save Settings
        </Button>
        {saved && (
          <span className="text-sm text-success-600">Settings saved!</span>
        )}
      </div>
    </div>
  );
}
