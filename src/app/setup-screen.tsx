import { invoke } from "@tauri-apps/api/core";
import { motion, useReducedMotion } from "motion/react";
import { useCallback, useState } from "react";
import type { FormEvent } from "react";

interface NetworkStatus {
  peerId: string;
  tcpPort: number;
}

interface SetupScreenProps {
  onReady: () => void;
}

function formatStartupError(reason: unknown): string {
  if (typeof reason === "string" && reason.length > 0) {
    return reason;
  }
  if (reason instanceof Error) {
    return reason.message;
  }
  return "Could not start the LAN service.";
}

export function SetupScreen({ onReady }: SetupScreenProps) {
  const reduceMotion = useReducedMotion();
  const [username, setUsername] = useState(
    () => window.localStorage.getItem("zenpaws.username") ?? ""
  );
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);

  const submit = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      const trimmed = username.trim();
      if (!trimmed || trimmed.length > 32) {
        setError("Choose a name between 1 and 32 characters.");
        return;
      }
      setStarting(true);
      setError(null);
      try {
        const status = await invoke<NetworkStatus>("start_network", {
          username: trimmed,
        });
        window.localStorage.setItem("zenpaws.peerId", status.peerId);
        window.localStorage.setItem("zenpaws.username", trimmed);
        onReady();
      } catch (error: unknown) {
        setError(formatStartupError(error));
        setStarting(false);
      }
    },
    [onReady, username]
  );

  return (
    <main className="setup-page">
      <motion.section
        animate={{ opacity: 1, y: 0 }}
        className="setup-card"
        initial={reduceMotion ? false : { opacity: 0, y: 16 }}
        transition={reduceMotion ? { duration: 0 } : { duration: 0.35 }}
      >
        <div className="brand-mark" aria-hidden="true">
          🐾
        </div>
        <p className="eyebrow">Welcome to ZenPaws</p>
        <h1>Your cozy LAN chat</h1>
        <p className="setup-copy">
          Pick a name for this device. ZenPaws keeps your messages on your local
          network.
        </p>
        <form onSubmit={submit}>
          <label htmlFor="username">Your name</label>
          <input
            autoComplete="username"
            id="username"
            maxLength={32}
            onChange={(event) => setUsername(event.target.value)}
            placeholder="e.g. Maya"
            value={username}
          />
          {error ? (
            <p className="form-error" role="alert">
              {error}
            </p>
          ) : null}
          <button className="primary-button" disabled={starting} type="submit">
            {starting ? "Joining your LAN..." : "Enter ZenPaws"}
          </button>
        </form>
      </motion.section>
    </main>
  );
}
