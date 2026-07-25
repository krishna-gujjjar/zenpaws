const SHARED_ROOM = "shared";

export function directRoom(peerId: string): string {
  return `dm:${peerId}`;
}

export function isDirectRoom(room: string): boolean {
  return room.startsWith("dm:") && room.length > 3;
}

export function roomLabel(room: string): string {
  return room === SHARED_ROOM ? "Shared room" : "Direct message";
}

export { SHARED_ROOM };
