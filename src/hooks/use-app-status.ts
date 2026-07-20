import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

/** Exact shape of the Rust `AppStatus` struct - kept in sync by hand until
 * Phase 2+ introduces generated bindings. No `any`, no unchecked cast: this
 * is the one place that asserts the IPC payload's shape, and it does so
 * with a declared interface, not a type assertion. */
export interface AppStatus {
  name: string;
  version: string;
}

/**
 * Phase 1 smoke-test hook verifying the frontend↔backend IPC bridge works
 * end-to-end. Matches the temporary `app_status` command in
 * `src-tauri/src/commands/mod.rs` - both are removed/replaced once Phase 2
 * introduces the first real feature command.
 */
export function useAppStatus() {
  return useQuery({
    queryFn: () => invoke<AppStatus>("app_status"),
    queryKey: ["app-status"],
  });
}
