import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

interface MessageEventPayload {
  MessageAckReceived?: { message_id: string };
  MessageDeleted?: { message_id: string };
  MessageEdited?: { message_id: string };
  MessageReactionChanged?: { message_id: string };
  MessageReceived?: { room: string };
}

function isMessageEvent(value: unknown): value is MessageEventPayload {
  return typeof value === "object" && value !== null;
}

export function useMessageEvents(room: string): void {
  const client = useQueryClient();

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<unknown>("zenpaws://event", (event) => {
      if (!isMessageEvent(event.payload)) {
        return;
      }
      if (
        event.payload.MessageReceived?.room === room ||
        event.payload.MessageDeleted ||
        event.payload.MessageEdited ||
        event.payload.MessageReactionChanged
      ) {
        client.invalidateQueries({ queryKey: ["messages"] }).catch(() => null);
      }
      if (event.payload.MessageAckReceived) {
        client
          .invalidateQueries({
            queryKey: [
              "message-status",
              event.payload.MessageAckReceived.message_id,
            ],
          })
          .catch(() => null);
      }
    })
      .then((dispose) => {
        unlisten = dispose;
      })
      .catch(() => null);
    return () => unlisten?.();
  }, [client, room]);
}
