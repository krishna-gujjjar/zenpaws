import { motion } from "motion/react";
import type { ReactNode } from "react";
import { ChatShell } from "../features/chat/chat-shell";
import { useAppStatus } from "../hooks/use-app-status";

/**
 * The visible Phase 1 shell: confirms the Rust backend is reachable over
 * IPC. Replaced by real chat/pet UI starting Phase 2+ - this file's only
 * job right now is to prove the wiring works, using the same stack
 * (Motion, TanStack Query) the real UI will use, so Phase 2 isn't the first
 * time any of it runs.
 */
export function AppShell(): ReactNode {
  const { data, isLoading, isError } = useAppStatus();

  if (!(isLoading || isError)) {
    return <ChatShell />;
  }

  return (
    <motion.main
      animate={{ opacity: 1 }}
      initial={{ opacity: 0 }}
      style={styles.main}
      transition={{ duration: 0.3 }}
    >
      <h1 style={styles.heading}>ZenPaws</h1>
      <p style={styles.status}>{renderStatus(isLoading, isError, data?.version)}</p>
    </motion.main>
  );
}

function renderStatus(isLoading: boolean, isError: boolean, version: string | undefined): string {
  if (isLoading) {
    return "Connecting to backend...";
  }
  if (isError) {
    return "Backend unreachable - check the Rust process.";
  }
  return `Backend connected - v${version ?? "unknown"}`;
}

const styles = {
  heading: { fontSize: "2rem", margin: 0 },
  main: {
    alignItems: "center",
    display: "flex",
    flexDirection: "column" as const,
    fontFamily: "system-ui, sans-serif",
    gap: "0.5rem",
    height: "100vh",
    justifyContent: "center",
  },
  status: { color: "#666", margin: 0 },
};
