import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import {
  type FormEvent,
  type MouseEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

import { ChatcnReactionPicker } from "../../../components/chat/chatcn-reaction-picker";
import { MessageContent } from "../../../components/chatcn/ai/message";
import { Avatar, AvatarFallback } from "../../../components/ui/avatar";
import { useMessageStatus } from "../../../queries/message-status";
import type { ChatMessage } from "../../../queries/messages";
import { parseMentions } from "../utils/mentions";
import { isDirectRoom } from "../utils/room";

interface MessageItemProps {
  message: ChatMessage;
  onReply: (message: ChatMessage) => void;
  room: string;
}

export function MessageItem({ message, onReply, room }: MessageItemProps) {
  const [editing, setEditing] = useState(false);
  const [body, setBody] = useState(message.body);
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
  } | null>(null);
  const client = useQueryClient();
  const status = useMessageStatus(message.id, room);
  const editRef = useRef<HTMLDivElement>(null);
  const peerId = isDirectRoom(room) ? room.slice(3) : null;
  const localPeerId = window.localStorage.getItem("zenpaws.peerId");
  const authorLabel =
    message.authorId === localPeerId
      ? "You"
      : message.authorName || message.authorId;
  const reactionCounts = groupReactions(message.reactions);

  useEffect(() => {
    if (peerId === null) {
      return;
    }
    invoke("mark_message_read", { messageId: message.id, peerId }).catch(
      () => null
    );
  }, [message.id, peerId]);

  const refreshMessages = useCallback(
    () => client.invalidateQueries({ queryKey: ["messages", room] }),
    [client, room]
  );
  const react = useCallback(
    async (emoji: string) => {
      await invoke("add_reaction", { emoji, messageId: message.id, room });
      await refreshMessages();
    },
    [message.id, refreshMessages, room]
  );
  const remove = useCallback(async () => {
    await invoke("delete_message", { id: message.id, room });
    await refreshMessages();
    setContextMenu(null);
  }, [message.id, refreshMessages, room]);
  const copy = useCallback(async () => {
    await navigator.clipboard.writeText(message.body);
    setContextMenu(null);
  }, [message.body]);
  const saveEdit = useCallback(async () => {
    await invoke("edit_message", { body, id: message.id, room });
    await refreshMessages();
    setEditing(false);
    setContextMenu(null);
  }, [body, message.id, refreshMessages, room]);
  const changeBody = useCallback((event: FormEvent<HTMLDivElement>) => {
    setBody(event.currentTarget.textContent ?? "");
  }, []);
  const reply = useCallback(() => {
    onReply(message);
    setContextMenu(null);
  }, [message, onReply]);
  const startEditing = useCallback(() => {
    setEditing(true);
    setContextMenu(null);
  }, []);
  useEffect(() => {
    if (editing && editRef.current) {
      editRef.current.textContent = body;
    }
  }, [editing]);

  const openContextMenu = useCallback((event: MouseEvent<HTMLElement>) => {
    event.preventDefault();
    setContextMenu({ x: event.clientX, y: event.clientY });
  }, []);
  const menuRef = useRef<HTMLDivElement>(null);
  const closeContextMenu = useCallback(() => setContextMenu(null), []);

  useEffect(() => {
    if (contextMenu === null) {
      return;
    }
    const closeOnOutsideClick = (event: globalThis.PointerEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        closeContextMenu();
      }
    };
    const closeOnEscape = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape") {
        closeContextMenu();
      }
    };
    document.addEventListener("pointerdown", closeOnOutsideClick);
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("pointerdown", closeOnOutsideClick);
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, [closeContextMenu, contextMenu]);

  return (
    <article className="message-item" onContextMenu={openContextMenu}>
      <div className="message-content">
        <div className="message-author">
          <Avatar className="message-avatar">
            <AvatarFallback>
              {authorLabel.slice(0, 2).toUpperCase()}
            </AvatarFallback>
          </Avatar>
          <strong>{authorLabel}</strong>
        </div>
        {message.deleted ? (
          <p className="message-deleted">Message deleted</p>
        ) : editing ? (
          <>
            <div
              aria-label="Edit message"
              className="message-input edit-message-input"
              contentEditable
              onInput={changeBody}
              ref={editRef}
              role="textbox"
              suppressContentEditableWarning
            />
            <button onClick={saveEdit} type="button">
              Save message
            </button>
          </>
        ) : (
          <MessageContent className="message-body">
            {parseMentions(message.body).map((part) => (
              <span
                className={
                  part.type === "mention" ? "message-mention" : undefined
                }
                key={part.id}
              >
                {part.text}
              </span>
            ))}
          </MessageContent>
        )}
        {reactionCounts.length > 0 ? (
          <div aria-label="Message reactions" className="message-reactions">
            {reactionCounts.map((reaction) => (
              <span key={reaction.emoji}>
                {reaction.emoji} {reaction.count}
              </span>
            ))}
          </div>
        ) : null}
        {message.replyTo ? (
          <div className="reply-preview">
            <strong>{message.replyAuthorName ?? "Message"}</strong>
            <span>{message.replyBody ?? "Original message unavailable"}</span>
          </div>
        ) : null}
        <div className="message-meta">
          <span>
            {messageStatusLabel(
              room,
              status.data?.read,
              status.data?.delivered
            )}
          </span>
        </div>
        <div className="hover-reaction">
          <ChatcnReactionPicker onSelect={react} />
        </div>
      </div>
      {contextMenu ? (
        <div
          aria-label="Message context menu"
          className="message-context-menu"
          ref={menuRef}
          style={{ left: contextMenu.x, top: contextMenu.y }}
          role="menu"
        >
          <button onClick={copy} role="menuitem" type="button">
            Copy message
          </button>
          <button onClick={reply} role="menuitem" type="button">
            Reply
          </button>
          <button onClick={startEditing} role="menuitem" type="button">
            Edit message
          </button>
          <button onClick={remove} role="menuitem" type="button">
            Delete message
          </button>
        </div>
      ) : null}
    </article>
  );
}

function messageStatusLabel(
  room: string,
  read = false,
  delivered = false
): string {
  if (room === "shared") {
    return "Sent";
  }
  if (read) {
    return "Read";
  }
  return delivered ? "Delivered" : "Sent";
}

function groupReactions(
  reactions: readonly string[]
): Array<{ count: number; emoji: string }> {
  const counts = new Map<string, number>();
  for (const emoji of reactions) {
    counts.set(emoji, (counts.get(emoji) ?? 0) + 1);
  }
  return [...counts].map(([emoji, count]) => ({ count, emoji }));
}
