<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import Cat from "$lib/components/cat.svelte";
  import { isCursorOnHead } from "$lib/utils/pet-detector";
  import { ShakeDetector } from "$lib/utils/shake-detector";

  const appWindow = getCurrentWindow();
  const shakeDetector = new ShakeDetector();

  let isDragging = false;
  let isWobbling = false;
  let pupilOffsetX = 0;
  let pupilOffsetY = 0;
  let catState: "idle" | "hunt" | "pet" = "idle";

  let prevX = 0;
  let prevY = 0;
  let huntTimeout: any = null;
  let wobbleTimeout: any = null;
  let petHoverStart: number | null = null;
  let petTimeout: any = null;

  const PET_HOVER_DELAY = 300; // ms hover before pet triggers
  const PET_RESET_DELAY = 1200;

  function triggerWobble() {
    isWobbling = false;
    setTimeout(() => {
      isWobbling = true;
      if (wobbleTimeout) {
        clearTimeout(wobbleTimeout);
      }
      wobbleTimeout = setTimeout(() => {
        isWobbling = false;
      }, 500);
    }, 10);
  }

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
      .catch(() => {});
  }

  function stopDragging() {
    isDragging = false;
    shakeDetector.reset();
  }

  onMount(() => {
    const interval = setInterval(async () => {
      try {
        const [cursorX, cursorY] = await invoke<[number, number]>(
          "get_cursor_position"
        );
        const winPos = await appWindow.outerPosition();
        const winSize = await appWindow.outerSize();

        // Eye follow
        const centerX = winPos.x + winSize.width / 2;
        const centerY = winPos.y + winSize.height / 2;
        const dx = cursorX - centerX;
        const dy = cursorY - centerY;
        const dist = Math.sqrt(dx * dx + dy * dy);
        const ratio = Math.min(dist / 150, 1);
        const angle = Math.atan2(dy, dx);
        pupilOffsetX = Math.cos(angle) * ratio * 0.5;
        pupilOffsetY = Math.sin(angle) * ratio * 0.5;

        // Speed
        const speed = Math.sqrt(
          (cursorX - prevX) ** 2 + (cursorY - prevY) ** 2
        );

        if (!isDragging) {
          // Pet detection
          const onHead = isCursorOnHead(
            cursorX,
            cursorY,
            winPos.x,
            winPos.y,
            winSize.width,
            winSize.height
          );

          if (onHead && catState !== "hunt") {
            if (petHoverStart === null) {
              petHoverStart = Date.now();
            } else if (Date.now() - petHoverStart >= PET_HOVER_DELAY) {
              catState = "pet";
              if (petTimeout) {
                clearTimeout(petTimeout);
              }
              petTimeout = setTimeout(() => {
                catState = "idle";
                petHoverStart = null;
              }, PET_RESET_DELAY);
            }
          } else {
            petHoverStart = null;
          }

          // Hunt detection — only if not petting
          if (catState !== "pet" && speed > 60) {
            catState = "hunt";
            if (huntTimeout) {
              clearTimeout(huntTimeout);
            }
            huntTimeout = setTimeout(() => (catState = "idle"), 2000);
          }
        }

        // Shake detection — only while dragging
        if (isDragging) {
          const [winX] = await invoke<[number, number]>("get_window_position");
          const shook = shakeDetector.update(winX);
          if (shook) {
            triggerWobble();
          }
        }

        prevX = cursorX;
        prevY = cursorY;
      } catch {}
    }, 50);

    return () => {
      clearInterval(interval);
      if (huntTimeout) {
        clearTimeout(huntTimeout);
      }
      if (wobbleTimeout) {
        clearTimeout(wobbleTimeout);
      }
      if (petTimeout) {
        clearTimeout(petTimeout);
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
  >
    <Cat
      {isDragging}
      {isWobbling}
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
</style>
