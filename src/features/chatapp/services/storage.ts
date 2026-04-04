function getStorage() {
  if (typeof window === "undefined") {
    return null;
  }
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

export function safeStorageGet(key: string) {
  try {
    return getStorage()?.getItem(key) ?? null;
  } catch {
    return null;
  }
}

export function safeStorageSet(key: string, value: string) {
  try {
    getStorage()?.setItem(key, value);
  } catch {
    // Ignore storage failures.
  }
}

export function safeStorageRemove(key: string) {
  try {
    getStorage()?.removeItem(key);
  } catch {
    // Ignore storage failures.
  }
}

export function safeStorageKeysByPrefix(prefix: string) {
  try {
    const storage = getStorage();
    if (!storage) {
      return [] as string[];
    }
    const keys: string[] = [];
    for (let index = 0; index < storage.length; index += 1) {
      const key = storage.key(index);
      if (typeof key === "string" && key.startsWith(prefix)) {
        keys.push(key);
      }
    }
    return keys;
  } catch {
    return [] as string[];
  }
}
