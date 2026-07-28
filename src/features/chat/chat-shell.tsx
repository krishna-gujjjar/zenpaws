import { useCallback, useState } from "react";

import { useMessageEvents } from "../../hooks/use-message-events";
import { usePeerPresence } from "../../hooks/use-peer-presence";
import { useTypingEvents } from "../../hooks/use-typing-events";
import type { ChatMessage } from "../../queries/messages";
import { MessageComposer } from "./components/message-composer";
import { MessageList } from "./components/message-list";
import { MessageSearch } from "./components/message-search";
import { NetworkDebug } from "./components/network-debug";
import { RoomTabs } from "./components/room-tabs";
import { directRoom, SHARED_ROOM } from "./utils/room";

export function ChatShell() {
  const [reply, setReply] = useState<ChatMessage | null>(null);
  const [room, setRoom] = useState(SHARED_ROOM);
  const [searchOpen, setSearchOpen] = useState(false);
  const [debugOpen, setDebugOpen] = useState(false);
  const typingPeers = useTypingEvents(room);
  useMessageEvents(room);
  const peers = usePeerPresence();
  const cancelReply = useCallback(() => setReply(null), []);
  const openSearch = useCallback(() => setSearchOpen(true), []);
  const closeSearch = useCallback(() => setSearchOpen(false), []);
  const closeDebug = useCallback(() => setDebugOpen(false), []);
  const openDebug = useCallback(() => setDebugOpen(true), []);
  const rooms = [
    SHARED_ROOM,
    ...[...peers]
      .filter(([, state]) => state !== "Disconnected")
      .map(([peerId]) => directRoom(peerId)),
  ];
  if (debugOpen) {
    return <NetworkDebug onClose={closeDebug} />;
  }

  const renderMessageArea = () => {
    if (debugOpen) {
      return <NetworkDebug onClose={closeDebug} />;
    }
    if (searchOpen) {
      return <MessageSearch onClose={closeSearch} onSelectRoom={setRoom} />;
    }
    return (
      <>
        {typingPeers.size > 0 ? (
          <p className="typing-indicator">{typingPeers.size} peer typing...</p>
        ) : null}
        <MessageList onReply={setReply} room={room} />
      </>
    );
  };

  return (
    <main className="chat-page">
      <aside className="chat-sidebar">
        <div className="sidebar-brand">
          <span aria-hidden="true" className="brand-mark">
            🐾
          </span>
          <div>
            <strong>ZenPaws</strong>
            <span>Local network</span>
          </div>
        </div>
        <p className="sidebar-label">Conversations</p>
        <RoomTabs onSelect={setRoom} rooms={rooms} selectedRoom={room} />
        <div className="sidebar-footer">
          <span className="online-dot" /> LAN connected
          <button className="debug-link" onClick={openDebug} type="button">
            Network diagnostics
          </button>
        </div>
      </aside>
      <section className="chat-panel">
        <header className="chat-header">
          <div>
            <p className="eyebrow">Conversation</p>
            <h1>{room === SHARED_ROOM ? "Shared room" : "Direct message"}</h1>
          </div>
          <div className="chat-header-actions">
            <button
              className="secondary-button"
              onClick={openSearch}
              type="button"
            >
              Search
            </button>
            <span className="connection-pill">
              <span className="online-dot" /> Secure LAN
            </span>
          </div>
        </header>
        <div className="message-area">{renderMessageArea()}</div>
        <MessageComposer
          onCancelReply={cancelReply}
          onSent={cancelReply}
          replyTo={reply?.id ?? null}
          room={room}
        />
      </section>
    </main>
  );
}
