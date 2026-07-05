export class ShakeDetector {
  private lastX: number | null = null;
  private lastDirection = 0;
  private flipWindow: number[] = []; // timestamps of flips
  private readonly FLIP_THRESHOLD = 3; // flips needed
  private readonly FLIP_WINDOW_MS = 600; // within this time
  private readonly MIN_DELTA = 8; // min px movement to count

  update(x: number): boolean {
    if (this.lastX === null) {
      this.lastX = x;
      return false;
    }

    const delta = x - this.lastX;
    this.lastX = x;

    // Ignore tiny movements
    if (Math.abs(delta) < this.MIN_DELTA) {
      return false;
    }

    const direction = delta > 0 ? 1 : -1;

    // Direction changed = flip
    if (this.lastDirection !== 0 && direction !== this.lastDirection) {
      const now = Date.now();
      this.flipWindow.push(now);

      // Remove old flips outside the window
      this.flipWindow = this.flipWindow.filter(
        (t) => now - t < this.FLIP_WINDOW_MS
      );

      if (this.flipWindow.length >= this.FLIP_THRESHOLD) {
        this.flipWindow = []; // reset after triggering
        this.lastDirection = direction;
        return true; // SHAKE DETECTED
      }
    }

    this.lastDirection = direction;
    return false;
  }

  reset() {
    this.lastX = null;
    this.lastDirection = 0;
    this.flipWindow = [];
  }
}
