# TODO:
- [*] Properly implement cameras
    * OpenXR: matrix mulitplication for prefix
    * Winit: prefix passthrough
- [*] Index buffers (static, dynamic)
- [*] Transforms
- [ ] Actually write correct barriers for uploads...
- [ ] Image textures
    * Samplers
- [ ] MSAA
- [ ] Tracking shaders (feature, requires shaderc and notify)
- [ ] Instance buffers (static, dynamic)
- [ ] Able to write junk data from CPU buffer into GPU by overflow/underflow?
- [ ] Turn on sync validation; emit proper memory barries for copies
- [ ] Setting application name in settings should actually set the window title from FPS!
- [ ] Seperate view/projection matrices, resolution in UBO
- [ ] Simple GPU-driven rendering
- [ ] Point size and line size; useful for making circles