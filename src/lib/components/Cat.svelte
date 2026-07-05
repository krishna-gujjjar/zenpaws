<script lang="ts">
    export let pupilOffsetX = 0;
    export let pupilOffsetY = 0;
    export let state:
        | "idle"
        | "hunt"
        | "pet"
        | "knead"
        | "overheat"
        | "scroll"
        | "stretch" = "idle";
    export let isDragging = false;
    export let isWobbling = false;
    export let kneadFrame = 0;
    export let paperLength = 0;
    export let pattern: "black" | "siamese" | "orange" | "white" | "gray" =
        "black";

    const LEFT_EYE = { x: 24, y: 25 };
    const RIGHT_EYE = { x: 40, y: 25 };
    const MAX_OFFSET = 1.8;

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

    $: colors = (() => {
        switch (pattern) {
            case "siamese":
                return {
                    body: "#f2efe9",
                    point: "#3d2f27",
                    belly: "#e6e0d4",
                    whisker: "#1a1a1a",
                    earInner: "#d8a096",
                };
            case "orange":
                return {
                    body: "#e58332",
                    point: "#c96518",
                    belly: "#f7ba81",
                    whisker: "#ffffff",
                    earInner: "#f4a28c",
                };
            case "white":
                return {
                    body: "#ffffff",
                    point: "#e8e8e8",
                    belly: "#f0f0f0",
                    whisker: "#333333",
                    earInner: "#f4b8ba",
                };
            case "gray":
                return {
                    body: "#6c7a89",
                    point: "#525e6b",
                    belly: "#8c9bae",
                    whisker: "#ffffff",
                    earInner: "#d1a7a7",
                };
            case "black":
            default:
                return {
                    body: "#1a1a1a",
                    point: "#111111",
                    belly: "#262626",
                    whisker: "#ffffff",
                    earInner: "#443333",
                };
        }
    })();

    $: activeBodyColor = state === "overheat" ? "#ff3333" : colors.body;
    $: activePointColor = state === "overheat" ? "#cc0000" : colors.point;
</script>

<div
    class="cat-wrapper"
    class:mochi-stretch={isDragging || state === "stretch"}
    class:wobble={isWobbling}
    class:pet={state === "pet"}
    class:overheat={state === "overheat"}
    class:scroll={state === "scroll"}
