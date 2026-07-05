<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import Cat from "$lib/components/cat.svelte";
  import { isCursorOnHead } from "$lib/utils/pet-detector";
  import { ShakeDetector } from "$lib/utils/shake-detector";
  import { TypingDetector } from "$lib/utils/typing-detector";

  type Timer = ReturnType<typeof setTimeout> | null;
  type Interval = ReturnType<typeof setInterval> | null;

  const appWindow = getCurrentWindow();
  const shakeDetector = new ShakeDetector();
  const typingDetector = new TypingDetector();

  let isDragging = false;
  let isWobbling = false;
  let pupilOffsetX = 0;
  let pupilOffsetY = 0;
  let kneadFrame = 0;
  let paperLength = 0;
  let catState: "idle" | "hunt" | "pet" | "knead" | "overheat" | "scroll" =
    "idle";

  let prevX = 0;
  let prevY = 0;

  let huntTimeout: Timer = null;
  let wobbleTimeout: Timer = null;
  let petHoverStart: number | null = null;
  let petTimeout: Timer = null;
  let kneadTimeout: Timer = null;
  let kneadInterval: Interval = null;
  let scrollTimeout: Timer = null;

  // --- Wobble ---
  function triggerWobble() {
    isWobbling = false;
    setTimeout(() => {
      isWobbling = true;
      if (wobbleTimeout) {
        clearTimeout(wobbleTimeout);
      }
      wobbleTimeout = setTimeout(() => (isWobbling = false), 500);
    }, 10);
  }

  // --- Knead ---
  function startKneadAnimation() {
    if (kneadInterval) {
      return;
    }
    kneadInterval = setInterval(() => {
      kneadFrame = kneadFrame === 0 ? 1 : 0;
    }, 250);
  }

  function stopKneadAnimation() {
    if (kneadInterval) {
      clearInterval(kneadInterval);
      kneadInterval = null;
    }
  }

  // --- Drag ---
  function startDragging(e: MouseEvent) {
    if (e.button !== 0) {
      return;
    }
    e.preventDefault();
    isDragging = true;
    catState = "idle";
    shakeDetector.reset();
    appWindow
      .setFocus()
      .then(() => appWindow.startDragging())
      .catch((err: unknown) => {
        // Drag cancelled or window unfocused — not a fatal error
        console.debug("[Comnyang] drag cancelled", err);
      });
  }

  function stopDragging() {
    isDragging = false;
    shakeDetector.reset();
  }

  // --- Poll sub-functions (reduce complexity) ---

  function updateEyes(
    cursorX: number,
    cursorY: number,
    winX: number,
    winY: number,
    winW: number,
    winH: number
  ) {
    const centerX = winX + winW / 2;
    const centerY = winY + winH / 2;
    const dx = cursorX - centerX;
    const dy = cursorY - centerY;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const ratio = Math.min(dist / 150, 1);
    const angle = Math.atan2(dy, dx);
    pupilOffsetX = Math.cos(angle) * ratio * 0.5;
    pupilOffsetY = Math.sin(angle) * ratio * 0.5;
  }

  function updatePetState(
    cursorX: number,
    cursorY: number,
    winX: number,
    winY: number,
    winW: number,
    winH: number
  ) {
    const onHead = isCursorOnHead(cursorX, cursorY, winX, winY, winW, winH);

    if (
      onHead &&
      catState !== "hunt" &&
      catState !== "knead" &&
      catState !== "overheat" &&
      catState !== "scroll"
    ) {
      if (petHoverStart === null) {
        petHoverStart = Date.now();
      } else if (Date.now() - petHoverStart >= 300) {
        catState = "pet";
        if (petTimeout) {
          clearTimeout(petTimeout);
        }
        petTimeout = setTimeout(() => {
          catState = "idle";
          petHoverStart = null;
        }, 1200);
      }
    } else if (catState !== "pet") {
      petHoverStart = null;
    }
  }

  function updateHuntState(speed: number) {
    if (catState !== "idle" && catState !== "hunt") {
      return;
    }
    if (speed > 60) {
      catState = "hunt";
      if (huntTimeout) {
        clearTimeout(huntTimeout);
      }
      huntTimeout = setTimeout(() => (catState = "idle"), 2000);
    }
  }

  async function updateShake() {
    const [winX] = await invoke<[number, number]>("get_window_position");
    const shook = shakeDetector.update(winX);
    if (shook) {
      triggerWobble();
    }
  }

  async function pollTick() {
    const [cursorX, cursorY] = await invoke<[number, number]>(
      "get_cursor_position"
    );
    const winPos = await appWindow.outerPosition();
    const winSize = await appWindow.outerSize();

    updateEyes(
      cursorX,
      cursorY,
      winPos.x,
      winPos.y,
      winSize.width,
      winSize.height
    );

    const speed = Math.sqrt((cursorX - prevX) ** 2 + (cursorY - prevY) ** 2);

    if (!isDragging) {
      updatePetState(
        cursorX,
        cursorY,
        winPos.x,
        winPos.y,
        winSize.width,
        winSize.height
      );
      updateHuntState(speed);
    }

    if (isDragging) {
      await updateShake();
    }

    prevX = cursorX;
    prevY = cursorY;
  }

  onMount(() => {
    // --- Keyboard ---
    const unlistenKey = listen("key-typed", () => {
      if (isDragging || catState === "scroll") {
        return;
      }

      const result = typingDetector.onKeyPress();
      catState = result;

      if (result === "knead") {
        startKneadAnimation();
      } else {
        stopKneadAnimation();
      }

      if (kneadTimeout) {
        clearTimeout(kneadTimeout);
      }
      kneadTimeout = setTimeout(() => {
        catState = "idle";
        stopKneadAnimation();
        typingDetector.reset();
      }, 1500);
    });

    // --- Scroll ---
    const unlistenScroll = listen<number>("scroll-event", (event) => {
      if (isDragging) {
        return;
      }

      catState = "scroll";
      stopKneadAnimation();

      paperLength = Math.min(paperLength + Math.abs(event.payload) * 0.5, 20);

      if (scrollTimeout) {
        clearTimeout(scrollTimeout);
      }
      scrollTimeout = setTimeout(() => {
        catState = "idle";
        const retract = setInterval(() => {
          paperLength = Math.max(paperLength - 2, 0);
          if (paperLength <= 0) {
            clearInterval(retract);
          }
        }, 50);
      }, 1000);
    });

    // --- Poll interval (reduced complexity by extracting sub-functions) ---
    const interval = setInterval(() => {
      pollTick().catch((_err: unknown) => {
        // Silently ignore poll errors (window not ready, etc.)
      });
    }, 50);

    return () => {
      clearInterval(interval);
      stopKneadAnimation();
      unlistenKey.then((f) => f());
      unlistenScroll.then((f) => f());
      if (huntTimeout) {
        clearTimeout(huntTimeout);
      }
      if (wobbleTimeout) {
        clearTimeout(wobbleTimeout);
      }
      if (petTimeout) {
        clearTimeout(petTimeout);
      }
      if (kneadTimeout) {
        clearTimeout(kneadTimeout);
      }
      if (scrollTimeout) {
        clearTimeout(scrollTimeout);
      }
    };
  });
