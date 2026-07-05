<script lang="ts">
  export let pupilOffsetX = 0;
  export let pupilOffsetY = 0;
  export let state: "idle" | "hunt" = "idle";
  export let isDragging = false; // New prop

  const LEFT_EYE = { x: 24, y: 26 };
  const RIGHT_EYE = { x: 40, y: 26 };
  const MAX_OFFSET = 1;
</script>

<div
  class="panda-wrapper"
  class:hunt={state === "hunt"}
  class:dragging={isDragging}
>
  <img src="/panda-idle.svg" alt="panda" class="panda-base" draggable="false">

  <svg class="eye-overlay" viewBox="0 0 64 64">
    <title>Panda Eyes</title>
    <rect
      x={LEFT_EYE.x + pupilOffsetX * MAX_OFFSET - 0.5}
      y={LEFT_EYE.y + pupilOffsetY * MAX_OFFSET - 0.5}
      width="1"
      height="1"
      fill="#1a1a2e"
      opacity="0.85"
    />
    <rect
      x={RIGHT_EYE.x + pupilOffsetX * MAX_OFFSET - 0.5}
      y={RIGHT_EYE.y + pupilOffsetY * MAX_OFFSET - 0.5}
      width="1"
      height="1"
      fill="#1a1a2e"
      opacity="0.85"
    />
    {#if state === "hunt"}
      <rect x="21" y="21" width="5" height="1" fill="#1a1a1a" opacity="0.7" />
      <rect x="38" y="21" width="5" height="1" fill="#1a1a1a" opacity="0.7" />
    {/if}
  </svg>
</div>

<style>
  .panda-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
    transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  /* The "Mochi" Stretch */
  .dragging {
    transform: scale(0.85, 1.25) translateY(-10px);
  }

  .panda-base {
    display: block;
    width: 100%;
    height: 100%;
    user-select: none;
    image-rendering: pixelated;
  }

  .eye-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    shape-rendering: crispEdges;
  }
</style>
