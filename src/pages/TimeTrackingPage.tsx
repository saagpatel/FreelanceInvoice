import { useCallback, useEffect, useState } from "react";
import { Button } from "../components/shared/Button";
import { Select } from "../components/shared/Select";
import { Input } from "../components/shared/Input";
import { EmptyState } from "../components/shared/EmptyState";
import { Modal } from "../components/shared/Modal";
import { useTimerStore } from "../stores/timerStore";
import { formatDuration, formatDateTime, formatHours } from "../lib/formatters";
import * as commands from "../lib/commands";
import type { Project, TimeEntry } from "../types";

const toLocalDateTimeInput = (iso: string) => {
  const date = new Date(iso);
  const offset = date.getTimezoneOffset() * 60_000;
  return new Date(date.getTime() - offset).toISOString().slice(0, 16);
};

const toIsoFromLocalInput = (value: string) => new Date(value).toISOString();

export function TimeTrackingPage() {
  const { timer, startTimer, stopTimer, pauseTimer, resumeTimer, loading } =
    useTimerStore();
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProjectId, setSelectedProjectId] = useState("");
  const [description, setDescription] = useState("");
  const [entries, setEntries] = useState<TimeEntry[]>([]);
  const [loadingEntries, setLoadingEntries] = useState(false);
  const [entryError, setEntryError] = useState<string | null>(null);
  const [manualModalOpen, setManualModalOpen] = useState(false);
  const [editingEntryId, setEditingEntryId] = useState<string | null>(null);
  const [manualDescription, setManualDescription] = useState("");
  const [manualStart, setManualStart] = useState("");
  const [manualEnd, setManualEnd] = useState("");
  const [manualBillable, setManualBillable] = useState(true);
  const [manualSaving, setManualSaving] = useState(false);

  const loadProjects = useCallback(async () => {
    const p = await commands.listProjects("active");
    setProjects(p);
  }, []);

  const loadEntries = useCallback(async () => {
    if (!selectedProjectId && !timer.project_id) return;
    setLoadingEntries(true);
    setEntryError(null);
    try {
      const projectId = selectedProjectId || timer.project_id;
      if (projectId) {
        const e = await commands.listTimeEntries(projectId);
        setEntries(e);
      }
    } catch (err) {
      setEntryError(
        err instanceof Error ? err.message : "Failed to load time entries",
      );
    } finally {
      setLoadingEntries(false);
    }
  }, [selectedProjectId, timer.project_id]);

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  useEffect(() => {
    loadEntries();
  }, [loadEntries]);

  const handleStart = async () => {
    if (!selectedProjectId) return;
    await startTimer(selectedProjectId, description || undefined);
    setDescription("");
  };

  const handleStop = async () => {
    await stopTimer();
    loadEntries();
  };

  const handleDeleteEntry = async (id: string) => {
    await commands.deleteTimeEntry(id);
    loadEntries();
  };

  const openManualModal = (entry?: TimeEntry) => {
    const now = new Date();
    const oneHourAgo = new Date(now.getTime() - 60 * 60 * 1000);

    setEditingEntryId(entry?.id ?? null);
    setManualDescription(entry?.description ?? "");
    setManualStart(
      entry
        ? toLocalDateTimeInput(entry.start_time)
        : toLocalDateTimeInput(oneHourAgo.toISOString()),
    );
    setManualEnd(
      entry
        ? toLocalDateTimeInput(entry.end_time)
        : toLocalDateTimeInput(now.toISOString()),
    );
    setManualBillable(entry?.is_billable ?? true);
    setEntryError(null);
    setManualModalOpen(true);
  };

  const closeManualModal = () => {
    setManualModalOpen(false);
    setEditingEntryId(null);
  };

  const handleSaveManualEntry = async () => {
    const projectId = selectedProjectId || timer.project_id;
    if (!projectId) {
      setEntryError("Select a project before adding a manual entry.");
      return;
    }
    if (!manualStart || !manualEnd) {
      setEntryError("Start and end time are required.");
      return;
    }
    if (new Date(manualEnd) <= new Date(manualStart)) {
      setEntryError("End time must be after start time.");
      return;
    }

    try {
      setManualSaving(true);
      setEntryError(null);
      if (editingEntryId) {
        await commands.updateManualTimeEntry(editingEntryId, {
          description: manualDescription.trim() || null,
          start_time: toIsoFromLocalInput(manualStart),
          end_time: toIsoFromLocalInput(manualEnd),
          is_billable: manualBillable,
        });
      } else {
        await commands.createManualTimeEntry({
          project_id: projectId,
          description: manualDescription.trim() || null,
          start_time: toIsoFromLocalInput(manualStart),
          end_time: toIsoFromLocalInput(manualEnd),
          is_billable: manualBillable,
        });
      }
      closeManualModal();
      await loadEntries();
    } catch (err) {
      setEntryError(
        err instanceof Error ? err.message : "Failed to save manual entry",
      );
    } finally {
      setManualSaving(false);
    }
  };

  const projectOptions = projects.map((p) => ({
    value: p.id,
    label: p.name,
  }));

  return (
    <div>
      <h1 className="text-2xl font-bold text-gray-900 mb-6">Time Tracking</h1>

      {/* Timer Controls */}
      <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
        {timer.is_running ? (
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-500">Currently tracking</p>
              <p className="text-lg font-semibold text-gray-900">
                {timer.project_name}
              </p>
              {timer.description && (
                <p className="text-sm text-gray-600">{timer.description}</p>
              )}
            </div>
            <div className="flex items-center gap-4">
              <span className="text-3xl font-mono font-bold text-gray-900 tabular-nums">
                {formatDuration(timer.elapsed_secs)}
              </span>
              <div className="flex gap-2">
                {timer.is_paused ? (
                  <Button
                    variant="secondary"
                    onClick={() => resumeTimer()}
                    disabled={loading}
                  >
                    Resume
                  </Button>
                ) : (
                  <Button
                    variant="secondary"
                    onClick={() => pauseTimer()}
                    disabled={loading}
                  >
                    Pause
                  </Button>
                )}
                <Button
                  variant="danger"
                  onClick={handleStop}
                  disabled={loading}
                >
                  Stop
                </Button>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex items-end gap-4">
            <div className="flex-1">
              <Select
                label="Project"
                options={projectOptions}
                placeholder="Select a project..."
                value={selectedProjectId}
                onChange={(e) => setSelectedProjectId(e.target.value)}
              />
            </div>
            <div className="flex-1">
              <Input
                label="Description (optional)"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="What are you working on?"
              />
            </div>
            <Button
              onClick={handleStart}
              disabled={!selectedProjectId || loading}
              loading={loading}
            >
              Start Timer
            </Button>
          </div>
        )}
      </div>

      {/* Time Entries */}
      <div className="bg-white rounded-xl border border-gray-200">
        <div className="p-4 border-b border-gray-200">
          <div className="flex items-center justify-between gap-3">
            <h2 className="text-lg font-semibold text-gray-900">
              Recent Entries
            </h2>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => openManualModal()}
              disabled={!selectedProjectId && !timer.project_id}
            >
              Add Manual Entry
            </Button>
          </div>
        </div>
        {entryError && (
          <div className="px-4 py-3 border-b border-danger-200 bg-danger-50 text-sm text-danger-700">
            {entryError}
          </div>
        )}
        {loadingEntries ? (
          <div className="p-8 text-center text-gray-500">Loading...</div>
        ) : entries.length === 0 ? (
          <EmptyState
            title="No time entries yet"
            description="Start a timer or add a manual entry to begin tracking time."
          />
        ) : (
          <div className="divide-y divide-gray-100">
            {entries.map((entry) => (
              <div
                key={entry.id}
                className="flex items-center justify-between px-4 py-3"
              >
                <div>
                  <p className="text-sm font-medium text-gray-900">
                    {entry.description ?? "No description"}
                  </p>
                  <p className="text-xs text-gray-500">
                    {formatDateTime(entry.start_time)}
                    {entry.is_manual && (
                      <span className="ml-2 text-primary-500">Manual</span>
                    )}
                  </p>
                </div>
                <div className="flex items-center gap-4">
                  <span className="text-sm font-mono text-gray-700">
                    {formatHours(entry.duration_secs)}
                  </span>
                  {entry.is_manual && (
                    <button
                      onClick={() => openManualModal(entry)}
                      className="text-xs text-primary-600 hover:text-primary-700 font-medium"
                    >
                      Edit
                    </button>
                  )}
                  <button
                    onClick={() => handleDeleteEntry(entry.id)}
                    className="text-gray-400 hover:text-danger-500 transition-colors"
                  >
                    <svg
                      className="w-4 h-4"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                      />
                    </svg>
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <Modal
        isOpen={manualModalOpen}
        onClose={closeManualModal}
        title={editingEntryId ? "Edit Manual Entry" : "Add Manual Entry"}
      >
        <div className="space-y-4">
          <Input
            label="Description (optional)"
            value={manualDescription}
            onChange={(e) => setManualDescription(e.target.value)}
            placeholder="What was worked on?"
          />
          <Input
            label="Start time"
            type="datetime-local"
            value={manualStart}
            onChange={(e) => setManualStart(e.target.value)}
          />
          <Input
            label="End time"
            type="datetime-local"
            value={manualEnd}
            onChange={(e) => setManualEnd(e.target.value)}
          />
          <label className="flex items-center gap-2 text-sm text-gray-700">
            <input
              type="checkbox"
              checked={manualBillable}
              onChange={(e) => setManualBillable(e.target.checked)}
            />
            Billable entry
          </label>
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="secondary" onClick={closeManualModal}>
              Cancel
            </Button>
            <Button onClick={handleSaveManualEntry} loading={manualSaving}>
              {editingEntryId ? "Save Changes" : "Add Entry"}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
