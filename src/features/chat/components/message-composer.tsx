import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { type ChangeEvent, type FormEvent, useCallback, useRef, useState } from "react";

interface MessageComposerProps {
  onCancelReply: () => void;
  replyTo: string | null;
  room: string;
}

export function MessageComposer({ onCancelReply, replyTo, room }: MessageComposerProps) {
  const [body, setBody] = useState("");
  const [error, setError] = useState<string | null>(null);
  const client = useQueryClient();
  const typingTimeout = useRef<number | null>(null);

  const stopTyping = useCallback(() => {
    invoke("set_typing", { isTyping: false, room }).catch(() => undefined);
  }, [room]);

  const updateTyping = useCallback(
    (nextBody: string) => {
      setBody(nextBody);
      invoke("set_typing", { isTyping: nextBody.trim().length > 0, room }).catch(() => undefined);
      if (typingTimeout.current !== null) {
        window.clearTimeout(typingTimeout.current);
      }
      typingTimeout.current = window.setTimeout(stopTyping, 2000);
    },
    [room, stopTyping],
  );

  const changeBody = useCallback(
    (event: ChangeEvent<HTMLTextAreaElement>) => {
      updateTyping(event.target.value);
    },
    [updateTyping],
  );

  const submit = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      try {
        await invoke("send_message", { body, replyTo, room });
        setBody("");
        stopTyping();
        setError(null);
        await client.invalidateQueries({ queryKey: ["messages", room] });
      } catch (reason: unknown) {
        setError(reason instanceof Error ? reason.message : "Could not send message.");
      }
    },
    [body, client, replyTo, room, stopTyping],
  );

  return (
    <form onSubmit={submit}>
      {replyTo ? (
        <p>
          Replying to {replyTo}{" "}
          <button onClick={onCancelReply} type="button">
            Cancel reply
          </button>
        </p>
      ) : null}
      <label htmlFor="message-body">Message</label>
      <textarea id="message-body" onBlur={stopTyping} onChange={changeBody} value={body} />
      {error ? <p role="alert">{error}</p> : null}
      <button disabled={body.trim().length === 0} type="submit">
        Send
      </button>
    </form>
  );
}
