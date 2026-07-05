<script lang="ts">
  export let bodyColor = "#636e72";
  export let eyeColor = "#2c3e50";
  export let pupilOffsetX = 0; // -0.5 to 0.5
  export let pupilOffsetY = 0; // -0.5 to 0.5
</script>

<svg viewBox="0 0 32 32" class="pixel-cat">
  <title>Pixel Cat</title>
  <!-- Tail -->
  <rect x="26" y="20" width="4" height="4" fill={bodyColor} />
  <rect x="24" y="22" width="4" height="4" fill={bodyColor} />

  <!-- Body -->
  <rect x="6" y="14" width="20" height="12" fill={bodyColor} />

  <!-- Head -->
  <rect x="16" y="6" width="12" height="10" fill={bodyColor} />

  <!-- Ears -->
  <rect x="16" y="4" width="2" height="2" fill={bodyColor} />
  <rect x="26" y="4" width="2" height="2" fill={bodyColor} />

  <!-- Eye sockets (whites) -->
  <rect x="18" y="10" width="2" height="2" fill="#ecf0f1" />
  <rect x="24" y="10" width="2" height="2" fill="#ecf0f1" />

  <!-- Pupils (follow mouse) -->
  <rect
    x={18.5 + pupilOffsetX}
    y={10.5 + pupilOffsetY}
    width="1"
    height="1"
    fill={eyeColor}
  />
  <rect
    x={24.5 + pupilOffsetX}
    y={10.5 + pupilOffsetY}
    width="1"
    height="1"
    fill={eyeColor}
  />

  <!-- Nose -->
  <rect x="22" y="12" width="2" height="1" fill="#e74c3c" />
</svg>

<style>
  .pixel-cat {
    width: 100%;
    height: 100%;
    shape-rendering: crispEdges;
  }
</style>
