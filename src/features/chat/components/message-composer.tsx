import { invoke } from "@tauri-apps/api/core";
import { useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useState } from "react";

interface MessageComposerProps {
  room: string;
}

export function MessageComposer({ room }: MessageComposerProps) {
  const [body, setBody] = useState("");
  const [error, setError] = useState<string | null>(null);
  const client = useQueryClient();

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    try {
      await invoke("send_message", { body, room });
      setBody("");
      setError(null);
      await client.invalidateQueries({ queryKey: ["messages", room] });
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : "Could not send message.");
    }
  }

  return (
    <form onSubmit={submit}>
      <label htmlFor="message-body">Message</label>
      <textarea id="message-body" onChange={(event) => setBody(event.target.value)} value={body} />
      {error ? <p role="alert">{error}</p> : null}
      <button disabled={body.trim().length === 0} type="submit">Send</button>
    </form>
  );
}
