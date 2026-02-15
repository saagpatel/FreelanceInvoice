import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

vi.mock("zustand/middleware", async () => {
  const actual = await vi.importActual<typeof import("zustand/middleware")>(
    "zustand/middleware"
  );
  return {
    ...actual,
    persist: (((initializer: unknown) => initializer) as unknown) as typeof actual.persist,
  };
});

// Mock commands before importing the store
vi.mock("../lib/commands", () => ({
  getTimerState: vi.fn(),
  startTimer: vi.fn(),
  stopTimer: vi.fn(),
  pauseTimer: vi.fn(),
  resumeTimer: vi.fn(),
}));

import { useTimerStore } from "./timerStore";
import * as commands from "../lib/commands";

describe("timerStore", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    // Reset store state
    useTimerStore.setState({
      timer: {
        is_running: false,
        is_paused: false,
        project_id: null,
        project_name: null,
        description: null,
        elapsed_secs: 0,
        start_time: null,
      },
      tickInterval: null,
      loading: false,
      error: null,
    });
  });

  afterEach(() => {
    useTimerStore.getState().stopTicking();
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it("starts with default timer state", () => {
    const state = useTimerStore.getState();
    expect(state.timer.is_running).toBe(false);
    expect(state.timer.elapsed_secs).toBe(0);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it("tick increments elapsed_secs when running", () => {
    useTimerStore.setState({
      timer: {
        is_running: true,
        is_paused: false,
        project_id: "p1",
        project_name: "Test",
        description: null,
        elapsed_secs: 10,
        start_time: new Date().toISOString(),
      },
    });

    useTimerStore.getState().tick();
    expect(useTimerStore.getState().timer.elapsed_secs).toBe(11);
  });

  it("tick does not increment when paused", () => {
    useTimerStore.setState({
      timer: {
        is_running: true,
        is_paused: true,
        project_id: "p1",
        project_name: "Test",
        description: null,
        elapsed_secs: 10,
        start_time: new Date().toISOString(),
      },
    });

    useTimerStore.getState().tick();
    expect(useTimerStore.getState().timer.elapsed_secs).toBe(10);
  });

  it("tick does not increment when not running", () => {
    useTimerStore.getState().tick();
    expect(useTimerStore.getState().timer.elapsed_secs).toBe(0);
  });

  it("fetchTimerState updates state from backend", async () => {
    const mockTimer = {
      is_running: true,
      is_paused: false,
      project_id: "p1",
      project_name: "Project",
      description: "Working",
      elapsed_secs: 120,
      start_time: new Date().toISOString(),
    };
    vi.mocked(commands.getTimerState).mockResolvedValue(mockTimer);

    await useTimerStore.getState().fetchTimerState();

    expect(useTimerStore.getState().timer).toEqual(mockTimer);
    expect(useTimerStore.getState().error).toBeNull();
  });

  it("fetchTimerState sets error on failure", async () => {
    vi.mocked(commands.getTimerState).mockRejectedValue("Network error");

    await useTimerStore.getState().fetchTimerState();

    expect(useTimerStore.getState().error).toBe("Network error");
  });

  it("stopTimer resets state", async () => {
    useTimerStore.setState({
      timer: {
        is_running: true,
        is_paused: false,
        project_id: "p1",
        project_name: "Test",
        description: null,
        elapsed_secs: 300,
        start_time: new Date().toISOString(),
      },
    });
    vi.mocked(commands.stopTimer).mockResolvedValue({
      id: "e1",
      project_id: "p1",
      description: null,
      start_time: "",
      end_time: "",
      duration_secs: 300,
      is_billable: true,
      is_manual: false,
      invoice_id: null,
      created_at: "",
    });

    await useTimerStore.getState().stopTimer();

    expect(useTimerStore.getState().timer.is_running).toBe(false);
    expect(useTimerStore.getState().timer.elapsed_secs).toBe(0);
  });
});
