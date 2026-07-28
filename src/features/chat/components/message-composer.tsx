import { useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useRef, useState } from "react";
import type { FormEvent, KeyboardEvent } from "react";

interface MessageComposerProps {
  onCancelReply: () => void;
  onSent: () => void;
  replyTo: string | null;
  room: string;
}

export function MessageComposer({
  onCancelReply,
  onSent,
  replyTo,
  room,
}: MessageComposerProps) {
  const [body, setBody] = useState("");
  const [error, setError] = useState<string | null>(null);
  const client = useQueryClient();
  const formRef = useRef<HTMLFormElement>(null);
  const composerRef = useRef<HTMLDivElement>(null);
  const typingTimeout = useRef<number | null>(null);

  const stopTyping = useCallback(() => {
    invoke("set_typing", { isTyping: false, room }).catch(() => {});
  }, [room]);

  const updateTyping = useCallback(
    (nextBody: string) => {
      setBody(nextBody);
      invoke("set_typing", {
        isTyping: nextBody.trim().length > 0,
        room,
      }).catch(() => {});
      if (typingTimeout.current !== null) {
        window.clearTimeout(typingTimeout.current);
      }
      typingTimeout.current = window.setTimeout(stopTyping, 2000);
    },
    [room, stopTyping]
  );

  const changeBody = useCallback(
    (event: FormEvent<HTMLDivElement>) => {
      updateTyping(event.currentTarget.textContent ?? "");
    },
    [updateTyping]
  );
  const handleKeyDown = useCallback((event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      formRef.current?.requestSubmit();
    }
  }, []);

  const submit = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      try {
        await invoke("send_message", { body, replyTo, room });
        setBody("");
        if (composerRef.current) {
          composerRef.current.textContent = "";
        }
        stopTyping();
        setError(null);
        onSent();
        await client.invalidateQueries({ queryKey: ["messages", room] });
      } catch (error: unknown) {
        setError(
          error instanceof Error ? error.message : "Could not send message."
        );
      }
    },
    [body, client, onSent, replyTo, room, stopTyping]
  );

  return (
    <form className="message-composer" onSubmit={submit} ref={formRef}>
      {replyTo ? (
        <p>
          Replying to {replyTo}{" "}
          <button onClick={onCancelReply} type="button">
            Cancel reply
          </button>
        </p>
      ) : null}
      <label htmlFor="message-body">Message</label>
      <div
        aria-label="Message"
        aria-multiline="true"
        className="message-input"
        contentEditable
        data-placeholder="Write a message..."
        onBlur={stopTyping}
        onInput={changeBody}
        onKeyDown={handleKeyDown}
        ref={composerRef}
        role="textbox"
        tabIndex={0}
        suppressContentEditableWarning
      />
      {error ? <p role="alert">{error}</p> : null}
      <button disabled={body.trim().length === 0} type="submit">
        Send
      </button>
    </form>
  );
}
