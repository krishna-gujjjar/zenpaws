import { invoke } from "@tauri-apps/api/core";
import { motion, useReducedMotion } from "motion/react";
import { useCallback, useEffect, useState } from "react";
import type { ReactNode } from "react";

import { ChatShell } from "../features/chat/chat-shell";
import { useAppStatus } from "../hooks/use-app-status";
import { SetupScreen } from "./setup-screen";

type NetworkState = "ready" | "setup" | "starting";

export function AppShell(): ReactNode {
  const { data, isLoading, isError } = useAppStatus();
  const reduceMotion = useReducedMotion();
  const storedUsername = window.localStorage.getItem("zenpaws.username");
  const storedPeerId = window.localStorage.getItem("zenpaws.peerId");
  const [networkState, setNetworkState] = useState<NetworkState>(() =>
    storedUsername && storedPeerId ? "starting" : "setup"
  );
  const markReady = useCallback(() => setNetworkState("ready"), []);

  useEffect(() => {
    if (networkState !== "starting" || !storedUsername) {
      return;
    }
    let active = true;
    invoke("start_network", { username: storedUsername })
      .then(() => {
        if (active) {
          setNetworkState("ready");
        }
      })
      .catch(() => {
        if (active) {
          setNetworkState("setup");
        }
      });
    return () => {
      active = false;
    };
  }, [networkState, storedUsername]);

  if (isLoading || isError || networkState === "starting") {
    return (
      <StatusScreen
        isError={isError}
        isLoading={isLoading || networkState === "starting"}
        reduceMotion={reduceMotion}
        version={data?.version}
      />
    );
  }
  if (networkState === "setup") {
    return <SetupScreen onReady={markReady} />;
  }
  return <ChatShell />;
}

interface StatusScreenProps {
  isError: boolean;
  isLoading: boolean;
  reduceMotion: boolean | null;
  version: string | undefined;
}

function StatusScreen({
  isError,
  isLoading,
  reduceMotion,
  version,
}: StatusScreenProps) {
  return (
    <motion.main
      animate={reduceMotion ? false : { opacity: 1 }}
      className="status-page"
      initial={reduceMotion ? false : { opacity: 0 }}
      transition={reduceMotion ? { duration: 0 } : { duration: 0.3 }}
    >
      <div aria-hidden="true" className="brand-mark">
        🐾
      </div>
      <h1>ZenPaws</h1>
      <p>{renderStatus(isLoading, isError, version)}</p>
    </motion.main>
  );
}

function renderStatus(
  isLoading: boolean,
  isError: boolean,
  version: string | undefined
): string {
  if (isLoading) {
    return "Connecting to the local service...";
  }
  if (isError) {
    return "The backend is unavailable. Check the Rust process and try again.";
  }
  return `Backend connected - v${version ?? "unknown"}`;
}
