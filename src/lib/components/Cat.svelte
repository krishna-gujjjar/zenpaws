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
    export let petType: "cat" | "panda" | "parrot" = "cat";
    export let pattern:
        | "black"
        | "siamese"
        | "orange"
        | "mackerel"
        | "white"
        | "gray" = "black";

    const LEFT_EYE = { x: 23, y: 24 };
    const RIGHT_EYE = { x: 41, y: 24 };
    const MAX_OFFSET = 2.0;

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

    $: catColors = (() => {
        switch (pattern) {
            case "siamese":
                return {
                    body: "#f4f1ea",
                    point: "#3d2f27",
                    belly: "#e8e2d8",
                    whisker: "#1a1a1a",
                    earInner: "#d8a096",
                    stripe: "#3d2f27",
                };
            case "orange":
                return {
                    body: "#e58332",
                    point: "#c96518",
                    belly: "#f7ba81",
                    whisker: "#ffffff",
                    earInner: "#f4a28c",
                    stripe: "#b85714",
                };
            case "mackerel":
                return {
                    body: "#7a6855",
                    point: "#383028",
                    belly: "#ffffff",
                    whisker: "#ffffff",
                    earInner: "#e8afb8",
                    stripe: "#383028",
                };
            case "white":
                return {
                    body: "#ffffff",
                    point: "#e8e8e8",
                    belly: "#f0f0f0",
                    whisker: "#333333",
                    earInner: "#f4b8ba",
                    stripe: "#e8e8e8",
                };
            case "gray":
                return {
                    body: "#6c7a89",
                    point: "#525e6b",
                    belly: "#8c9bae",
                    whisker: "#ffffff",
                    earInner: "#d1a7a7",
                    stripe: "#525e6b",
                };
            case "black":
            default:
                return {
                    body: "#181818",
                    point: "#111111",
                    belly: "#242424",
                    whisker: "#ffffff",
                    earInner: "#443333",
                    stripe: "#111111",
                };
        }
    })();

    $: activeBodyColor = getBodyColor(state, petType, catColors.body);
    $: activePointColor = getPointColor(state, petType, catColors.point);
    $: activeBellyColor = getBellyColor(state, petType, catColors.belly);
    $: pawColor = getPawColor(petType, activePointColor);
    $: isMochi = isDragging || state === "stretch";

    function getBodyColor(st: string, pet: string, cBody: string): string {
        if (st === "overheat") {
            return "#ff3333";
        }
        if (pet === "cat") {
            return cBody;
        }
        if (pet === "panda") {
            return "#ffffff";
        }
        return "#7fd158";
    }

    function getPointColor(st: string, pet: string, cPoint: string): string {
        if (st === "overheat") {
            return "#cc0000";
        }
        if (pet === "cat") {
            return cPoint;
        }
        if (pet === "panda") {
            return "#1c1c1c";
        }
        return "#3f8a2b";
    }

    function getBellyColor(st: string, pet: string, cBelly: string): string {
        if (st === "overheat") {
            return "#ff6666";
        }
        if (pet === "cat") {
            return cBelly;
        }
        if (pet === "panda") {
            return "#ffffff";
        }
        return "#95e06d";
    }

    function getPawColor(pet: string, ptColor: string): string {
        if (pet === "parrot") {
            return "#b5653b";
        }
        return ptColor;
    }
</script>

<div
    class="pet-wrapper"
    class:mochi-stretch={isMochi}
    class:wobble={isWobbling}
    class:pet-anim={state === "pet"}
    class:hunt-crouch={state === "hunt"}
    class:overheat={state === "overheat"}
    class:scroll={state === "scroll"}
    class:idle-breathe={state === "idle" && !isDragging}
