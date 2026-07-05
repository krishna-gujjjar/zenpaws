// Infer typing from cursor being idle
// When cursor stops moving, user is likely typing
export class TypingDetector {
  private lastMoveTime = Date.now();
  private keyTimestamps: number[] = [];
  private readonly IDLE_MS = 800; // cursor still for 800ms = typing
  private readonly OVERHEAT_WPM = 60;
  private readonly WINDOW_MS = 3000;

  // Call this every poll tick with current cursor position
  onCursorMove() {
    this.lastMoveTime = Date.now();
  }

  // Call this every poll tick
  // Returns null if not typing, 'knead' or 'overheat' if typing
  getTypingState(): "knead" | "overheat" | null {
    const idleMs = Date.now() - this.lastMoveTime;

    // Not idle enough — user is moving mouse, not typing
    if (idleMs < this.IDLE_MS) {
      return null;
    }

    return null; // idle but no key signal
  }

  // Call when a key event is received (from JS keydown)
  onKeyPress(): "knead" | "overheat" {
    const now = Date.now();
    this.keyTimestamps.push(now);
    this.keyTimestamps = this.keyTimestamps.filter(
      (t) => now - t < this.WINDOW_MS
    );
    const wpm = this.keyTimestamps.length / 5 / (this.WINDOW_MS / 60_000);
    return wpm >= this.OVERHEAT_WPM ? "overheat" : "knead";
  }

  reset() {
    this.keyTimestamps = [];
  }
}
