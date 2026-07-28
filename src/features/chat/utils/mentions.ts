export interface MessagePart {
  id: string;
  text: string;
  type: "mention" | "text";
}

const MENTION_PATTERN = /@[a-zA-Z0-9_-]{1,32}/gu;

export function parseMentions(body: string): MessagePart[] {
  const parts: MessagePart[] = [];
  let cursor = 0;
  for (const match of body.matchAll(MENTION_PATTERN)) {
    const index = match.index ?? cursor;
    if (index > cursor) {
      parts.push({
        id: `text-${cursor}`,
        text: body.slice(cursor, index),
        type: "text",
      });
    }
    parts.push({ id: `mention-${index}`, text: match[0], type: "mention" });
    cursor = index + match[0].length;
  }
  if (cursor < body.length) {
    parts.push({
      id: `text-${cursor}`,
      text: body.slice(cursor),
      type: "text",
    });
  }
  return parts.length > 0
    ? parts
    : [{ id: "text-0", text: body, type: "text" }];
}
