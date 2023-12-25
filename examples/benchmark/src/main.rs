use rand::Rng;
use std::time::Instant;
use ABC_ECS::*;

struct Position {
    x: f32,
    y: f32,
}

impl Component for Position {}

struct Velocity {
    x: f32,
    y: f32,
}

impl Component for Velocity {}

struct Health {
    health: f32,
}

impl Component for Health {}

fn main() {
    // test the performance with a varrying number of components

    for i in 0..100 {
        let start_time = Instant::now();
        // arbitrary number of iterations to get a more accurate result
        for i in 0..1000 {
            if i % 10 != 0 {
                continue;
            }
            let mut world = GameEngine::new();
            {
                let entities_and_components = &mut world.entities_and_components;

                // add entities and components
                for j in 0..i {
                    let entity = entities_and_components.add_entity();

                    if j % 2 == 0 {
                        entities_and_components
                            .add_component_to(entity, Position { x: 0.0, y: 0.0 });
                        entities_and_components.add_component_to(entity, Health { health: 100.0 });
                    } else {
                        entities_and_components
                            .add_component_to(entity, Velocity { x: 0.0, y: 0.0 });
                    }
                }

                // get entities and components
                for _ in 0..100 {
                    for j in
                        0..entities_and_components.get_entity_count_with_component::<Position>()
                    {
                        let entity =
                            entities_and_components.get_nth_entity_with_component::<Position>(j).expect(
                                "Failed to get entity with component Position. This should never happen.",
                            );

                        let (transform,) =
                            entities_and_components.get_components_mut::<(Position,)>(entity);
                    }
                }
            }
        }
        print!("\n{}: ", i);
        for i in 0..(start_time.elapsed().as_millis() / 5) {
            print!("|");
        }
    }
}
