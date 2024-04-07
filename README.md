# Performance
This ECS was built with usability in mind. That being said it is only a pseudo-ECS. Component-based multi-threading has not yet been implemented. I am not sure it ever will be, but I am interested in an entity chunk-based multithreading approach.
# ABC Game Engine - Simple ECS Framework
This Rust project provides a basic framework for managing game entities, components, and systems in the ABC Game Engine using an Entity Component System (ECS) approach.

## Quick Example
Create a Game Engine:

```rust
fn main(){
    let mut world = World::new();

    let entities_and_components = &mut world.entities_and_components;
    // Add an Entity and Components:
    let entity = entities_and_components.add_entity();

    // Add components like Position and Velocity
    entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
    entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

    // or you can do it in one step:
    let entity = entities_and_components
        .add_entity_with((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));

    // Add your system to the engine
    engine.add_system(MovementSystem {});

    // Run the engine in a loop
    // would want to run this in a loop in a real game
    for _ in 0..5 {
        engine.run();
    }
}
```

## Components and Systems
The example includes simple components like Position and Velocity, along with a MovementSystem that updates positions based on velocities. Customize these components and systems according to your game's needs.

## Testing
Explore the included test module to see how entities, components, systems, and the game engine are used together. Use this as a starting point for writing your own tests.

Feel free to tweak and expand the ECS framework to fit your game development requirements within the ABC Game Engine!
