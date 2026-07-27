import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from 'react';
import type { ChangeEvent, MouseEvent } from 'react';

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
  const peerId = isDirectRoom(room) ? room.slice(3) : null;
  const localPeerId = window.localStorage.getItem("zenpaws.peerId");
  const authorLabel =
    message.authorId === localPeerId
      ? "You"
      : message.authorName || message.authorId;

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
  const react = useCallback(async () => {
    await invoke("add_reaction", { emoji: "👍", messageId: message.id, room });
    await refreshMessages();
  }, [message.id, refreshMessages, room]);
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
  const changeBody = useCallback((event: ChangeEvent<HTMLTextAreaElement>) => {
    setBody(event.target.value);
  }, []);
  const reply = useCallback(() => {
    onReply(message);
    setContextMenu(null);
  }, [message, onReply]);
  const startEditing = useCallback(() => {
    setEditing(true);
    setContextMenu(null);
  }, []);
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
        <strong>{authorLabel}</strong>
        {editing ? (
          <>
            <textarea
              aria-label="Edit message"
              onChange={changeBody}
              value={body}
            />
            <button onClick={saveEdit} type="button">
              Save message
            </button>
          </>
        ) : (
          <p className="message-body">
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
          </p>
        )}
        {message.replyTo ? <small>Replying to {message.replyTo}</small> : null}
        <div className="message-meta">
          <span>
            {messageStatusLabel(
              room,
              status.data?.read,
              status.data?.delivered
            )}
          </span>
        </div>
        <button
          aria-label="React with thumbs up"
          className="hover-reaction"
          onClick={react}
          type="button"
        >
          👍
        </button>
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
          <button onClick={closeContextMenu} role="menuitem" type="button">
            Close menu
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