>
    <svg class="pet-svg" viewBox="0 0 64 64" shape-rendering="crispEdges">
        <title>Comnyang Companion</title>

        {#if petType === "cat"}
            <!-- ==================== 🐱 PROPER RETRO PIXEL CAT FIGURES ==================== -->
            <!-- Curled Stepped Tail -->
            <g
                class="tail-group"
                class:tail-wag={state === "idle"}
                class:tail-switch={state === "hunt"}
            >
                <rect
                    x="46"
                    y="46"
                    width="6"
                    height="4"
                    fill={activePointColor}
                />
                <rect
                    x="50"
                    y="42"
                    width="4"
                    height="6"
                    fill={activePointColor}
                />
                <rect
                    x="54"
                    y="34"
                    width="4"
                    height="10"
                    fill={activePointColor}
                />
                <rect
                    x="50"
                    y="30"
                    width="6"
                    height="4"
                    fill={activePointColor}
                />
                <rect
                    x="48"
                    y="32"
                    width="4"
                    height="4"
                    fill={activePointColor}
                />
            </g>

            <!-- DISTINCT STATE FIGURES FOR CAT -->
            {#if state === "knead"}
                <!-- KNEAD FIGURE (Leaning forward typing pose matching 8.41.42 AM / kneab.png) -->
                <rect
                    x="18"
                    y="34"
                    width="28"
                    height="14"
                    fill={activeBodyColor}
                />
                <rect
                    x="20"
                    y="48"
                    width="24"
                    height="4"
                    fill={activeBodyColor}
                />
                <rect
                    x="22"
                    y="38"
                    width="20"
                    height="10"
                    fill={activeBellyColor}
                />

                <!-- Tabby Stripes on Back/Flanks -->
                {#if (pattern === "orange" || pattern === "mackerel") && state !== "overheat"}
                    <rect
                        x="18"
                        y="38"
                        width="4"
                        height="2"
                        fill={catColors.stripe}
                    />
                    <rect
                        x="42"
                        y="38"
                        width="4"
                        height="2"
                        fill={catColors.stripe}
                    />
                    <rect
                        x="18"
                        y="44"
                        width="4"
                        height="2"
                        fill={catColors.stripe}
                    />
                    <rect
                        x="42"
                        y="44"
                        width="4"
                        height="2"
                        fill={catColors.stripe}
                    />
                {/if}
            {:else if state === "hunt"}
                <!-- HUNT FIGURE (Stalking crouch pose matching hunt.png) -->
                <rect
                    x="16"
                    y="38"
                    width="32"
                    height="12"
                    fill={activeBodyColor}
                />
                <rect
                    x="18"
                    y="50"
                    width="28"
                    height="4"
                    fill={activeBodyColor}
                />
                <rect
                    x="22"
                    y="40"
                    width="20"
                    height="10"
                    fill={activeBellyColor}
                />
            {:else}
                <!-- NORMAL / IDLE / PET / OVERHEAT / SCROLL FIGURE -->
                <rect
                    x="20"
                    y="34"
                    width="24"
                    height="2"
                    fill={activeBodyColor}
                />
                <rect
                    x="18"
                    y="36"
                    width="28"
                    height="14"
                    fill={activeBodyColor}
                />
                <rect
                    x="20"
                    y="50"
                    width="24"
                    height="2"
                    fill={activeBodyColor}
                />
                <!-- Belly / White Bib contrast patch -->
                <rect
                    x="22"
                    y="38"
                    width="20"
                    height="12"
                    fill={activeBellyColor}
                />

                <!-- Tabby Stripes on Back/Flanks -->
                {#if (pattern === "orange" || pattern === "mackerel") && state !== "overheat"}
                    <rect
                        x="18"
                        y="40"
                        width="4"
                        height="2"
                        fill={catColors.stripe}
                    />
                    <rect
                        x="42"
                        y="40"
                        width="4"
                        height="2"
                        fill={catColors.stripe}
                    />
                    <rect
                        x="18"
                        y="46"
                        width="4"
                        height="2"
                        fill={catColors.stripe}
                    />
                    <rect
                        x="42"
                        y="46"
                        width="4"
                        height="2"
                        fill={catColors.stripe}
                    />
                {/if}
            {/if}

            <!-- Pointy Ears (3-step pixel stairs) -->
            <g
                class="ears-group"
                class:ears-hunt={state === "hunt"}
                class:ears-droop={state === "overheat"}
            >
                <!-- Left Ear -->
                <rect
                    x="16"
                    y="11"
                    width="8"
                    height="3"
                    fill={activePointColor}
                />
                <rect
                    x="18"
                    y="8"
                    width="4"
                    height="3"
                    fill={activePointColor}
                />
                <rect
                    x="19"
                    y="6"
                    width="2"
                    height="2"
                    fill={activePointColor}
                />
                <rect
                    x="18"
                    y="10"
                    width="4"
                    height="3"
                    fill={catColors.earInner}
                />
                <!-- Right Ear -->
                <rect
                    x="40"
                    y="11"
                    width="8"
                    height="3"
                    fill={activePointColor}
                />
                <rect
                    x="42"
                    y="8"
                    width="4"
                    height="3"
                    fill={activePointColor}
                />
                <rect
                    x="43"
                    y="6"
                    width="2"
                    height="2"
                    fill={activePointColor}
                />
                <rect
                    x="42"
                    y="10"
                    width="4"
                    height="3"
                    fill={catColors.earInner}
                />
            </g>

            <!-- Round Stepped Head -->
            <g class:head-lift={state === "pet"}>
                <rect
                    x="18"
                    y="14"
                    width="28"
                    height="1"
                    fill={activeBodyColor}
                />
                <rect
                    x="16"
                    y="15"
                    width="32"
                    height="1"
                    fill={activeBodyColor}
                />
                <rect
                    x="14"
                    y="16"
                    width="36"
                    height="18"
                    fill={activeBodyColor}
                />
                <rect
                    x="16"
                    y="34"
                    width="32"
                    height="1"
                    fill={activeBodyColor}
                />
                <rect
                    x="18"
                    y="35"
                    width="28"
                    height="1"
                    fill={activeBodyColor}
                />

                <!-- Tabby Forehead & Cheek Stripes -->
                {#if (pattern === "orange" || pattern === "mackerel") && state !== "overheat"}
                    <rect
                        x="28"
                        y="15"
                        width="2"
                        height="3"
                        fill={catColors.stripe}
                    />
                    <rect
                        x="34"
                        y="15"
                        width="2"
                        height="3"
                        fill={catColors.stripe}
                    />
                    <rect
                        x="14"
                        y="24"
                        width="3"
                        height="2"
                        fill={catColors.stripe}
                    />
                    <rect
                        x="47"
                        y="24"
                        width="3"
                        height="2"
                        fill={catColors.stripe}
                    />
                {/if}

                <!-- Siamese Face Point Mask -->
                {#if pattern === "siamese" && state !== "overheat"}
                    <rect
                        x="20"
                        y="19"
                        width="24"
                        height="15"
                        fill={catColors.point}
                    />
                    <rect
                        x="18"
                        y="21"
                        width="28"
                        height="11"
                        fill={catColors.point}
                    />
                {/if}

                <!-- White Muzzle for Mackerel Tabby matching 8.40.02 AM -->
                {#if pattern === "mackerel" && state !== "overheat"}
                    <rect x="26" y="28" width="12" height="6" fill="#ffffff" />
                {/if}

                <!-- Crisp Horizontal Whiskers -->
                <rect
                    x="5"
                    y="24"
                    width="8"
                    height="1"
                    fill={catColors.whisker}
                />
                <rect
                    x="5"
                    y="28"
                    width="8"
                    height="1"
                    fill={catColors.whisker}
                />
                <rect
                    x="51"
                    y="24"
                    width="8"
                    height="1"
                    fill={catColors.whisker}
                />
                <rect
                    x="51"
                    y="28"
                    width="8"
                    height="1"
                    fill={catColors.whisker}
                />

                <!-- Cute Nose and w-Mouth -->
                <rect x="31" y="28" width="2" height="1" fill="#ff8da1" />
                <rect
                    x="30"
                    y="30"
                    width="1"
                    height="1"
                    fill="#ffffff"
                    opacity="0.9"
                />
                <rect
                    x="31"
                    y="31"
                    width="1"
                    height="1"
                    fill="#ffffff"
                    opacity="0.9"
                />
                <rect
                    x="32"
                    y="30"
                    width="1"
                    height="1"
                    fill="#ffffff"
                    opacity="0.9"
                />
                <rect
                    x="33"
                    y="31"
                    width="1"
                    height="1"
                    fill="#ffffff"
                    opacity="0.9"
                />
                <rect
                    x="34"
                    y="30"
                    width="1"
                    height="1"
                    fill="#ffffff"
                    opacity="0.9"
                />

                <!-- Cheek Blush -->
                {#if state === "pet"}
                    <rect
                        x="16"
                        y="27"
                        width="4"
                        height="2"
                        fill="#ff8da1"
                        opacity="0.9"
                    />
                    <rect
                        x="44"
                        y="27"
                        width="4"
                        height="2"
                        fill="#ff8da1"
                        opacity="0.9"
                    />
                {/if}
            </g>
        {:else if petType === "panda"}
            <!-- ==================== 🐼 PROPER RETRO PIXEL PANDA FIGURES ==================== -->
            <!-- Round Charcoal Ears -->
            <rect x="13" y="10" width="8" height="6" fill={activePointColor} />
            <rect x="15" y="8" width="4" height="2" fill={activePointColor} />
            <rect x="43" y="10" width="8" height="6" fill={activePointColor} />
            <rect x="45" y="8" width="4" height="2" fill={activePointColor} />

            <!-- Charcoal Arms wrapping sides & White Belly -->
            <rect x="16" y="36" width="8" height="16" fill={activePointColor} />
            <rect x="40" y="36" width="8" height="16" fill={activePointColor} />
            <rect x="24" y="36" width="16" height="16" fill={activeBodyColor} />

            <!-- Round Stepped White Head -->
            <g class:head-lift={state === "pet"}>
                <rect
                    x="18"
                    y="14"
                    width="28"
                    height="1"
                    fill={activeBodyColor}
                />
                <rect
                    x="16"
                    y="15"
                    width="32"
                    height="1"
                    fill={activeBodyColor}
                />
                <rect
                    x="14"
                    y="16"
                    width="36"
                    height="18"
                    fill={activeBodyColor}
                />
                <rect
                    x="16"
                    y="34"
                    width="32"
                    height="1"
                    fill={activeBodyColor}
                />
                <rect
                    x="18"
                    y="35"
                    width="28"
                    height="1"
                    fill={activeBodyColor}
                />

                <!-- Distinctive Angled Dark Eye Patches -->
                <rect
                    x="17"
                    y="19"
                    width="10"
                    height="10"
                    fill={activePointColor}
                />
                <rect
                    x="16"
                    y="21"
                    width="12"
                    height="6"
                    fill={activePointColor}
                />
                <rect
                    x="37"
                    y="19"
                    width="10"
                    height="10"
                    fill={activePointColor}
                />
                <rect
                    x="36"
                    y="21"
                    width="12"
                    height="6"
                    fill={activePointColor}
                />

                <!-- Cute Nose & Smiling Mouth -->
                <rect x="30" y="28" width="4" height="2" fill="#111111" />
                <rect x="31" y="31" width="2" height="1" fill="#111111" />
                <rect x="29" y="32" width="2" height="1" fill="#111111" />
                <rect x="33" y="32" width="2" height="1" fill="#111111" />

                <!-- Cheek Blush -->
                {#if state === "pet"}
                    <rect
                        x="15"
                        y="27"
                        width="4"
                        height="2"
                        fill="#ff8da1"
                        opacity="0.9"
                    />
                    <rect
                        x="45"
                        y="27"
                        width="4"
                        height="2"
                        fill="#ff8da1"
                        opacity="0.9"
                    />
                {/if}
            </g>
        {:else}
            <!-- ==================== 🦜 PROPER RETRO PIXEL PARROT FIGURES ==================== -->
            <!-- Green Tail Feathers -->
            <rect x="12" y="44" width="8" height="6" fill="#3f8a2b" />
            <rect x="8" y="48" width="10" height="6" fill="#2d681e" />

            <!-- Chubby Green Torso -->
            <rect x="18" y="34" width="28" height="18" fill={activeBodyColor} />
            <rect x="20" y="38" width="24" height="14" fill="#95e06d" />

            <!-- Folded Dark Green Wings -->
            <g class="wings-group" class:wings-flap={state === "hunt"}>
                <rect
                    x="12"
                    y="34"
                    width="8"
                    height="14"
                    fill={activePointColor}
                />
                <rect
                    x="44"
                    y="34"
                    width="8"
                    height="14"
                    fill={activePointColor}
                />
            </g>

            <!-- Round Stepped Green Head -->
            <g class:head-lift={state === "pet"}>
                <rect
                    x="18"
                    y="14"
                    width="28"
                    height="1"
                    fill={activeBodyColor}
                />
                <rect
                    x="16"
                    y="15"
                    width="32"
                    height="1"
                    fill={activeBodyColor}
                />
                <rect
                    x="14"
                    y="16"
                    width="36"
                    height="18"
                    fill={activeBodyColor}
                />
                <rect
                    x="16"
                    y="34"
                    width="32"
                    height="1"
                    fill={activeBodyColor}
                />

                <!-- Orange/Peach Cheek Patches -->
                <rect x="16" y="25" width="6" height="6" fill="#f2a477" />
                <rect x="42" y="25" width="6" height="6" fill="#f2a477" />

                <!-- Big Orange/Brown Curved Beak -->
                <rect x="28" y="23" width="8" height="10" fill="#c45d3c" />
                <rect x="30" y="21" width="4" height="14" fill="#e08658" />
                <rect x="30" y="35" width="4" height="2" fill="#9e472b" />
            </g>
        {/if}

        <!-- ==================== SHARED EYES & EXPRESSIONS LAYER ==================== -->
        <g class:head-lift={state === "pet"}>
            {#if state === "pet" || state === "stretch"}
                <!-- Happy squinty closed eyes ( ^_^ ) matching 8.41.17 AM -->
                <rect x="20" y="24" width="1" height="1" fill="#ffffff" />
                <rect x="21" y="23" width="4" height="1" fill="#ffffff" />
                <rect x="25" y="24" width="1" height="1" fill="#ffffff" />

                <rect x="38" y="24" width="1" height="1" fill="#ffffff" />
                <rect x="39" y="23" width="4" height="1" fill="#ffffff" />
                <rect x="43" y="24" width="1" height="1" fill="#ffffff" />
            {:else if state === "overheat"}
                <!-- Overheat swirly/dizzy eyes -->
                <rect x="20" y="21" width="6" height="6" fill="#ffffff" />
                <rect x="38" y="21" width="6" height="6" fill="#ffffff" />
                <rect x="22" y="23" width="2" height="2" fill="#ff0000" />
                <rect x="40" y="23" width="2" height="2" fill="#ff0000" />
            {:else}
                <!-- Normal pixel white eye backgrounds -->
                <rect x="20" y="21" width="6" height="6" fill="#ffffff" />
                <rect x="38" y="21" width="6" height="6" fill="#ffffff" />

                <!-- Moving Pupils that track cursor (3x3 pixel squares) -->
                <rect
                    x={Math.round(20 + 1.5 + pupilOffsetX * MAX_OFFSET)}
                    y={Math.round(21 + 1.5 + pupilOffsetY * MAX_OFFSET)}
                    width={state === "hunt" ? "4" : "3"}
                    height={state === "hunt" ? "4" : "3"}
                    fill="#000000"
                />
                <rect
                    x={Math.round(38 + 1.5 + pupilOffsetX * MAX_OFFSET)}
                    y={Math.round(21 + 1.5 + pupilOffsetY * MAX_OFFSET)}
                    width={state === "hunt" ? "4" : "3"}
                    height={state === "hunt" ? "4" : "3"}
                    fill="#000000"
                />

                <!-- Angry hunt eyebrows -->
                {#if state === "hunt"}
                    <rect
                        x="19"
                        y="19"
                        width="8"
                        height="2"
                        fill={activePointColor}
                        transform="rotate(15, 23, 20)"
                    />
                    <rect
                        x="37"
                        y="19"
                        width="8"
                        height="2"
                        fill={activePointColor}
                        transform="rotate(-15, 41, 20)"
                    />
                {/if}
            {/if}
        </g>

        <!-- ==================== SHARED PAWS / FEET LAYER ==================== -->
        {#if state === "knead"}
            <!-- Kneading 3D Keyboard Keys matching 8.41.42 AM / kneab.png -->
            <rect x="16" y="56" width="14" height="6" fill="#777777" />
            <rect x="17" y="56" width="12" height="4" fill="#dddddd" />
            <rect x="34" y="56" width="14" height="6" fill="#777777" />
            <rect x="35" y="56" width="12" height="4" fill="#dddddd" />

            <!-- Alternating Paw taps reaching over keyboard keys -->
            <rect
                x="19"
                y={kneadFrame === 0 ? 50 : 53}
                width="8"
                height="7"
                rx="2"
                fill={pawColor}
            />
            {#if petType === "panda"}
                <rect
                    x="21"
                    y={kneadFrame === 0 ? 52 : 55}
                    width="4"
                    height="2"
                    fill="#ff8da1"
                />
            {/if}
            <rect
                x="37"
                y={kneadFrame === 1 ? 50 : 53}
                width="8"
                height="7"
                rx="2"
                fill={pawColor}
            />
            {#if petType === "panda"}
                <rect
                    x="39"
                    y={kneadFrame === 1 ? 52 : 55}
                    width="4"
                    height="2"
                    fill="#ff8da1"
                />
            {/if}
        {:else if state === "scroll"}
            <!-- Scroll Paper Roll -->
            <rect
                x="22"
                y="50"
                width="20"
                height={Math.min(paperLength, 12)}
                fill="#f5f0e8"
            />
            {#if paperLength > 3}<rect
                    x="24"
                    y="53"
                    width="12"
                    height="1"
                    fill="#cccccc"
                />{/if}
            {#if paperLength > 6}<rect
                    x="24"
                    y="56"
                    width="10"
                    height="1"
                    fill="#cccccc"
                />{/if}
            <!-- Paws rolling paper -->
            <rect x="19" y="48" width="8" height="6" rx="2" fill={pawColor} />
            <rect x="37" y="48" width="8" height="6" rx="2" fill={pawColor} />
        {:else}
            <!-- Idle Paws / Feet -->
            {#if petType === "panda"}
                <!-- Panda Big Black Feet with Toe Beans & Pink Pad -->
                <rect
                    x="18"
                    y="50"
                    width="10"
                    height="7"
                    fill={activePointColor}
                />
                <rect x="19" y="51" width="2" height="2" fill="#ff8da1" />
                <rect x="22" y="51" width="2" height="2" fill="#ff8da1" />
                <rect x="25" y="51" width="2" height="2" fill="#ff8da1" />
                <rect x="20" y="54" width="6" height="2" fill="#ff8da1" />

                <rect
                    x="36"
                    y="50"
                    width="10"
                    height="7"
                    fill={activePointColor}
                />
                <rect x="37" y="51" width="2" height="2" fill="#ff8da1" />
                <rect x="40" y="51" width="2" height="2" fill="#ff8da1" />
                <rect x="43" y="51" width="2" height="2" fill="#ff8da1" />
            {:else}
                <!-- Cat Paws / Parrot Claws -->
                <rect x="20" y="50" width="8" height="6" fill={pawColor} />
                <rect x="36" y="50" width="8" height="6" fill={pawColor} />
            {/if}
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
    .pet-wrapper {
        position: relative;
        width: 100%;
        height: 100%;
    }

    .pet-svg {
        width: 100%;
        height: 100%;
        display: block;
        user-select: none;
        -webkit-user-drag: none;
    }

    /* Authentic Elastic Mochi Stretch (Planted at bottom feet 50% 100%) */
    .mochi-stretch .pet-svg {
        transform-origin: 50% 100%;
        animation: mochi-elastic-stretch 0.9s infinite alternate
            cubic-bezier(0.45, 0.05, 0.55, 0.95);
    }

    @keyframes mochi-elastic-stretch {
        0% {
            transform: scaleX(1) scaleY(1);
        }
        35% {
            transform: scaleX(0.82) scaleY(1.42);
        }
        70% {
            transform: scaleX(0.88) scaleY(1.3);
        }
        100% {
            transform: scaleX(0.75) scaleY(1.55);
        }
    }

    /* Realistic Breathing Bob in Idle */
    .idle-breathe .pet-svg {
        animation: idle-breathing 2.8s infinite ease-in-out;
    }
    @keyframes idle-breathing {
        0%,
        100% {
            transform: translateY(0) scaleY(1);
        }
        50% {
            transform: translateY(-1.5px) scaleY(1.02);
        }
    }

    /* Hunt Crouch */
    .hunt-crouch .pet-svg {
        transform-origin: 50% 100%;
        transform: scaleY(0.93) translateY(2px);
    }

    /* Head Lift in Pet Mode matching 8.41.17 AM */
    .head-lift {
        transform: translateY(-2.5px);
        transition: transform 0.2s ease-out;
    }

    .wobble {
        animation: wobble-shake 0.15s infinite alternate ease-in-out;
    }
    .pet-anim .pet-svg {
        animation: pet-happy 0.4s infinite alternate ease-in-out;
    }
    .scroll .pet-svg {
        animation: scroll-lean 0.6s infinite alternate ease-in-out;
    }
    .overheat .pet-svg {
        filter: drop-shadow(0 0 6px rgba(255, 50, 50, 0.8));
    }

    /* Tail & Wings animations */
    .tail-wag {
        transform-origin: 46px 44px;
        animation: tail-sway 2.2s infinite ease-in-out;
    }
    .tail-switch {
        transform-origin: 46px 44px;
        animation: tail-switching 0.25s infinite alternate ease-in-out;
    }
    .wings-flap {
        transform-origin: 32px 36px;
        animation: wings-flapping 0.18s infinite alternate ease-in-out;
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
    @keyframes tail-switching {
        0% {
            transform: rotate(-8deg);
        }
        100% {
            transform: rotate(18deg);
        }
    }
    @keyframes wings-flapping {
        0% {
            transform: scaleX(1);
        }
        100% {
            transform: scaleX(1.18) translateY(-3px);
        }
    }
    @keyframes pet-happy {
        0% {
            transform: translateY(0);
        }
        100% {
            transform: translateY(-3px);
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
