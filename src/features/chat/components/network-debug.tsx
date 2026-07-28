import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";

interface NetworkDiagnostics {
  connectedPeers: number;
  mdnsAvailable: boolean;
  tcpAddress: string;
  udpAddress: string;
}

interface NetworkDebugProps {
  onClose: () => void;
}

export function NetworkDebug({ onClose }: NetworkDebugProps) {
  const [diagnostics, setDiagnostics] = useState<NetworkDiagnostics | null>(
    null
  );
  const [error, setError] = useState<string | null>(null);
  const refresh = useCallback(async () => {
    try {
      setDiagnostics(await invoke<NetworkDiagnostics>("network_diagnostics"));
      setError(null);
    } catch (error: unknown) {
      setError(
        error instanceof Error ? error.message : "Diagnostics unavailable."
      );
    }
  }, []);

  return (
    <section aria-label="Network diagnostics" className="network-debug">
      <div className="network-debug-header">
        <div>
          <p className="eyebrow">Troubleshooting</p>
          <h2>LAN diagnostics</h2>
        </div>
        <button onClick={onClose} type="button">
          Close
        </button>
      </div>
      <button onClick={refresh} type="button">
        Refresh diagnostics
      </button>
      {error ? (
        <p className="form-error" role="alert">
          {error}
        </p>
      ) : null}
      {diagnostics ? (
        <dl>
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
      ) : null}
    </section>
  );
}