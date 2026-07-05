<script lang="ts">
  export let pupilOffsetX = 0;
  export let pupilOffsetY = 0;
  export let state: "idle" | "hunt" | "pet" = "idle";
  export let isDragging = false;
  export let isWobbling = false;

  const LEFT_EYE = { x: 23.5, y: 25.5 };
  const RIGHT_EYE = { x: 39.5, y: 25.5 };
  const MAX_OFFSET = 1.2;

  // Heart particles
  let hearts: { id: number; x: number }[] = [];
  let heartId = 0;

  $: if (state === "pet") {
    spawnHeart();
  }

  function spawnHeart() {
    const id = heartId++;
    const x = 20 + Math.random() * 24; // SVG x range over head
    hearts = [...hearts, { id, x }];
    setTimeout(() => {
      hearts = hearts.filter((h) => h.id !== id);
    }, 1000);
  }
</script>

<div
  class="panda-wrapper"
  class:dragging={isDragging && !isWobbling}
  class:wobble={isWobbling}
  class:pet={state === "pet"}
>
  <img src="/panda-idle.svg" alt="panda" class="panda-base" draggable="false">

  <svg class="eye-overlay" viewBox="0 0 64 64">
    <!-- IDLE / HUNT: normal pupils -->
    {#if state !== "pet"}
      <rect
        x={LEFT_EYE.x + pupilOffsetX * MAX_OFFSET - 0.5}
        y={LEFT_EYE.y + pupilOffsetY * MAX_OFFSET - 0.5}
        width="1.2"
        height="1.2"
        fill="#0a0a1a"
        opacity="0.9"
      />
      <rect
        x={RIGHT_EYE.x + pupilOffsetX * MAX_OFFSET - 0.5}
        y={RIGHT_EYE.y + pupilOffsetY * MAX_OFFSET - 0.5}
        width="1.2"
        height="1.2"
        fill="#0a0a1a"
        opacity="0.9"
      />
    {/if}

    <!-- HUNT: angry brows -->
    {#if state === "hunt"}
      <rect
        x="20"
        y="20"
        width="6"
        height="1"
        fill="#1a1a1a"
        opacity="0.85"
        transform="rotate(10, 23, 20)"
      />
      <rect
        x="38"
        y="20"
        width="6"
        height="1"
        fill="#1a1a1a"
        opacity="0.85"
        transform="rotate(-10, 41, 20)"
      />
    {/if}

    <!-- PET: squint eyes (happy ~) -->
    {#if state === "pet"}
      <!-- Left squint ~  -->
      <path
        d="M22 26 Q23.5 24.5 25 26"
        stroke="#0a0a1a"
        stroke-width="1"
        fill="none"
        opacity="0.9"
      />
      <!-- Right squint ~ -->
      <path
        d="M38 26 Q39.5 24.5 41 26"
        stroke="#0a0a1a"
        stroke-width="1"
        fill="none"
        opacity="0.9"
      />
      <!-- Left blush -->
      <rect
        x="19"
        y="28"
        width="4"
        height="2"
        rx="1"
        fill="#ff9999"
        opacity="0.5"
      />
      <!-- Right blush -->
      <rect
        x="40"
        y="28"
        width="4"
        height="2"
        rx="1"
        fill="#ff9999"
        opacity="0.5"
      />

      <!-- Floating hearts -->
      {#each hearts as heart (heart.id)}
        <text
          x={heart.x}
          y="18"
          font-size="4"
          fill="#ff6b9d"
          opacity="0.9"
          class="heart-float"
        >
          ♥
        </text>
      {/each}
    {/if}
  </svg>
</div>

<style>
  .panda-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
    transition: transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .dragging {
    transform: scale(0.88, 1.2);
  }

  .wobble {
    animation: wobble 0.5s ease-in-out;
  }

  /* Gentle happy bounce when petted */
  .pet {
    animation: pet-happy 0.4s infinite alternate ease-in-out;
  }

  .panda-base {
    width: 100%;
    height: 100%;
    image-rendering: pixelated;
    display: block;
    -webkit-user-drag: none;
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

  .heart-float {
    animation: float-up 1s ease-out forwards;
  }

  @keyframes float-up {
    0% {
      transform: translateY(0px);
      opacity: 1;
    }
    100% {
      transform: translateY(-12px);
      opacity: 0;
    }
  }

  @keyframes pet-happy {
    0% {
      transform: scale(1) rotate(-1deg);
    }
    100% {
      transform: scale(1.04) rotate(1deg);
    }
  }

  @keyframes wobble {
    0% {
      transform: scale(0.88, 1.2) rotate(0deg);
    }
    15% {
      transform: scale(1.1, 0.9) rotate(-6deg);
    }
    30% {
      transform: scale(0.9, 1.1) rotate(6deg);
    }
    45% {
      transform: scale(1.05, 0.95) rotate(-4deg);
    }
    60% {
      transform: scale(0.95, 1.05) rotate(4deg);
    }
    80% {
      transform: scale(1.02, 0.98) rotate(-2deg);
    }
    100% {
      transform: scale(0.88, 1.2) rotate(0deg);
    }
  }
</style>
