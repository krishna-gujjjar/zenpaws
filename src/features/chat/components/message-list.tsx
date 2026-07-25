import { useVirtualizer } from "@tanstack/react-virtual";
import { useCallback, useRef } from "react";
import { type ChatMessage, useInfiniteMessages } from "../../../queries/messages";
import { MessageItem } from "./message-item";

interface MessageListProps {
  onReply: (message: ChatMessage) => void;
  room: string;
}

export function MessageList({ onReply, room }: MessageListProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const messages = useInfiniteMessages(room);
  const items = messages.data?.pages.flat() ?? [];
  const rows = useVirtualizer({
    count: items.length,
    estimateSize: () => 52,
    getScrollElement: () => scrollRef.current,
    overscan: 8,
  });
  const loadOlder = useCallback(() => {
    messages.fetchNextPage().catch(() => undefined);
  }, [messages]);

  if (messages.isError) {
    return <p role="alert">Could not load messages.</p>;
  }

  return (
    <section aria-label="Messages" ref={scrollRef} style={{ height: "100%", overflow: "auto" }}>
      {messages.hasNextPage ? (
        <button onClick={loadOlder} type="button">
          Load older messages
        </button>
      ) : null}
      <div style={{ height: rows.getTotalSize(), position: "relative" }}>
        {rows.getVirtualItems().map((item) => {
          const message = items[item.index];
          if (!message) {
            return null;
          }
          return (
            <div
              key={message.id}
              style={{ left: 0, position: "absolute", top: item.start, width: "100%" }}
            >
              <MessageItem message={message} onReply={onReply} room={room} />
            </div>
          );
        })}
      </div>
    </section>
  );
}
