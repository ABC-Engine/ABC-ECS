# ABC Game Engine - Simple ECS Framework
This Rust project provides a basic framework for managing game entities, components, and systems in the ABC Game Engine using an Entity Component System (ECS) approach.

## Quick Start
Create a Game Engine:

```rust
let mut engine = GameEngine::new();
```

Add an Entity and Components:

```rust
let entities_and_components = &mut engine.entities_and_components;
let entity = entities_and_components.add_entity();
```

Add components like Position and Velocity

```rust
entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });
```
Define a System:

```rust
struct MovementSystem {}

impl System for MovementSystem {
    fn run(&self, engine: &mut EntitiesAndComponents) {
        // Logic to update positions based on velocities
    }
}
```

Run the Engine:

```rust
// Add your system to the engine
engine.add_system(Box::new(MovementSystem {}));

// Run the engine in a loop
loop {
    engine.run();
}
```

## Components and Systems
The example includes simple components like Position and Velocity, along with a MovementSystem that updates positions based on velocities. Customize these components and systems according to your game's needs.

## Testing
Explore the included test module to see how entities, components, systems, and the game engine are used together. Use this as a starting point for writing your own tests.

Feel free to tweak and expand the ECS framework to fit your game development requirements within the ABC Game Engine!
