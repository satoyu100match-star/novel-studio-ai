/**
 * Single source of truth for user-facing app identity in the frontend.
 *
 * Per the project's "must stay renameable" rule, this string is not to be
 * duplicated as a literal anywhere else in `src/`. The Rust side gets its
 * copy from `src-tauri/tauri.conf.json` (`productName`), exposed to the
 * frontend at runtime via the `app_info` command (see
 * `src/services/appInfoService.ts`). This constant is only the
 * synchronous fallback used before that round-trip resolves (e.g. first
 * paint, or in tests that don't run inside Tauri).
 */
export const APP_NAME_FALLBACK = "Novel Studio AI";
