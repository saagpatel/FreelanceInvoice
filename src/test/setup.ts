import "@testing-library/jest-dom/vitest";
import { beforeAll, beforeEach, vi } from "vitest";

const storageMap = new Map<string, string>();

const localStorageMock: Storage = {
  getItem: (key: string) => storageMap.get(key) ?? null,
  setItem: (key: string, value: string) => {
    storageMap.set(key, value);
  },
  removeItem: (key: string) => {
    storageMap.delete(key);
  },
  clear: () => {
    storageMap.clear();
  },
  key: (index: number) => Array.from(storageMap.keys())[index] ?? null,
  get length() {
    return storageMap.size;
  },
};

beforeAll(() => {
  Object.defineProperty(globalThis, "localStorage", {
    value: localStorageMock,
    writable: true,
  });
});

beforeEach(() => {
  localStorageMock.clear();
  vi.clearAllMocks();
});
