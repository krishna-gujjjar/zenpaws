<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onDestroy, onMount } from "svelte";
    import Cat from "$lib/components/cat.svelte";

    const appWindow = getCurrentWindow();
    let isDragging = false;
    let pupilOffsetX = 0;
    let pupilOffsetY = 0;

    function clamp(val: number, min: number, max: number) {
        return Math.min(Math.max(val, min), max);
    }

    let interval: ReturnType<typeof setInterval>;

    onMount(() => {
        interval = setInterval(async () => {
            try {
                const [cursorX, cursorY] = await invoke<[number, number]>(
                    "get_cursor_position",
                );
                const winPos = await appWindow.outerPosition();
                const winSize = await appWindow.outerSize();

                const centerX = winPos.x + winSize.width / 2;
                const centerY = winPos.y + winSize.height / 2;

                const dx = cursorX - centerX;
                const dy = cursorY - centerY;

                const dist = Math.sqrt(dx * dx + dy * dy);
                const maxDist = 150;
                const ratio = Math.min(dist / maxDist, 1);
                const angle = Math.atan2(dy, dx);

                pupilOffsetX = clamp(Math.cos(angle) * ratio * 0.5, -0.5, 0.5);
                pupilOffsetY = clamp(Math.sin(angle) * ratio * 0.5, -0.5, 0.5);
            } catch (_) {}
        }, 50);
    });

    onDestroy(() => clearInterval(interval));

    async function startDragging() {
        isDragging = true;
        await appWindow.startDragging();
    }
</script>

<svelte:window on:mouseup={() => (isDragging = false)} />

<div class="stage" on:mousedown={startDragging} role="presentation">
    <div class="cat-container" class:bounce={!isDragging}>
        <Cat bodyColor="#636e72" {pupilOffsetX} {pupilOffsetY} />
    </div>
</div>

<style>
    .stage {
        align-items: center;
        cursor: grab;
        display: flex;
        height: 160px;
        justify-content: center;
        user-select: none;
        width: 160px;
    }

    .cat-container {
        width: 120px;
        height: 120px;
    }

    .bounce {
        animation: idle-bounce 2s infinite ease-in-out;
    }

    @keyframes idle-bounce {
        0%,
        100% {
            transform: translateY(0) scale(1);
        }
        50% {
            transform: translateY(2px) scale(1.02, 0.98);
        }
    }
</style>
