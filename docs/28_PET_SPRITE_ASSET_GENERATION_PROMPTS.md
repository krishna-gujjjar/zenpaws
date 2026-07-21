# 28 - Pet Sprite Asset Generation Prompts

Companion to `10_PET_ENGINE_SPECIFICATION.md` and `23_ASSET_LICENSES.md`.

This is a working prompt kit for producing the sprite sheets Phase 4 needs. It is asset guidance, not application code.

## Read this before generating anything

General-purpose image generators are not well suited to independent animation frames: proportions, palette, pose style, and alignment drift between generations. Use a sprite-sheet consistency tool that locks a character reference and generates animation states against it, such as PixelLab or a comparable current tool. Check current pricing, terms of service, and commercial-use terms before committing to a tool.

Recommended workflow:

1. Generate one character reference sheet per pet.
2. Use that locked reference as image input for every state animation.
3. Touch up palette, frame alignment, and loop seams in Aseprite, Piskel, or the sprite tool editor.
4. Record provenance and commercial-use terms in `23_ASSET_LICENSES.md` before shipping.

If a dedicated sprite tool is unavailable, use the character reference as image-to-image input for every subsequent generation. Never generate animation frames text-only.

## Global style bible

Paste this verbatim into every prompt:

```text
Style: cute chibi 2D game sprite, flat cel-shading, thin 1px dark outline,
soft rounded shapes, big expressive eyes, minimal shading with a single
top-left light source. NOT photorealistic, NOT painterly, NOT 3D-rendered.

Side-view profile facing right, full body visible, centered in frame.
Background: fully transparent alpha channel.
Canvas: 128x128px per frame. Character is drawn on an internal 32x32 pixel
grid then upscaled with nearest-neighbor. Crisp pixel edges, no blur,
anti-aliasing artifacts, or smooth gradients.
Palette: warm, friendly, moderately saturated and consistent across all pets.
```

## Character reference prompts

Generate one reference before any state frames. These are proposals, not locked designs.

### Cat

```text
[GLOBAL STYLE BIBLE]
Subject: small round chibi cat, orange tabby stripes, cream belly, short
triangular ears, long loosely curled tail, bright green eyes, closed-mouth
content expression, sitting neutral pose facing right.
```

### Dog

```text
[GLOBAL STYLE BIBLE]
Subject: small chibi shiba-inu-style dog, cream and tan fur, one ear perked
and one slightly folded, curled tail over back, dark round eyes, open friendly
smile, sitting neutral pose facing right.
```

### Fox

```text
[GLOBAL STYLE BIBLE]
Subject: small chibi fox, bright orange fur, white chest and tail tip, large
pointed ears, bushy tail, amber eyes, sly but friendly closed-mouth smile,
standing neutral pose facing right.
```

### Rabbit

```text
[GLOBAL STYLE BIBLE]
Subject: small chibi rabbit, soft grey-white fur, one long ear standing and
one flopped, round fluffy tail, pink nose, large dark eyes, sitting neutral
pose facing right.
```

### Slime

```text
[GLOBAL STYLE BIBLE]
Subject: small translucent teal-green gel blob, glossy upper-left highlight,
two simple black dot eyes, no mouth or a tiny curved smile, rounded teardrop
body, resting neutral pose with no legs.
```

### Ghost

```text
[GLOBAL STYLE BIBLE]
Subject: small pastel lavender-white ghost, rounded head, wavy scalloped
bottom edge, simple round dot eyes, small open o mouth or gentle smile,
slight forward-lean floating neutral pose facing right.
```

### Parrot

```text
[GLOBAL STYLE BIBLE]
Subject: small chibi parrot, vivid red-blue-green macaw-style plumage, short
curved beak, round black eyes, perched neutral pose on both feet facing right,
wings folded at sides.
```

## Per-state animation prompt template

Use the locked reference as an image input.

```text
[GLOBAL STYLE BIBLE]
Using the attached character reference, generate a looping animation sequence
for the "<STATE>" action: <STATE DESCRIPTION>.
Frame count: <N>. Keep proportions, palette, outline weight, and silhouette
identical to the reference in every frame. Only the pose changes.
```

## State animation guide

| State | Frames | FPS | Motion |
|---|---:|---:|---|
| `idle` | 4 | 6 | Subtle breathing or bob, occasional blink, loop. |
| `walk` | 6 | 10 | Side-view walk loop. Parrot uses a hop cycle. |
| `run` | 6 | 14 | Faster exaggerated walk. Parrot uses wing-flutter glide. |
| `jump` | 5 | 10 | Crouch, launch, apex, falling, land; non-looping. |
| `sleep` | 3 | 2 | Curled up, slow rise/fall, optional alternating Zzz; loop. |
| `typing` | 4 | 8 | Tiny alternating paw or wing taps; loop. |
| `notification` | 3 | 8 | Head or ear perk alert; non-looping. |
| `celebrate` | 6 | 12 | Joyful bounce or spin, wings or arms up. |
| `fight` | 5 | 10 | Playful non-violent cartoon mock-battle only. |
| `play` | 5 | 10 | Batting or chasing an implied toy; loop. |
| `away` | 3 | 3 | Slow distracted look-around, slightly desaturated; loop. |
| `offline` | 1 | - | Static greyscale or desaturated pose. |

## Export and naming convention

```text
assets/pets/<pet>/
  manifest.json
  sprites/
    idle.png
    walk.png
    run.png
    jump.png
    sleep.png
    typing.png
    notification.png
    celebrate.png
    fight.png
    play.png
    away.png
    offline.png
  sounds/
  LICENSE.txt
```

Export one transparent PNG per state as a horizontal sprite strip: N frames, each 128px wide and 128px tall, ordered frame 1 through N left to right. The `frameCount` in `manifest.json` controls slicing.

## Before any pet ships

Fill in the pet row in `23_ASSET_LICENSES.md`, including tool commercial-use terms and any manual Aseprite or Piskel edits.
