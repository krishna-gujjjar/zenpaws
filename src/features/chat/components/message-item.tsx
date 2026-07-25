import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { type ChangeEvent, useCallback, useState } from "react";
import { useMessageStatus } from "../../../queries/message-status";
import type { ChatMessage } from "../../../queries/messages";

interface MessageItemProps {
  message: ChatMessage;
  onReply: (message: ChatMessage) => void;
  room: string;
}

export function MessageItem({ message, onReply, room }: MessageItemProps) {
  const [editing, setEditing] = useState(false);
  const [body, setBody] = useState(message.body);
  const client = useQueryClient();
  const status = useMessageStatus(message.id, room);

  const refreshMessages = useCallback(
    () => client.invalidateQueries({ queryKey: ["messages", room] }),
    [client, room],
  );
  const react = useCallback(async () => {
    await invoke("add_reaction", { emoji: "👍", messageId: message.id });
    await refreshMessages();
  }, [message.id, refreshMessages]);
  const remove = useCallback(async () => {
    await invoke("delete_message", { id: message.id });
    await refreshMessages();
  }, [message.id, refreshMessages]);
  const copy = useCallback(() => navigator.clipboard.writeText(message.body), [message.body]);
  const saveEdit = useCallback(async () => {
    await invoke("edit_message", { body, id: message.id });
    await refreshMessages();
    setEditing(false);
  }, [body, message.id, refreshMessages]);
  const changeBody = useCallback((event: ChangeEvent<HTMLTextAreaElement>) => {
    setBody(event.target.value);
  }, []);
  const reply = useCallback(() => onReply(message), [message, onReply]);
  const startEditing = useCallback(() => setEditing(true), []);

  return (
    <article>
      <strong>{message.authorId}</strong>
      {editing ? (
        <>
          <textarea aria-label="Edit message" onChange={changeBody} value={body} />
          <button onClick={saveEdit} type="button">
            Save
          </button>
        </>
      ) : (
        <p>{message.body}</p>
      )}
      {message.replyTo ? <small>Replying to {message.replyTo}</small> : null}
      <fieldset>
        <legend>Message actions</legend>
        <button onClick={copy} type="button">
          Copy
        </button>
        <button onClick={reply} type="button">
          Reply
        </button>
        <button onClick={startEditing} type="button">
          Edit
        </button>
        <button onClick={react} type="button">
          React
        </button>
        <button onClick={remove} type="button">
          Delete
        </button>
        <span>{messageStatusLabel(room, status.data?.read, status.data?.delivered)}</span>
      </fieldset>
    </article>
  );
}

function messageStatusLabel(room: string, read = false, delivered = false): string {
  if (room === "shared") {
    return "Sent";
  }
  if (read) {
    return "Read";
  }
  return delivered ? "Delivered" : "Sent";
}
