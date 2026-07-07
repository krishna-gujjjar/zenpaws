<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher } from "svelte";

  export let visible = false;
  export let petType: "cat" | "panda" | "parrot" = "cat";
  export let pattern:
    | "black"
    | "siamese"
    | "orange"
    | "mackerel"
    | "white"
    | "gray" = "black";

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
  <div class="settings-panel" role="dialog" aria-label="Settings">
    <button type="button" class="close-btn" on:click={close}>✕</button>

    <!-- Companion Selector -->
    <div class="sec-label">Choose Companion:</div>
    <div class="pet-row">
      <button
        type="button"
        class="pet-btn"
        class:sel={petType === "cat"}
        on:click={() => (petType = "cat")}
      >
        🐱 Cat
      </button>
      <button
        type="button"
        class="pet-btn"
        class:sel={petType === "panda"}
        on:click={() => (petType = "panda")}
      >
        🐼 Panda
      </button>
      <button
        type="button"
        class="pet-btn"
        class:sel={petType === "parrot"}
        on:click={() => (petType = "parrot")}
      >
        🦜 Parrot
      </button>
    </div>

    <!-- Cat Pattern Selector matching Screenshot 8.40.02 AM -->
    {#if petType === "cat"}
      <div class="sec-label" style="margin-top: 4px;">Cat Style:</div>
      <div class="color-row">
        <button
          type="button"
          class="col-btn c-black"
          class:c-sel={pattern === "black"}
          on:click={() => (pattern = "black")}
          title="Black"
        ></button>
        <button
          type="button"
          class="col-btn c-siamese"
          class:c-sel={pattern === "siamese"}
          on:click={() => (pattern = "siamese")}
          title="Siamese"
        ></button>
        <button
          type="button"
          class="col-btn c-orange"
          class:c-sel={pattern === "orange"}
          on:click={() => (pattern = "orange")}
          title="Orange Tabby"
        ></button>
        <button
          type="button"
          class="col-btn c-mackerel"
          class:c-sel={pattern === "mackerel"}
          on:click={() => (pattern = "mackerel")}
          title="Mackerel Tabby"
        ></button>
        <button
          type="button"
          class="col-btn c-white"
          class:c-sel={pattern === "white"}
          on:click={() => (pattern = "white")}
          title="White"
        ></button>
        <button
          type="button"
          class="col-btn c-gray"
          class:c-sel={pattern === "gray"}
          on:click={() => (pattern = "gray")}
          title="Gray"
        ></button>
      </div>
    {/if}

    <!-- Stretch Reminder Timer -->
    <div class="sec-label" style="margin-top: 6px;">Stretch Reminder:</div>
    <div class="row">
      <input
        id="interval"
        type="number"
        min="1"
        max="120"
        bind:value={intervalMins}
        disabled={active}
      >
      <span>min</span>
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
    top: 2px;
    right: 2px;
    bottom: 2px;
    left: 2px;
    z-index: 999;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 8px;
    font-family: monospace;
    font-size: 10px;
    color: #fff;
    background: rgba(20, 20, 30, 0.96);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
  }

  .close-btn {
    position: absolute;
    top: 4px;
    right: 6px;
    padding: 2px 4px;
    font-size: 11px;
    color: #aaa;
    cursor: pointer;
    background: none;
    border: none;
  }
  .close-btn:hover {
    color: #fff;
  }

  .sec-label {
    margin-bottom: 3px;
    font-size: 9px;
    font-weight: bold;
    color: #ccc;
  }

  .pet-row {
    display: flex;
    gap: 4px;
    margin-bottom: 4px;
  }

  .pet-btn {
    flex: 1;
    padding: 3px 0;
    font-size: 11px;
    color: #fff;
    text-align: center;
    cursor: pointer;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }
  .pet-btn.sel {
    font-weight: bold;
    background: rgba(76, 175, 130, 0.4);
    border-color: #4caf82;
    transform: scale(1.03);
  }

  .color-row {
    display: flex;
    gap: 5px;
    justify-content: center;
    margin-bottom: 4px;
  }
  .col-btn {
    width: 16px;
    height: 16px;
    padding: 0;
    cursor: pointer;
    border: 1.5px solid rgba(255, 255, 255, 0.3);
    border-radius: 50%;
  }
  .col-btn.c-sel {
    border-color: #fff;
    box-shadow: 0 0 0 1.5px #4caf82;
    transform: scale(1.15);
  }
  .c-black {
    background: #181818;
  }
  .c-siamese {
    background: #f4f1ea;
    border-color: #3d2f27;
  }
  .c-orange {
    background: #e58332;
    border-color: #c96518;
  }
  .c-mackerel {
    background: #7a6855;
    border-color: #383028;
  }
  .c-white {
    background: #ffffff;
  }
  .c-gray {
    background: #6c7a89;
  }

  .row {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  input[type="number"] {
    width: 36px;
    padding: 2px;
    font-size: 10px;
    color: #fff;
    text-align: center;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }

  .btn {
    flex: 1;
    padding: 3px 6px;
    font-size: 10px;
    font-weight: bold;
    cursor: pointer;
    border: none;
    border-radius: 4px;
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
    margin-top: 3px;
    font-size: 9px;
    color: #aaffcc;
    text-align: center;
  }
</style>
