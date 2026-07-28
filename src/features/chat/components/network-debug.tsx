import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";

interface NetworkDiagnostics {
  connectedPeers: number;
  localAddress: string | null;
  mdnsAvailable: boolean;
  recentLogs: string[];
  tcpAddress: string;
  udpAddress: string;
}

interface NetworkDebugProps {
  onClose: () => void;
}

function formatDiagnosticsError(reason: unknown): string {
  if (typeof reason === "string" && reason.length > 0) {
    if (reason.includes("Command network_diagnostics not found")) {
      return "The Rust backend is stale. Stop the running app and restart with bun run tauri dev.";
    }
    return reason;
  }
  if (reason instanceof Error) {
    return reason.message;
  }
  return "Diagnostics unavailable.";
}

export function NetworkDebug({ onClose }: NetworkDebugProps) {
  const [diagnostics, setDiagnostics] = useState<NetworkDiagnostics | null>(
    null
  );
  const [error, setError] = useState<string | null>(null);
  const refresh = useCallback(async () => {
    try {
      const response = await invoke<Partial<NetworkDiagnostics>>(
        "network_diagnostics"
      );
      setDiagnostics({
        connectedPeers: response.connectedPeers ?? 0,
        localAddress: response.localAddress ?? null,
        mdnsAvailable: response.mdnsAvailable ?? false,
        recentLogs: Array.isArray(response.recentLogs)
          ? response.recentLogs
          : [],
        tcpAddress: response.tcpAddress ?? "Unavailable",
        udpAddress: response.udpAddress ?? "Unavailable",
      });
      setError(null);
    } catch (reason: unknown) {
      setError(formatDiagnosticsError(reason));
    }
  }, []);

  return (
    <main aria-label="Network diagnostics" className="network-debug-screen">
      <section className="network-debug">
        <div className="network-debug-header">
          <div>
            <p className="eyebrow">Troubleshooting</p>
            <h2>LAN diagnostics</h2>
            <p>
              Use these details to compare both computers and check firewall
              rules.
            </p>
          </div>
          <button onClick={onClose} type="button">
            Back to chat
          </button>
        </div>
        <div className="network-debug-actions">
          <button onClick={refresh} type="button">
            Refresh diagnostics
          </button>
          <span>Run this on both computers.</span>
        </div>
        {error ? (
          <p className="form-error" role="alert">
            {error}
          </p>
        ) : null}
        {diagnostics ? (
          <>
            <dl>
              <div>
                <dt>LAN address</dt>
                <dd>{diagnostics.localAddress ?? "Unavailable"}</dd>
              </div>
              <div>
                <dt>mDNS</dt>
                <dd>
                  {diagnostics.mdnsAvailable
                    ? "Available"
                    : "Unavailable; UDP fallback active"}
                </dd>
              </div>
              <div>
                <dt>TCP listener</dt>
                <dd>{diagnostics.tcpAddress}</dd>
              </div>
              <div>
                <dt>UDP socket</dt>
                <dd>{diagnostics.udpAddress}</dd>
              </div>
              <div>
                <dt>Connected peers</dt>
                <dd>{diagnostics.connectedPeers}</dd>
              </div>
            </dl>
            <div className="network-help">
              <h3>Checklist</h3>
              <ul>
                <li>
                  Both LAN addresses should belong to the same private subnet.
                </li>
                <li>Allow ZenPaws through Windows Defender Firewall.</li>
                <li>Allow Local Network access for ZenPaws on macOS.</li>
                <li>
                  Disable router client isolation or guest Wi-Fi isolation.
                </li>
                <li>
                  Verify both computers are not using separate guest networks.
                </li>
              </ul>
            </div>
            <div className="network-log" aria-label="Network event log">
              <h3>Recent backend events</h3>
              {diagnostics.recentLogs.length === 0 ? (
                <p>No events recorded.</p>
              ) : null}
              {diagnostics.recentLogs.map((log) => (
                <p key={log}>{log}</p>
              ))}
            </div>
          </>
        ) : null}
      </section>
    </main>
  );
}
