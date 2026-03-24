import { create } from "zustand";
import type { Tier } from "../types";
import * as commands from "../lib/commands";

type Theme = "light" | "dark" | "system";

const DEFAULT_TIER: Tier = "free";
const DEFAULT_THEME: Theme = "system";
const DEFAULT_HOURLY_RATE = 100;

const isTier = (value: string | undefined): value is Tier =>
  value === "free" || value === "pro" || value === "premium";

const isTheme = (value: string | undefined): value is Theme =>
  value === "light" || value === "dark" || value === "system";

const parseTier = (value: string | undefined): Tier =>
  isTier(value) ? value : DEFAULT_TIER;

const parseTheme = (value: string | undefined): Theme =>
  isTheme(value) ? value : DEFAULT_THEME;

const parseHourlyRate = (value: string | undefined): number => {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : DEFAULT_HOURLY_RATE;
};

interface AppStore {
  tier: Tier;
  businessName: string;
  businessEmail: string;
  businessAddress: string;
  defaultHourlyRate: number;
  claudeApiKey: string;
  stripeApiKey: string;
  stripeSuccessUrl: string;
  stripeCancelUrl: string;
  theme: Theme;
  loading: boolean;

  loadSettings: () => Promise<void>;
  saveSetting: (key: string, value: string) => Promise<void>;
}

export const useAppStore = create<AppStore>()((set) => ({
  tier: "free",
  businessName: "",
  businessEmail: "",
  businessAddress: "",
  defaultHourlyRate: DEFAULT_HOURLY_RATE,
  claudeApiKey: "",
  stripeApiKey: "",
  stripeSuccessUrl: "",
  stripeCancelUrl: "",
  theme: DEFAULT_THEME,
  loading: true,

  loadSettings: async () => {
    try {
      const settings = await commands.getAllSettings();
      const map = new Map(settings.map((s) => [s.key, s.value]));

      set({
        tier: parseTier(map.get("tier")),
        businessName: map.get("business_name") ?? "",
        businessEmail: map.get("business_email") ?? "",
        businessAddress: map.get("business_address") ?? "",
        defaultHourlyRate: parseHourlyRate(map.get("default_hourly_rate")),
        claudeApiKey: map.get("claude_api_key") ?? "",
        stripeApiKey: map.get("stripe_api_key") ?? "",
        stripeSuccessUrl: map.get("stripe_success_url") ?? "",
        stripeCancelUrl: map.get("stripe_cancel_url") ?? "",
        theme: parseTheme(map.get("theme")),
        loading: false,
      });
    } catch {
      set({ loading: false });
    }
  },

  saveSetting: async (key: string, value: string) => {
    await commands.setSetting(key, value);
    // Update local state based on key
    const keyMap: Record<string, string> = {
      tier: "tier",
      business_name: "businessName",
      business_email: "businessEmail",
      business_address: "businessAddress",
      default_hourly_rate: "defaultHourlyRate",
      claude_api_key: "claudeApiKey",
      stripe_api_key: "stripeApiKey",
      stripe_success_url: "stripeSuccessUrl",
      stripe_cancel_url: "stripeCancelUrl",
      theme: "theme",
    };
    const stateKey = keyMap[key];
    if (stateKey) {
      set({
        [stateKey]:
          stateKey === "defaultHourlyRate" ? parseHourlyRate(value) : value,
      } as Partial<AppStore>);
    }
  },
}));
