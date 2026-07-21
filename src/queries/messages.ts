import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";

export interface ChatMessage {
  authorId: string;
  body: string;
  createdAt: number;
  id: string;
  replyTo: string | null;
  room: string;
}

export interface ChatCursor {
  createdAt: number;
  id: string;
}

interface ListMessagesArgs {
  cursor?: ChatCursor;
  limit: number;
  room: string;
}

export function useMessages(room: string, cursor?: ChatCursor) {
  return useQuery({
    queryFn: () =>
      invoke<ChatMessage[]>("list_messages", {
        cursor,
        limit: 50,
        room,
      }),
    queryKey: ["messages", room, cursor?.createdAt, cursor?.id],
    staleTime: Infinity,
  });
}

export function useMessageSearch(query: string) {
  return useQuery({
    enabled: query.trim().length > 0,
    queryFn: () => invoke<ChatMessage[]>("search_messages", { limit: 50, query }),
    queryKey: ["message-search", query],
    staleTime: Infinity,
  });
}

export async function listMessages(args: ListMessagesArgs): Promise<ChatMessage[]> {
  return invoke<ChatMessage[]>("list_messages", args);
}