>
    <svg class="cat-svg" viewBox="0 0 64 64" shape-rendering="crispEdges">
        <title>Comnyang Pixel Cat</title>

        <!-- Tail (stepped pixel tail, wagging) -->
        <g
            class="tail-group"
            class:tail-wag={state === "idle" || state === "hunt"}
        >
            <rect x="46" y="44" width="4" height="4" fill={activePointColor} />
            <rect x="50" y="40" width="4" height="6" fill={activePointColor} />
            <rect x="52" y="34" width="4" height="8" fill={activePointColor} />
            <rect x="48" y="30" width="6" height="4" fill={activePointColor} />
            <rect x="46" y="32" width="4" height="4" fill={activePointColor} />
        </g>

        <!-- Body / Torso (Authentic Stepped Pixel Art) -->
        <rect x="18" y="32" width="28" height="20" fill={activeBodyColor} />
        <rect x="16" y="36" width="32" height="14" fill={activeBodyColor} />
        <!-- Belly patch -->
        <rect
            x="22"
            y="36"
            width="20"
            height="14"
            fill={state === "overheat" ? "#ff6666" : colors.belly}
        />

        <!-- Ears (Stepped pixel pyramids) -->
        <g
            class="ears-group"
            class:ears-hunt={state === "hunt"}
            class:ears-droop={state === "overheat"}
        >
            <!-- Left Ear -->
            <rect x="18" y="15" width="6" height="3" fill={activePointColor} />
            <rect x="19" y="12" width="4" height="3" fill={activePointColor} />
            <rect x="20" y="9" width="2" height="3" fill={activePointColor} />
            <rect x="20" y="14" width="2" height="3" fill={colors.earInner} />
            <!-- Right Ear -->
            <rect x="40" y="15" width="6" height="3" fill={activePointColor} />
            <rect x="41" y="12" width="4" height="3" fill={activePointColor} />
            <rect x="42" y="9" width="2" height="3" fill={activePointColor} />
            <rect x="42" y="14" width="2" height="3" fill={colors.earInner} />
        </g>

        <!-- Head (Stepped Pixel Box) -->
        <rect x="16" y="16" width="32" height="20" fill={activeBodyColor} />
        <rect x="14" y="18" width="36" height="16" fill={activeBodyColor} />

        <!-- Siamese Face Point Mask -->
        {#if pattern === "siamese" && state !== "overheat"}
            <rect x="20" y="20" width="24" height="15" fill={colors.point} />
            <rect x="18" y="22" width="28" height="11" fill={colors.point} />
        {/if}

        <!-- Whiskers (Crisp 1px Horizontal Lines) -->
        <!-- Left Whiskers -->
        <rect x="6" y="25" width="8" height="1" fill={colors.whisker} />
        <rect x="6" y="29" width="8" height="1" fill={colors.whisker} />
        <!-- Right Whiskers -->
        <rect x="50" y="25" width="8" height="1" fill={colors.whisker} />
        <rect x="50" y="29" width="8" height="1" fill={colors.whisker} />

        <!-- Nose and Mouth -->
        <rect x="31" y="29" width="2" height="1" fill="#ff8da1" />
        <rect
            x="30"
            y="31"
            width="1"
            height="1"
            fill={pattern === "siamese" || pattern === "black"
                ? "#ffffff"
                : "#4a3b32"}
            opacity="0.8"
        />
        <rect
            x="31"
            y="32"
            width="1"
            height="1"
            fill={pattern === "siamese" || pattern === "black"
                ? "#ffffff"
                : "#4a3b32"}
            opacity="0.8"
        />
        <rect
            x="32"
            y="31"
            width="1"
            height="1"
            fill={pattern === "siamese" || pattern === "black"
                ? "#ffffff"
                : "#4a3b32"}
            opacity="0.8"
        />
        <rect
            x="33"
            y="32"
            width="1"
            height="1"
            fill={pattern === "siamese" || pattern === "black"
                ? "#ffffff"
                : "#4a3b32"}
            opacity="0.8"
        />
        <rect
            x="34"
            y="31"
            width="1"
            height="1"
            fill={pattern === "siamese" || pattern === "black"
                ? "#ffffff"
                : "#4a3b32"}
            opacity="0.8"
        />

        <!-- Cheek Blush when petted -->
        {#if state === "pet"}
            <rect
                x="17"
                y="28"
                width="4"
                height="2"
                fill="#ff8da1"
                opacity="0.8"
            />
            <rect
                x="43"
                y="28"
                width="4"
                height="2"
                fill="#ff8da1"
                opacity="0.8"
            />
        {/if}

        <!-- Eyes Layer -->
        {#if state === "pet" || state === "stretch"}
            <!-- Happy squinty closed eyes (stepped pixel arches ^_^) -->
            <rect x="21" y="26" width="1" height="1" fill="#ffffff" />
            <rect x="22" y="25" width="2" height="1" fill="#ffffff" />
            <rect x="24" y="26" width="1" height="1" fill="#ffffff" />

            <rect x="39" y="26" width="1" height="1" fill="#ffffff" />
            <rect x="40" y="25" width="2" height="1" fill="#ffffff" />
            <rect x="42" y="26" width="1" height="1" fill="#ffffff" />
        {:else if state === "overheat"}
            <!-- Overheat swirly/dizzy eyes -->
            <rect x="21" y="23" width="6" height="6" fill="#ffffff" />
            <rect x="37" y="23" width="6" height="6" fill="#ffffff" />
            <rect x="23" y="25" width="2" height="2" fill="#ff0000" />
            <rect x="39" y="25" width="2" height="2" fill="#ff0000" />
        {:else}
            <!-- Normal pixel white eye boxes -->
            <rect x="21" y="22" width="6" height="6" fill="#ffffff" />
            <rect x="37" y="22" width="6" height="6" fill="#ffffff" />

            <!-- Moving Pupils (2x2 pixel blocks) -->
            <rect
                x={Math.round(LEFT_EYE.x - 1 + pupilOffsetX * MAX_OFFSET)}
                y={Math.round(LEFT_EYE.y - 1 + pupilOffsetY * MAX_OFFSET)}
                width="2"
                height="2"
                fill="#000000"
            />
            <rect
                x={Math.round(RIGHT_EYE.x - 1 + pupilOffsetX * MAX_OFFSET)}
                y={Math.round(RIGHT_EYE.y - 1 + pupilOffsetY * MAX_OFFSET)}
                width="2"
                height="2"
                fill="#000000"
            />

            <!-- Angry hunt eyebrows -->
            {#if state === "hunt"}
                <rect
                    x="20"
                    y="20"
                    width="8"
                    height="2"
                    fill={activePointColor}
                    transform="rotate(15, 24, 21)"
                />
                <rect
                    x="36"
                    y="20"
                    width="8"
                    height="2"
                    fill={activePointColor}
                    transform="rotate(-15, 40, 21)"
                />
            {/if}
        {/if}

        <!-- Paws / Legs (Pixel Blocks) -->
        {#if state === "knead"}
            <!-- Kneading Keyboard Keys -->
            <rect x="17" y="54" width="12" height="6" fill="#888888" />
            <rect x="18" y="54" width="10" height="4" fill="#cccccc" />
            <rect x="35" y="54" width="12" height="6" fill="#888888" />
            <rect x="36" y="54" width="10" height="4" fill="#cccccc" />

            <!-- Alternating Paw taps -->
            <rect
                x="19"
                y={kneadFrame === 0 ? 48 : 51}
                width="8"
                height="6"
                fill={activePointColor}
            />
            <rect
                x="37"
                y={kneadFrame === 1 ? 48 : 51}
                width="8"
                height="6"
                fill={activePointColor}
            />
        {:else if state === "scroll"}
            <!-- Scroll Paper Roll -->
            <rect
                x="22"
                y="50"
                width="20"
                height={Math.min(paperLength, 12)}
                fill="#f5f0e8"
            />
            {#if paperLength > 3}
                <rect x="24" y="53" width="12" height="1" fill="#cccccc" />
            {/if}
            {#if paperLength > 6}
                <rect x="24" y="56" width="10" height="1" fill="#cccccc" />
            {/if}
            <!-- Paws rolling paper -->
            <rect x="19" y="47" width="8" height="6" fill={activePointColor} />
            <rect x="37" y="47" width="8" height="6" fill={activePointColor} />
        {:else}
            <!-- Idle Paws -->
            <rect x="20" y="50" width="8" height="6" fill={activePointColor} />
            <rect x="36" y="50" width="8" height="6" fill={activePointColor} />
        {/if}

        <!-- Overheat Steam Particles -->
        {#if state === "overheat"}
            <text x="24" y="10" font-size="7" fill="#ffffff" class="steam-1"
                >~</text
            >
            <text x="32" y="6" font-size="8" fill="#ffcccc" class="steam-2"
                >~</text
            >
            <text x="40" y="9" font-size="7" fill="#ffffff" class="steam-3"
                >~</text
            >
        {/if}

        <!-- Petting Floating Hearts -->
        {#if state === "pet"}
            {#each hearts as heart (heart.id)}
                <text
                    x={heart.x}
                    y="12"
                    font-size="7"
                    fill="#ff4d6d"
                    class="heart-float">♥</text
                >
            {/each}
        {/if}
    </svg>
</div>

<style>
    .cat-wrapper {
        position: relative;
        width: 100%;
        height: 100%;
        transition: transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1);
    }

    .cat-svg {
        width: 100%;
        height: 100%;
        display: block;
        user-select: none;
        -webkit-user-drag: none;
    }

    /* Exaggerated Mochi Squash-and-Stretch */
    .mochi-stretch {
        transform-origin: 32px 56px;
        animation: mochi-elastic 0.7s infinite alternate
            cubic-bezier(0.45, 0.05, 0.55, 0.95);
    }

    @keyframes mochi-elastic {
        0% {
            transform: scaleY(1) scaleX(1);
        }
        35% {
            transform: scaleY(1.45) scaleX(0.75) translateY(-5px);
        }
        70% {
            transform: scaleY(1.3) scaleX(0.85) translateY(-2px);
        }
        100% {
            transform: scaleY(1.55) scaleX(0.7) translateY(-9px);
        }
    }

    .wobble {
        animation: wobble-shake 0.15s infinite alternate ease-in-out;
    }

    .pet {
        animation: pet-happy 0.4s infinite alternate ease-in-out;
    }

    .scroll {
        animation: scroll-lean 0.6s infinite alternate ease-in-out;
    }

    .overheat .cat-svg {
        filter: drop-shadow(0 0 6px rgba(255, 50, 50, 0.8));
    }

    /* Tail animations */
    .tail-wag {
        transform-origin: 46px 44px;
        animation: tail-sway 2.2s infinite ease-in-out;
    }

    /* Ears animations */
    .ears-hunt {
        transform-origin: 32px 18px;
        transform: translateY(2px) scaleY(0.8);
    }
    .ears-droop {
        transform-origin: 32px 18px;
        transform: translateY(4px) scaleY(0.6);
    }

    @keyframes tail-sway {
        0%,
        100% {
            transform: rotate(0deg);
        }
        50% {
            transform: rotate(14deg);
        }
    }

    @keyframes pet-happy {
        0% {
            transform: scale(1) translateY(0);
        }
        100% {
            transform: scale(1.05) translateY(-3px);
        }
    }

    @keyframes scroll-lean {
        0% {
            transform: rotate(-3deg);
        }
        100% {
            transform: rotate(3deg);
        }
    }

    @keyframes wobble-shake {
        0% {
            transform: translateX(-4px) rotate(-5deg);
        }
        100% {
            transform: translateX(4px) rotate(5deg);
        }
    }

    .heart-float {
        animation: float-up 1s ease-out forwards;
    }

    @keyframes float-up {
        0% {
            transform: translateY(0) scale(0.8);
            opacity: 1;
        }
        100% {
            transform: translateY(-16px) scale(1.3);
            opacity: 0;
        }
    }

    .steam-1 {
        animation: steam-rise 1.2s infinite ease-out;
    }
    .steam-2 {
        animation: steam-rise 1.5s infinite 0.3s ease-out;
    }
    .steam-3 {
        animation: steam-rise 1.3s infinite 0.6s ease-out;
    }

    @keyframes steam-rise {
        0% {
            transform: translateY(0);
            opacity: 0.8;
        }
        100% {
            transform: translateY(-12px);
            opacity: 0;
        }
    }
</style>
