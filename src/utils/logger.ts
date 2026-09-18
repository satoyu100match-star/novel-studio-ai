/**
 * Thin logging wrapper.
 *
 * Rule (CLAUDE.md "禁止事項" / spec #92): never log API keys, full
 * manuscript text, or other personal data. Callers pass short, structured
 * messages; do not pass raw document bodies or credential objects through
 * this module. In production builds, debug-level logs are dropped.
 */

type LogLevel = "debug" | "info" | "warn" | "error";

const isDev = import.meta.env.DEV;

function emit(level: LogLevel, message: string, context?: Record<string, unknown>) {
  if (level === "debug" && !isDev) return;

  const line = context ? `[${level}] ${message}` : `[${level}] ${message}`;
  switch (level) {
    case "debug":
      // eslint-disable-next-line no-console
      console.debug(line, context ?? "");
      break;
    case "info":
      // eslint-disable-next-line no-console
      console.info(line, context ?? "");
      break;
    case "warn":
      // eslint-disable-next-line no-console
      console.warn(line, context ?? "");
      break;
    case "error":
      // eslint-disable-next-line no-console
      console.error(line, context ?? "");
      break;
  }
}

export const logger = {
  debug: (message: string, context?: Record<string, unknown>) => emit("debug", message, context),
  info: (message: string, context?: Record<string, unknown>) => emit("info", message, context),
  warn: (message: string, context?: Record<string, unknown>) => emit("warn", message, context),
  error: (message: string, context?: Record<string, unknown>) => emit("error", message, context),
};
