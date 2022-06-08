# Platform

## Rendering
We provide a number of rendering methods to support animation.
The resolution of the screen is 160x128, and the game is 12x9 tiles. A tile sprite is therefore
12x12 (144x108), leaving (16x20) margin. Each tile is therefore 288 bytes.
The walking sprite sheet is arranged as rows with a fixed number of tiles.
<
>
^
v
-> If 4 frames, 16 * 288 = 4608 bytes

We then have special animations, assuming 8 frames
Death
Rescue
This is 2304 bytes.


TBD: Load a spritesheet using !include_bytes
