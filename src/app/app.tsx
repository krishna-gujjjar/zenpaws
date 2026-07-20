import type { ReactNode } from "react";
import { AppProviders } from "./app.provider";
import { AppShell } from "./app-shell";

/**
 * Root component. Deliberately minimal at Phase 1 - a real layout, routing
 * (if any), and feature areas (chat/, pets/, transfers/, settings/) are
 * added starting Phase 2 onward, per docs/15_AI_AGENT_RULES.md's structure
 * convention. This is not a placeholder feature; it's the honest current
 * state of the app shell.
 */
export function App(): ReactNode {
  return (
    <AppProviders>
      <AppShell />
    </AppProviders>
  );
}
