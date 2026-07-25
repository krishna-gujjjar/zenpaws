import { useCallback } from "react";
import { roomLabel } from "../utils/room";

interface RoomTabsProps {
  onSelect: (room: string) => void;
  rooms: readonly string[];
  selectedRoom: string;
}

export function RoomTabs({ onSelect, rooms, selectedRoom }: RoomTabsProps) {
  return (
    <nav aria-label="Chat rooms">
      {rooms.map((room) => (
        <RoomTab active={room === selectedRoom} key={room} onSelect={onSelect} room={room} />
      ))}
    </nav>
  );
}

interface RoomTabProps {
  active: boolean;
  onSelect: (room: string) => void;
  room: string;
}

function RoomTab({ active, onSelect, room }: RoomTabProps) {
  const select = useCallback(() => onSelect(room), [onSelect, room]);

  return (
    <button aria-pressed={active} onClick={select} type="button">
      {roomLabel(room)}
    </button>
  );
}
