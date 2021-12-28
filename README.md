# TODO:
- [x] Properly implement cameras
    * OpenXR: matrix mulitplication for prefix
    * Winit: prefix passthrough
- [x] Index buffers (static, dynamic)
- [x] Transforms
- [x] Actually write correct barriers for uploads...
- [x] MSAA
- [ ] Image textures (dynamic, and with sampling modes)
- [ ] Blending settings for shaders
- [ ] `egui`
- [ ] Tracking shaders (feature, requires shaderc and notify)
- [ ] Instance buffers (static, dynamic)
- [ ] Test if we are able to write junk data from CPU buffer into GPU by overflow/underflow?
- [ ] Setting application name in settings should actually set the window title, replacing FPS!
- [ ] Seperate view/projection matrices, resolution in UBO
- [ ] Switch to GPU-driven rendering if possible
- [ ] Point size and line size; useful for making circles
- [ ] OIT?
- [ ] Crate for text. Use SDF?
- [ ] Auto-resizing transforms buffer
- [ ] WGPU backend
- [ ] Sort draws by category, for speed?
- [ ] OpenXR controller display example
- [ ] Cut down on dependencies
