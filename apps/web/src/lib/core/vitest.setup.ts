/**
 * Node exposes a reserved `localStorage` global without an implementation
 * unless it is started with `--localstorage-file`. Vitest's jsdom environment
 * still provides the browser storage object on `window`, so mirror it onto the
 * global object used by the application and its browser-oriented tests.
 */
function createMemoryStorage(): Storage {
  const values = new Map<string, string>();

  return {
    get length() {
      return values.size;
    },
    clear() {
      values.clear();
    },
    getItem(key) {
      return values.get(key) ?? null;
    },
    key(index) {
      return [...values.keys()][index] ?? null;
    },
    removeItem(key) {
      values.delete(key);
    },
    setItem(key, value) {
      values.set(key, String(value));
    },
  };
}

if (typeof window !== 'undefined') {
  let storage: Storage;
  try {
    storage = window.localStorage ?? createMemoryStorage();
  } catch {
    storage = createMemoryStorage();
  }

  Object.defineProperty(globalThis, 'localStorage', {
    configurable: true,
    value: storage,
    writable: true,
  });
}
