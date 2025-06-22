# Vislum To-do

## 2025-06-25

- [x] Implement generic Container.
- [x] Implement base node and operator system.
- [x] Implement basic operator evaluation.
  - Sample "Unknown" and "Cycle" error. Keep it simple for now.
  - Brainstorm ways of decouping the interpreter "eval" from everything.
    - Generics likely won't work.
    - TypeRegistry and Box<dyn Any> would be way more complex than
      I want it to be for now.

## 2025-07-28
- [ ] Implement basic render abstractions.
  - [ ] Decide on Handles or Arcs.
      Handles provide centralized management, but this is mostly unecessarry
      as we don't plan any complex dependency management. However, these can
      simplify a bunch of code down the line, especially the rule that all
      engine values should be default constructable. This means we could have
      a Handle::None to represent things like empty textures. Either way,
      I feel this might be a bad abstraction, and it would be best to simply
      keep all inputs as required, and optionally provide a default value at
      node-input level and value level. A bit more complex, but drastically
      simplifies this, especially when we implement more complex effects that
      depend on complex data. Suppose we have a "StringFormat" input, then
      assuming the format would be an empty string would be an error.
  - [ ] Mesh. MeshManager.
  - [ ] Texture. TextureManager.
  - [ ] Material. MaterialManager.
  - [ ] RenderPipelineManager.
      - [ ] Basic rendering tailored to Material rendering.
- [ ] Resource injection in renderer.


- [ ] Ready to begin implementing the editor.
  - For the editor I want it to be really smart. Like: really.
  - For now, I can start simple:
    - [ ] NodeView with positional information.
    - [ ] Simple calculus.
      - [ ] Viewport that can understand our return-types.
      - [ ] Dynamic root node.