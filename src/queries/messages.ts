import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import type { InfiniteData } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export interface ChatMessage {
  authorId: string;
  authorName: string;
  body: string;
  createdAt: number;
  deleted: boolean;
  id: string;
  replyTo: string | null;
  reactions: string[];
  replyAuthorName: string | null;
  replyBody: string | null;
  room: string;
}

export interface ChatCursor {
  createdAt: number;
  id: string;
}

interface ListMessagesArgs extends Record<string, unknown> {
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
    staleTime: Number.POSITIVE_INFINITY,
  });
}

export function useInfiniteMessages(room: string) {
  return useInfiniteQuery<
    ChatMessage[],
    Error,
    InfiniteData<ChatMessage[]>,
    readonly ["messages", string],
    ChatCursor | undefined
  >({
    getNextPageParam: (page) => {
      const last = page.at(-1);
      return last ? { createdAt: last.createdAt, id: last.id } : undefined;
    },
    initialPageParam: undefined as ChatCursor | undefined,
    queryFn: ({ pageParam }) =>
      invoke<ChatMessage[]>("list_messages", {
        cursor: pageParam,
        limit: 50,
        room,
      }),
    queryKey: ["messages", room],
    staleTime: Number.POSITIVE_INFINITY,
  });
}

export function useMessageSearch(query: string) {
  return useQuery({
    enabled: query.trim().length > 0,
    queryFn: () =>
      invoke<ChatMessage[]>("search_messages", { limit: 50, query }),
    queryKey: ["message-search", query],
    staleTime: Number.POSITIVE_INFINITY,
  });
}

export async function listMessages(
  args: ListMessagesArgs
): Promise<ChatMessage[]> {
  return await invoke<ChatMessage[]>("list_messages", args);
}
