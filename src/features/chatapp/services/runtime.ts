export type TauriInvoke = (command: string, args?: Record<string, unknown>) => Promise<unknown>;
export type TauriWindowApi = {
  close?: () => Promise<void> | void;
  minimize?: () => Promise<void> | void;
  maximize?: () => Promise<void> | void;
  unmaximize?: () => Promise<void> | void;
  toggleMaximize?: () => Promise<void> | void;
  isMaximized?: () => Promise<boolean> | boolean;
  toggleFullscreen?: () => Promise<void> | void;
  setFullscreen?: (value: boolean) => Promise<void> | void;
  isFullscreen?: () => Promise<boolean> | boolean;
};
export type TauriNamespace = {
  core?: {
    invoke?: TauriInvoke;
    convertFileSrc?: (filePath: string, protocol?: string) => string;
  };
  window?: {
    getCurrentWindow?: () => TauriWindowApi;
  };
};

export function getTauriNamespace(): TauriNamespace | null {
  if (typeof window === "undefined") {
    return null;
  }
  return (window as Window & { __TAURI__?: TauriNamespace }).__TAURI__ ?? null;
}

export function getTauriInvoke(): TauriInvoke | null {
  if (typeof window === "undefined") {
    return null;
  }
  const runtime = window as Window & {
    __TAURI__?: TauriNamespace;
    __TAURI_INTERNALS__?: {
      invoke?: TauriInvoke;
    };
  };
  return runtime.__TAURI__?.core?.invoke ?? runtime.__TAURI_INTERNALS__?.invoke ?? null;
}

export function getTauriConvertFileSrc(): ((filePath: string, protocol?: string) => string) | null {
  if (typeof window === "undefined") {
    return null;
  }
  const runtime = window as Window & {
    __TAURI__?: TauriNamespace;
    __TAURI_INTERNALS__?: {
      convertFileSrc?: (filePath: string, protocol?: string) => string;
    };
  };
  return runtime.__TAURI__?.core?.convertFileSrc ?? runtime.__TAURI_INTERNALS__?.convertFileSrc ?? null;
}

export function getTauriWindow(): TauriWindowApi | null {
  return getTauriNamespace()?.window?.getCurrentWindow?.() ?? null;
}
