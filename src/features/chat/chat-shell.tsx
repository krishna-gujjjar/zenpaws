import { useCallback, useState } from "react";
import { usePeerPresence } from "../../hooks/use-peer-presence";
import { useTypingEvents } from "../../hooks/use-typing-events";
import type { ChatMessage } from "../../queries/messages";
import { MessageComposer } from "./components/message-composer";
import { MessageList } from "./components/message-list";
import { RoomTabs } from "./components/room-tabs";
import { directRoom, SHARED_ROOM } from "./utils/room";

export function ChatShell() {
  const [reply, setReply] = useState<ChatMessage | null>(null);
  const [room, setRoom] = useState(SHARED_ROOM);
  const typingPeers = useTypingEvents(room);
  const peers = usePeerPresence();
  const cancelReply = useCallback(() => setReply(null), []);
  const rooms = [
    SHARED_ROOM,
    ...[...peers]
      .filter(([, state]) => state !== "Disconnected")
      .map(([peerId]) => directRoom(peerId)),
  ];

  return (
    <main style={{ display: "grid", gridTemplateRows: "auto minmax(0, 1fr)", height: "100vh" }}>
      <header style={{ padding: "1rem" }}>
        <h1 style={{ margin: 0 }}>Shared room</h1>
        <RoomTabs onSelect={setRoom} rooms={rooms} selectedRoom={room} />
      </header>
      {typingPeers.size > 0 ? <p>{typingPeers.size} peer typing...</p> : null}
      <MessageList onReply={setReply} room={room} />
      <MessageComposer onCancelReply={cancelReply} replyTo={reply?.id ?? null} room={room} />
    </main>
  );
}