</script>

<svelte:window on:mouseup={stopDragging} />

<div class="stage" on:mousedown={startDragging} role="presentation">
  <div
    class="cat-container"
    class:bounce={!isDragging && catState === "idle"}
    class:hunt-wiggle={catState === "hunt" && !isDragging}
    class:knead-bob={catState === "knead" || catState === "overheat"}
  >
    <Cat
      {isDragging}
      {isWobbling}
      {kneadFrame}
      {paperLength}
      {pupilOffsetX}
      {pupilOffsetY}
      state={catState}
    />
  </div>
</div>

<style>
  .stage {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 160px;
    height: 160px;
    cursor: grab;
    background-color: rgba(0, 0, 0, 0.001);
  }

  .cat-container {
    width: 120px;
    height: 120px;
    pointer-events: none;
  }

  .bounce {
    animation: idle-float 3s infinite ease-in-out;
  }
  .hunt-wiggle {
    animation: hunt-wiggle 0.25s infinite alternate ease-in-out;
  }
  .knead-bob {
    animation: knead-bob 0.5s infinite alternate ease-in-out;
  }

  @keyframes idle-float {
    0%,
    100% {
      transform: translateX(0px) rotate(0deg);
    }
    25% {
      transform: translateX(2px) rotate(0.5deg);
    }
    75% {
      transform: translateX(-2px) rotate(-0.5deg);
    }
  }

  @keyframes hunt-wiggle {
    0% {
      transform: translateX(-3px) rotate(-2deg);
    }
    100% {
      transform: translateX(3px) rotate(2deg);
    }
  }

  @keyframes knead-bob {
    0% {
      transform: translateY(0px) rotate(-1deg);
    }
    100% {
      transform: translateY(2px) rotate(1deg);
    }
  }
</style>
