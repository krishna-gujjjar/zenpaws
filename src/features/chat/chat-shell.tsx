import { MessageComposer } from "./components/message-composer";
import { MessageList } from "./components/message-list";

const SHARED_ROOM = "shared";

export function ChatShell() {
  return (
    <main style={{ display: "grid", gridTemplateRows: "auto minmax(0, 1fr)", height: "100vh" }}>
      <header style={{ padding: "1rem" }}>
        <h1 style={{ margin: 0 }}>Shared room</h1>
      </header>
      <MessageList room={SHARED_ROOM} />
      <MessageComposer room={SHARED_ROOM} />
    </main>
  );
}
