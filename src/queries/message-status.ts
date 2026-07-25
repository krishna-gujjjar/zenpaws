import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { isDirectRoom } from "../features/chat/utils/room";

export interface MessageStatus {
  delivered: boolean;
  read: boolean;
}

export function useMessageStatus(messageId: string, room: string) {
  const peerId = isDirectRoom(room) ? room.slice(3) : null;
  return useQuery({
    enabled: peerId !== null,
    queryFn: () => invoke<MessageStatus>("message_status", { messageId, peerId }),
    queryKey: ["message-status", messageId, peerId],
    staleTime: Number.POSITIVE_INFINITY,
  });
}
