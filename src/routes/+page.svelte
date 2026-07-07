<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import Cat from "$lib/components/cat.svelte";
  import StretchSettings from "$lib/components/stretch-settings.svelte";
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
  let showSettings = false;
  let petType: "cat" | "panda" | "parrot" = "cat";
  let catPattern:
    | "black"
    | "siamese"
    | "orange"
    | "mackerel"
    | "white"
    | "gray" = "black";
  let catState:
    | "idle"
    | "hunt"
    | "pet"
    | "knead"
    | "overheat"
    | "scroll"
    | "stretch" = "idle";

  let prevX = 0;
  let prevY = 0;

  let huntTimeout: Timer = null;
  let wobbleTimeout: Timer = null;
  let petHoverStart: number | null = null;
  let petTimeout: Timer = null;
  let kneadTimeout: Timer = null;
  let kneadInterval: Interval = null;
  let scrollTimeout: Timer = null;
  let stretchTimeout: Timer = null;

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
  // function startDragging(e: MouseEvent) {
  //     if (e.button !== 0) {
  //         return;
  //     }
  //     e.preventDefault();
  //     isDragging = true;
  //     catState = "idle";
  //     shakeDetector.reset();
  //     appWindow
  //         .setFocus()
  //         .then(() => appWindow.startDragging())
  //         .catch((err: unknown) => {
  //             console.debug("[Comnyang] drag cancelled", err);
  //         });
  // }

  function stopDragging() {
    isDragging = false;
    shakeDetector.reset();
  }

  // --- Poll sub-functions ---
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
      catState !== "scroll" &&
      catState !== "stretch"
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
    if (speed > 80) {
      catState = "hunt";
      if (huntTimeout) {
        clearTimeout(huntTimeout);
      }
      huntTimeout = setTimeout(() => {
        if (catState === "hunt") {
          catState = "idle";
        }
      }, 350);
    } else if (catState === "hunt" && speed < 25 && !huntTimeout) {
      catState = "idle";
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
    const blockMenu = (e: MouseEvent) => {
      e.preventDefault();
    };
    window.addEventListener("contextmenu", blockMenu, { capture: true });

    // Keyboard
    const unlistenKey = listen("key-typed", () => {
      if (isDragging || catState === "scroll" || catState === "stretch") {
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

    // Scroll
    const unlistenScroll = listen<number>("scroll-event", (event) => {
      if (isDragging || catState === "stretch") {
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

    // Stretch reminder
    const unlistenStretch = listen("stretch-reminder", () => {
      catState = "stretch";
      stopKneadAnimation();
      // Auto-dismiss after 30s if user doesn't click
      if (stretchTimeout) {
        clearTimeout(stretchTimeout);
      }
      stretchTimeout = setTimeout(() => {
        catState = "idle";
      }, 30_000);
    });

    // Poll
    const interval = setInterval(() => {
      pollTick().catch((_err: unknown) => {
        // Silently ignore
      });
    }, 50);

    return () => {
      clearInterval(interval);
      stopKneadAnimation();
      unlistenKey.then((f) => f());
      unlistenScroll.then((f) => f());
      unlistenStretch.then((f) => f());
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
      if (stretchTimeout) {
        clearTimeout(stretchTimeout);
      }
    };
  });

  // --- Right click opens settings ---
  function _handleRightClick(e: MouseEvent) {
    e.preventDefault(); // prevent OS context menu
    showSettings = !showSettings;
  }

  function startDragging(e: MouseEvent) {
    if (e.button !== 0) {
      return; // ignore right click — already correct
    }
    e.preventDefault();
    isDragging = true;
    catState = "idle";
    shakeDetector.reset();
    appWindow
      .setFocus()
      .then(() => appWindow.startDragging())
      .then(() => {
        stopDragging();
      })
      .catch((err: unknown) => {
        console.debug("[Comnyang] drag cancelled", err);
        stopDragging();
      });
  }

  function handleClick() {
    if (catState === "stretch") {
      catState = "idle";
      if (stretchTimeout) {
        clearTimeout(stretchTimeout);
      }
    }
  }

  // Fix: use mousedown with button=2 instead of contextmenu
  // contextmenu fires too late after mousedown interferes
  function handleMouseDown(e: MouseEvent) {
    if (e.button === 2 || e.buttons === 2) {
      e.preventDefault();
      e.stopPropagation();
      showSettings = !showSettings;
      return;
    }
    startDragging(e);
  }

  // A11y: keyboard handler for div
  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      handleClick();
    }
    if (e.key === "Escape") {
      showSettings = false;
    }
  }
</script>

<svelte:window
  on:mouseup={stopDragging}
  on:contextmenu|preventDefault={_handleRightClick}
/>

<StretchSettings
  visible={showSettings}
  bind:petType
  bind:pattern={catPattern}
  on:close={() => (showSettings = false)}
/>

<div
  class="stage"
  on:mousedown={handleMouseDown}
  on:click={handleClick}
  on:keydown={handleKeyDown}
  on:contextmenu|preventDefault={_handleRightClick}
  role="button"
  tabindex="0"
>
  <div
    class="cat-container"
    class:bounce={!isDragging && catState === "idle"}
    class:hunt-wiggle={catState === "hunt" && !isDragging}
    class:knead-bob={catState === "knead" || catState === "overheat"}
  >
    <Cat
      {petType}
      pattern={catPattern}
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
