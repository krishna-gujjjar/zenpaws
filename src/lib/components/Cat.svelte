<script lang="ts">
  export let pupilOffsetX = 0;
  export let pupilOffsetY = 0;
  export let state: "idle" | "hunt" | "pet" | "knead" | "overheat" | "scroll" =
    "idle";
  export let isDragging = false;
  export let isWobbling = false;
  export let kneadFrame = 0;
  export let paperLength = 0; // 0-20 SVG units

  const LEFT_EYE = { x: 23.5, y: 25.5 };
  const RIGHT_EYE = { x: 39.5, y: 25.5 };
  const MAX_OFFSET = 1.2;

  let hearts: { id: number; x: number }[] = [];
  let heartId = 0;
  let lastPetState = false;

  $: {
    if (state === "pet" && !lastPetState) {
      spawnHeart();
    }
    lastPetState = state === "pet";
  }

  function spawnHeart() {
    const id = heartId++;
    const x = 20 + Math.random() * 24;
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
  class:overheat={state === "overheat"}
  class:scroll={state === "scroll"}
>
  <img src="/panda-idle.svg" alt="panda" class="panda-base" draggable="false">

  <svg class="eye-overlay" viewBox="0 0 64 64">
    <title>Eyes</title>
    <!-- Normal pupils -->
    {#if state !== "pet" && state !== "overheat"}
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

    <!-- Hunt: angry brows -->
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

    <!-- Pet: squint + blush + hearts -->
    {#if state === "pet"}
      <path
        d="M22 26 Q23.5 24.5 25 26"
        stroke="#0a0a1a"
        stroke-width="1"
        fill="none"
        opacity="0.9"
      />
      <path
        d="M38 26 Q39.5 24.5 41 26"
        stroke="#0a0a1a"
        stroke-width="1"
        fill="none"
        opacity="0.9"
      />
      <rect
        x="19"
        y="28"
        width="4"
        height="2"
        rx="1"
        fill="#ff9999"
        opacity="0.5"
      />
      <rect
        x="40"
        y="28"
        width="4"
        height="2"
        rx="1"
        fill="#ff9999"
        opacity="0.5"
      />
      {#each hearts as heart (heart.id)}
        <text
          x={heart.x}
          y="18"
          font-size="4"
          fill="#ff6b9d"
          class="heart-float"
        >
          ♥
        </text>
      {/each}
    {/if}

    <!-- Knead: paw tap overlay -->
    {#if state === "knead"}
      <rect
        x="18"
        y={kneadFrame === 0 ? 47 : 49}
        width="6"
        height="4"
        rx="1"
        fill="#2a2a2a"
        opacity="0.7"
      />
      <rect
        x="40"
        y={kneadFrame === 1 ? 47 : 49}
        width="6"
        height="4"
        rx="1"
        fill="#2a2a2a"
        opacity="0.7"
      />
      <rect x="14" y="52" width="36" height="1" fill="#888" opacity="0.4" />
    {/if}

    <!-- Overheat: wide eyes + steam -->
    {#if state === "overheat"}
      <rect x="21" y="24" width="3" height="3" fill="#ffffff" opacity="0.9" />
      <rect x="39" y="24" width="3" height="3" fill="#ffffff" opacity="0.9" />
      <rect x="22" y="25" width="1" height="1" fill="#0a0a1a" opacity="1" />
      <rect x="40" y="25" width="1" height="1" fill="#0a0a1a" opacity="1" />
      <text
        x="26"
        y="14"
        font-size="5"
        fill="#aaaaaa"
        opacity="0.8"
        class="steam-1"
      >
        ~
      </text>
      <text
        x="32"
        y="10"
        font-size="6"
        fill="#bbbbbb"
        opacity="0.7"
        class="steam-2"
      >
        ~
      </text>
      <text
        x="38"
        y="13"
        font-size="5"
        fill="#aaaaaa"
        opacity="0.8"
        class="steam-3"
      >
        ~
      </text>
    {/if}

    <!-- Scroll: paper roll unrolling below paws -->
    {#if state === "scroll"}
      <!-- Left paw holding paper -->
      <rect
        x="16"
        y="46"
        width="6"
        height="4"
        rx="1"
        fill="#2a2a2a"
        opacity="0.8"
      />
      <!-- Right paw holding paper -->
      <rect
        x="42"
        y="46"
        width="6"
        height="4"
        rx="1"
        fill="#2a2a2a"
        opacity="0.8"
      />

      <!-- Paper roll body (grows with paperLength) -->
      <rect
        x="22"
        y="50"
        width="20"
        height={Math.min(paperLength, 12)}
        rx="1"
        fill="#f5f0e8"
        opacity="0.95"
      />

      <!-- Paper lines (content on paper) -->
      {#if paperLength > 3}
        <rect x="24" y="53" width="12" height="0.8" fill="#ccc" opacity="0.6" />
      {/if}
      {#if paperLength > 6}
        <rect x="24" y="55" width="10" height="0.8" fill="#ccc" opacity="0.6" />
      {/if}
      {#if paperLength > 9}
        <rect x="24" y="57" width="14" height="0.8" fill="#ccc" opacity="0.6" />
      {/if}

      <!-- Paper roll cylinder at bottom -->
      <ellipse
        cx="32"
        cy={50 + Math.min(paperLength, 12)}
        rx="10"
        ry="2"
        fill="#e8e0d0"
        opacity="0.9"
      />
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

  .dragging {
    transform: scale(0.88, 1.2);
  }

  .pet {
    animation: pet-happy 0.4s infinite alternate ease-in-out;
  }

  .scroll {
    animation: scroll-lean 0.6s infinite alternate ease-in-out;
  }

  .overheat .panda-base {
    filter: sepia(0.3) saturate(2) hue-rotate(-10deg) brightness(1.1);
  }

  .wobble {
    animation: wobble 0.5s ease-in-out;
  }

  .heart-float {
    animation: float-up 1s ease-out forwards;
  }
  .steam-1 {
    animation: steam-rise 0.8s infinite ease-out;
  }
  .steam-2 {
    animation: steam-rise 0.8s 0.2s infinite ease-out;
  }
  .steam-3 {
    animation: steam-rise 0.8s 0.4s infinite ease-out;
  }

  @keyframes float-up {
    0% {
      transform: translateY(0);
      opacity: 1;
    }
    100% {
      transform: translateY(-12px);
      opacity: 0;
    }
  }

  @keyframes steam-rise {
    0% {
      transform: translateY(0) scaleX(1);
      opacity: 0.8;
    }
    100% {
      transform: translateY(-6px) scaleX(1.3);
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

  @keyframes scroll-lean {
    0% {
      transform: rotate(-1deg) translateY(0px);
    }
    100% {
      transform: rotate(1deg) translateY(1px);
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
