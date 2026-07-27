import { SmilePlus } from "lucide-react";
import { useCallback, useState } from "react";

import { Button } from "../ui/button";

const REACTIONS = ["👍", "❤️", "😂", "😮", "🙏", "🔥"] as const;

interface ChatcnReactionPickerProps {
  onSelect: (emoji: string) => void;
}

export function ChatcnReactionPicker({ onSelect }: ChatcnReactionPickerProps) {
  const [open, setOpen] = useState(false);
  const toggle = useCallback(() => setOpen((current) => !current), []);
  const select = useCallback(
    (emoji: string) => {
      onSelect(emoji);
      setOpen(false);
    },
    [onSelect]
  );

  return (
    <div className="relative">
      <Button
        aria-label="Add reaction"
        onClick={toggle}
        size="icon"
        variant="ghost"
      >
        <SmilePlus />
      </Button>
      {open ? (
        <div
          aria-label="Reaction picker"
          className="chatcn-reaction-picker"
          role="menu"
        >
          {REACTIONS.map((emoji) => (
            <button
              key={emoji}
              onClick={() => select(emoji)}
              role="menuitem"
              type="button"
            >
              {emoji}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
