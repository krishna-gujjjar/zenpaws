import { useCallback, useState } from 'react';
import type { FormEvent } from 'react';

import { useMessageSearch } from "../../../queries/messages";
import type { ChatMessage } from "../../../queries/messages";

interface MessageSearchProps {
  onClose: () => void;
  onSelectRoom: (room: string) => void;
}

export function MessageSearch({ onClose, onSelectRoom }: MessageSearchProps) {
  const [query, setQuery] = useState("");
  const [submittedQuery, setSubmittedQuery] = useState("");
  const results = useMessageSearch(submittedQuery);
  const submit = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      setSubmittedQuery(query.trim());
    },
    [query]
  );
  const selectResult = useCallback(
    (message: ChatMessage) => {
      onSelectRoom(message.room);
      onClose();
    },
    [onClose, onSelectRoom]
  );

  return (
    <section aria-label="Search messages" className="message-search">
      <form onSubmit={submit}>
        <label htmlFor="message-search-query">Search local messages</label>
        <div className="message-search-controls">
          <input
            id="message-search-query"
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search your history"
            value={query}
          />
          <button type="submit">Search</button>
          <button onClick={onClose} type="button">
            Close
          </button>
        </div>
      </form>
      {results.isError ? (
        <p className="form-error" role="alert">
          Search could not be completed.
        </p>
      ) : null}
      {results.data ? (
        <div className="search-results">
          {results.data.length === 0 ? (
            <p>No matching local messages.</p>
          ) : null}
          {results.data.map((message) => (
            <button
              key={message.id}
              onClick={() => selectResult(message)}
              type="button"
            >
              <strong>{message.authorName}</strong>
              <span>{message.body}</span>
            </button>
          ))}
        </div>
      ) : null}
    </section>
  );
}
