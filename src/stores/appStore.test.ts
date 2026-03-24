import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("../lib/commands", () => ({
  getAllSettings: vi.fn(),
  setSetting: vi.fn(),
}));

import { useAppStore } from "./appStore";
import * as commands from "../lib/commands";

describe("appStore", () => {
  beforeEach(() => {
    useAppStore.setState({
      tier: "free",
      businessName: "",
      businessEmail: "",
      businessAddress: "",
      defaultHourlyRate: 100,
      claudeApiKey: "",
      stripeApiKey: "",
      stripeSuccessUrl: "",
      stripeCancelUrl: "",
      theme: "system",
      loading: true,
    });
    vi.restoreAllMocks();
  });

  it("starts with default values", () => {
    const state = useAppStore.getState();
    expect(state.tier).toBe("free");
    expect(state.businessName).toBe("");
    expect(state.defaultHourlyRate).toBe(100);
    expect(state.theme).toBe("system");
    expect(state.loading).toBe(true);
  });

  it("loadSettings populates from backend", async () => {
    vi.mocked(commands.getAllSettings).mockResolvedValue([
      { key: "tier", value: "pro" },
      { key: "business_name", value: "Acme Corp" },
      { key: "business_email", value: "hi@acme.com" },
      { key: "default_hourly_rate", value: "150" },
      { key: "stripe_success_url", value: "https://app.example/success" },
      { key: "stripe_cancel_url", value: "https://app.example/cancel" },
      { key: "theme", value: "dark" },
    ]);

    await useAppStore.getState().loadSettings();

    const state = useAppStore.getState();
    expect(state.tier).toBe("pro");
    expect(state.businessName).toBe("Acme Corp");
    expect(state.businessEmail).toBe("hi@acme.com");
    expect(state.defaultHourlyRate).toBe(150);
    expect(state.stripeSuccessUrl).toBe("https://app.example/success");
    expect(state.stripeCancelUrl).toBe("https://app.example/cancel");
    expect(state.theme).toBe("dark");
    expect(state.loading).toBe(false);
  });

  it("loadSettings falls back for malformed tier/theme/rate values", async () => {
    vi.mocked(commands.getAllSettings).mockResolvedValue([
      { key: "tier", value: "enterprise" },
      { key: "default_hourly_rate", value: "not-a-number" },
      { key: "theme", value: "auto" },
    ]);

    await useAppStore.getState().loadSettings();

    const state = useAppStore.getState();
    expect(state.tier).toBe("free");
    expect(state.defaultHourlyRate).toBe(100);
    expect(state.theme).toBe("system");
    expect(state.loading).toBe(false);
  });

  it("loadSettings handles empty settings", async () => {
    vi.mocked(commands.getAllSettings).mockResolvedValue([]);

    await useAppStore.getState().loadSettings();

    const state = useAppStore.getState();
    expect(state.tier).toBe("free");
    expect(state.businessName).toBe("");
    expect(state.loading).toBe(false);
  });

  it("loadSettings handles errors gracefully", async () => {
    vi.mocked(commands.getAllSettings).mockRejectedValue("DB error");

    await useAppStore.getState().loadSettings();

    expect(useAppStore.getState().loading).toBe(false);
  });

  it("saveSetting updates backend and local state", async () => {
    vi.mocked(commands.setSetting).mockResolvedValue(undefined);

    await useAppStore.getState().saveSetting("business_name", "New Name");

    expect(commands.setSetting).toHaveBeenCalledWith(
      "business_name",
      "New Name",
    );
    expect(useAppStore.getState().businessName).toBe("New Name");
  });

  it("saveSetting converts hourly rate to number", async () => {
    vi.mocked(commands.setSetting).mockResolvedValue(undefined);

    await useAppStore.getState().saveSetting("default_hourly_rate", "200");

    expect(useAppStore.getState().defaultHourlyRate).toBe(200);
  });

  it("saveSetting falls back hourly rate when provided invalid number", async () => {
    vi.mocked(commands.setSetting).mockResolvedValue(undefined);

    await useAppStore
      .getState()
      .saveSetting("default_hourly_rate", "not-a-number");

    expect(useAppStore.getState().defaultHourlyRate).toBe(100);
  });
});
