import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

interface TypingEvent {
  TypingChanged: {
    is_typing: boolean;
    peer_id: string;
    room: string;
  };
}

export function useTypingEvents(room: string) {
  const [typingPeers, setTypingPeers] = useState<ReadonlySet<string>>(
    new Set()
  );
  const expiry = useRef(new Map<string, number>());

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<TypingEvent>("zenpaws://event", (event) => {
      const typing = event.payload.TypingChanged;
      if (!typing || typing.room !== room) {
        return;
      }
      setTypingPeers((peers) => {
        const next = new Set(peers);
        if (typing.is_typing) {
          const previous = expiry.current.get(typing.peer_id);
          if (previous !== undefined) {
            window.clearTimeout(previous);
          }
          next.add(typing.peer_id);
          expiry.current.set(
            typing.peer_id,
            window.setTimeout(() => {
              setTypingPeers((active) => {
                const updated = new Set(active);
                updated.delete(typing.peer_id);
                return updated;
              });
            }, 3000)
          );
        } else {
          const timeout = expiry.current.get(typing.peer_id);
          if (timeout !== undefined) {
            window.clearTimeout(timeout);
          }
          next.delete(typing.peer_id);
        }
        return next;
      });
    })
      .then((dispose) => {
        unlisten = dispose;
      })
      .catch(() => {});
    return () => unlisten?.();
  }, [room]);

  return typingPeers;
}
