import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

interface PeerStateEvent {
  PeerStateChanged: {
    peer_id: string;
    state: "Connecting" | "Connected" | "Degraded" | "Disconnected";
  };
}

export function usePeerPresence() {
  const [peers, setPeers] = useState<ReadonlyMap<string, string>>(new Map());

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<PeerStateEvent>("zenpaws://event", (event) => {
      const peer = event.payload.PeerStateChanged;
      if (!peer) {
        return;
      }
      setPeers((current) => new Map(current).set(peer.peer_id, peer.state));
    })
      .then((dispose) => {
        unlisten = dispose;
      })
      .catch(() => undefined);
    return () => unlisten?.();
  }, []);

  return peers;
}
