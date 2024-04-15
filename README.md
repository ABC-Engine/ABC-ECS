# Performance
This ECS was built with usability in mind. That being said it is only a pseudo-ECS. As needed you can use either the run or single_entity_step and pre-step to utilize the multithreading we offer. As needed you can switch between these two methods. This ECS can be really performant if you need it to be.

# ABC Game Engine - Simple ECS Framework
This Rust project provides a basic framework for managing game entities, components, and systems in the ABC Game Engine using an Entity Component System (ECS) approach.

# Quick Example
Create a World:

```rust
use ABC_ECS::World;

struct Position {
    x: f32,
    y: f32,
}

struct Velocity {
    x: f32,
    y: f32,
}

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
}
```

# Documentation
Visit the docs [here](https://github.com/ABC-Engine/ABC-ECS/wiki). 

# Contributing
Contributions are welcome! Start by filing an issue and we can work forward from there! If you're not sure what to work on but you want to help [Join the discord and ping me](https://discord.gg/6nTvhYRfpm), I'm happy to help!
