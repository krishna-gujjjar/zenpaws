<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { createEventDispatcher } from "svelte";

    export let visible = false;

    const dispatch = createEventDispatcher<{ close: undefined }>();

    let intervalMins = 30;
    let active = false;

    async function startTimer() {
        await invoke("start_stretch_timer", { intervalMins });
        active = true;
    }

    async function stopTimer() {
        await invoke("stop_stretch_timer");
        active = false;
    }

    function close() {
        dispatch("close");
    }
</script>

{#if visible}
    <div class="settings-panel">
        <button type="button" class="close-btn" on:click={close}>✕</button>
        <div class="title">🐾 Stretch Reminder</div>

        <div class="row">
            <label for="interval">Every</label>
            <input
                id="interval"
                type="number"
                min="1"
                max="120"
                bind:value={intervalMins}
                disabled={active}
            />
            <span>min</span>
        </div>

        <div class="row">
            {#if active}
                <button type="button" class="btn stop" on:click={stopTimer}>
                    Stop
                </button>
            {:else}
                <button type="button" class="btn start" on:click={startTimer}>
                    Start
                </button>
            {/if}
        </div>

        {#if active}
            <div class="status">⏱ Reminding every {intervalMins}m</div>
        {/if}
    </div>
{/if}

<style>
    .settings-panel {
        position: absolute;
        top: 4px;
        left: 4px;
        right: 4px;
        bottom: 4px;
        z-index: 999;
        padding: 10px;
        display: flex;
        flex-direction: column;
        justify-content: center;
        box-sizing: border-box;
        font-family: monospace;
        font-size: 11px;
        color: #fff;
        background: rgba(20, 20, 30, 0.95);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 10px;
        backdrop-filter: blur(8px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    }

    .title {
        margin-bottom: 8px;
        font-size: 12px;
        font-weight: bold;
        text-align: center;
    }

    .close-btn {
        position: absolute;
        top: 6px;
        right: 8px;
        padding: 0;
        font-size: 10px;
        color: #aaa;
        cursor: pointer;
        background: none;
        border: none;
    }

    .row {
        display: flex;
        gap: 6px;
        align-items: center;
        margin-bottom: 6px;
    }

    input[type="number"] {
        width: 48px;
        padding: 2px 4px;
        font-size: 11px;
        color: #fff;
        text-align: center;
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 4px;
    }

    .btn {
        flex: 1;
        padding: 4px 8px;
        font-size: 11px;
        font-weight: bold;
        cursor: pointer;
        border: none;
        border-radius: 5px;
    }

    .start {
        color: #fff;
        background: #4caf82;
    }

    .stop {
        color: #fff;
        background: #e05c5c;
    }

    .status {
        margin-top: 4px;
        font-size: 10px;
        color: #aaffcc;
        text-align: center;
    }
</style>
