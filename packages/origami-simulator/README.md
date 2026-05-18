# @treemaker/origami-simulator

Standalone FOLD-in origami simulation library for browser applications.

The package boundary is intentionally clean:

- no React dependency
- no Tauri or file-system behavior
- no TreeMaker model dependency
- optional Three.js renderer via `@treemaker/origami-simulator/three`

The dynamic simulation structure is adapted from Amanda Ghassaei's
[Origami Simulator](https://github.com/amandaghassaei/OrigamiSimulator), with
the app/UI layer removed and the public API reshaped into a reusable module.
The current solver uses a deterministic CPU beam/crease force step shaped after
the original dynamic solver metadata; WebGL helpers and the Three.js renderer
are isolated so the GPU solver can be deepened without changing app code.

License: MIT.
