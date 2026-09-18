import { getAppInfo, type AppInfo } from "@/types/tauriCommands";
import { isTauriRuntime } from "@/services/runtime";
import { APP_NAME_FALLBACK } from "@/app/config";
import { logger } from "@/utils/logger";

export async function readAppInfo(): Promise<AppInfo> {
  if (!isTauriRuntime()) {
    return { name: APP_NAME_FALLBACK, version: "0.0.0", identifier: "dev" };
  }
  try {
    return await getAppInfo();
  } catch (err) {
    logger.error("failed to read app info", { err: String(err) });
    return { name: APP_NAME_FALLBACK, version: "0.0.0", identifier: "unknown" };
  }
}
