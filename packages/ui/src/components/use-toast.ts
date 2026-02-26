"use client";

import * as React from "react";

export type ToastTone = "neutral" | "success" | "warning" | "error";

export type ToastInput = {
  id?: string;
  title?: React.ReactNode;
  description?: React.ReactNode;
  tone?: ToastTone;
  duration?: number;
};

export type ToastRecord = {
  id: string;
  title?: React.ReactNode;
  description?: React.ReactNode;
  tone: ToastTone;
  duration: number;
};

type ToastContextValue = {
  toasts: ToastRecord[];
  toast: (input: ToastInput) => string;
  dismissToast: (id: string) => void;
  clearToasts: () => void;
};

const DEFAULT_TOAST_DURATION = 4000;

const ToastContext = React.createContext<ToastContextValue | null>(null);

let toastCounter = 0;

function createToastId() {
  toastCounter += 1;
  return `toast-${toastCounter}`;
}

export type ToastProviderProps = {
  children: React.ReactNode;
  maxToasts?: number;
};

export function ToastProvider({ children, maxToasts = 5 }: ToastProviderProps) {
  const [toasts, setToasts] = React.useState<ToastRecord[]>([]);
  const timeoutMapRef = React.useRef<Map<string, number>>(new Map());

  const dismissToast = React.useCallback((id: string) => {
    setToasts((current) => current.filter((toast) => toast.id !== id));
  }, []);

  const clearToasts = React.useCallback(() => {
    setToasts([]);
  }, []);

  const toast = React.useCallback(
    (input: ToastInput) => {
      const id = input.id ?? createToastId();
      const nextToast: ToastRecord = {
        id,
        title: input.title,
        description: input.description,
        tone: input.tone ?? "neutral",
        duration: input.duration ?? DEFAULT_TOAST_DURATION,
      };

      setToasts((current) => {
        const deduped = current.filter((toastItem) => toastItem.id !== id);
        return [...deduped, nextToast].slice(-Math.max(maxToasts, 1));
      });

      return id;
    },
    [maxToasts],
  );

  React.useEffect(() => {
    const visibleIds = new Set(toasts.map((toastItem) => toastItem.id));

    timeoutMapRef.current.forEach((timeoutId, toastId) => {
      if (visibleIds.has(toastId)) {
        return;
      }

      window.clearTimeout(timeoutId);
      timeoutMapRef.current.delete(toastId);
    });

    for (const toastItem of toasts) {
      if (toastItem.duration <= 0 || timeoutMapRef.current.has(toastItem.id)) {
        continue;
      }

      const timeoutId = window.setTimeout(() => {
        dismissToast(toastItem.id);
      }, toastItem.duration);

      timeoutMapRef.current.set(toastItem.id, timeoutId);
    }
  }, [dismissToast, toasts]);

  React.useEffect(
    () => () => {
      timeoutMapRef.current.forEach((timeoutId) => {
        window.clearTimeout(timeoutId);
      });
      timeoutMapRef.current.clear();
    },
    [],
  );

  const contextValue = React.useMemo(
    () => ({
      toasts,
      toast,
      dismissToast,
      clearToasts,
    }),
    [clearToasts, dismissToast, toast, toasts],
  );

  return React.createElement(ToastContext.Provider, { value: contextValue }, children);
}

export function useToast() {
  const context = React.useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within ToastProvider");
  }

  return context;
}
