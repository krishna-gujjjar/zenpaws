<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import Cat from "$lib/components/cat.svelte";

  const appWindow = getCurrentWindow();

  let isDragging = false;
  let pupilOffsetX = 0;
  let pupilOffsetY = 0;
  let catState: "idle" | "hunt" = "idle";
  let prevX = 0;
  let prevY = 0;
  let huntTimeout: any = null;

  function startDragging(e: MouseEvent) {
    if (e.button !== 0) {
      return;
    }

    // CRITICAL: prevent text selection
    e.preventDefault();
    e.stopPropagation();

    isDragging = true;

    // CRITICAL: no await — must be called synchronously
    // so the OS can capture the mousedown event timing
    appWindow.startDragging().catch(() => {});
  }

  function stopDragging() {
    isDragging = false;
  }

  onMount(() => {
    const interval = setInterval(async () => {
      try {
        const [cursorX, cursorY] = await invoke<[number, number]>(
          "get_cursor_position"
        );
        const winPos = await appWindow.outerPosition();
        const winSize = await appWindow.outerSize();

        const centerX = winPos.x + winSize.width / 2;
        const centerY = winPos.y + winSize.height / 2;
        const dx = cursorX - centerX;
        const dy = cursorY - centerY;
        const dist = Math.sqrt(dx * dx + dy * dy);
        const ratio = Math.min(dist / 150, 1);
        const angle = Math.atan2(dy, dx);

        pupilOffsetX = Math.cos(angle) * ratio * 0.5;
        pupilOffsetY = Math.sin(angle) * ratio * 0.5;

        // Hunt detection
        const speed = Math.sqrt(
          (cursorX - prevX) ** 2 + (cursorY - prevY) ** 2
        );
        if (speed > 60) {
          catState = "hunt";
          if (huntTimeout) {
            clearTimeout(huntTimeout);
          }
          huntTimeout = setTimeout(() => (catState = "idle"), 2000);
        }

        prevX = cursorX;
        prevY = cursorY;
      } catch (_) {}
    }, 50);

    return () => {
      clearInterval(interval);
      if (huntTimeout) {
        clearTimeout(huntTimeout);
      }
    };
  });
</script>

<svelte:window on:mouseup={stopDragging} />

<div class="stage" on:mousedown={startDragging} role="presentation">
  <div class="cat-container" class:bounce={!isDragging && catState === "idle"}>
    <Cat {isDragging} {pupilOffsetX} {pupilOffsetY} state={catState} />
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
    animation: idle-bounce 2s infinite ease-in-out;
  }

  @keyframes idle-bounce {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(4px);
    }
  }
</style>
