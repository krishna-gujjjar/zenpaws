import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import type { ReactNode } from "react";

interface AppProvidersProps {
  children: ReactNode;
}

/**
 * Wraps the app in the providers every feature will need. Kept as its own
 * file per docs/15_AI_AGENT_RULES.md's structure convention - App.tsx stays
 * focused on layout, this file owns provider setup.
 */
export function AppProviders({ children }: AppProvidersProps): ReactNode {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            // Local Tauri-IPC "queries" aren't network calls with latency
            // to hide - no need for aggressive background refetching.
            refetchOnWindowFocus: false,
            retry: 1,
          },
        },
      })
  );

  return (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}
