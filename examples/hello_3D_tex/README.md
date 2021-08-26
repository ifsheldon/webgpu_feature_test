# Hello 3D Texture
A very simple example testing the capability of 3D texture of an WebGPU implementation.

Press `w` to play around with color transition.

A 3D texture with the resolution of (2,2,2) is colored with voxels like
```rust
let tex_data: &[u8] = &[
            0, 0, 0, 255, // RGBA in unsigned 8-bit integer
            0, 0, 255, 255,
            0, 255, 0, 255,
            0, 255, 255, 255,
            255, 0, 0, 255,
            255, 0, 255, 255,
            255, 255, 0, 255,
            255, 255, 255, 255,
        ];
```

The display will show a slice of it on UV plain, and pressing `w` will move the slice along `w` axis of the volume texture.

## Run
To run this example locally, enter `cargo run --example hello_3D_tex`.