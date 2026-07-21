import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef } from "react";
import { useMessages } from "../../../queries/messages";

interface MessageListProps {
  room: string;
}

export function MessageList({ room }: MessageListProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const messages = useMessages(room);
  const rows = useVirtualizer({
    count: messages.data?.length ?? 0,
    estimateSize: () => 52,
    getScrollElement: () => scrollRef.current,
    overscan: 8,
  });

  if (messages.isError) {
    return <p role="alert">Could not load messages.</p>;
  }

  return (
    <div aria-label="Messages" ref={scrollRef} style={{ height: "100%", overflow: "auto" }}>
      <div style={{ height: rows.getTotalSize(), position: "relative" }}>
        {rows.getVirtualItems().map((item) => {
          const message = messages.data?.[item.index];
          if (!message) {
            return null;
          }
          return (
            <article
              key={message.id}
              style={{ left: 0, position: "absolute", top: item.start, width: "100%" }}
            >
              <strong>{message.authorId}</strong>
              <p>{message.body}</p>
            </article>
          );
        })}
      </div>
    </div>
  );
}
