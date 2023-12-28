use colored::Colorize;
use std::time::Instant;
use ABC_ECS::*;

struct Position {
    x: f32,
    y: f32,
}

struct Velocity {
    x: f32,
    y: f32,
}

struct Health {
    health: f32,
}

struct PositionSystem;

impl System for PositionSystem {
    fn run(&mut self, entities_and_components: &mut EntitiesAndComponents) {
        let iter: Vec<Entity> = entities_and_components
            // not sure how to do this without collect, which drastically increases the time
            .get_entities_with_component::<Position>()
            .into_iter()
            .flatten()
            .cloned()
            .collect();

        for entity in iter {
            let entity: Entity = entity;
            let (position,) = entities_and_components.get_components_mut::<(Position,)>(entity);

            position.x += 1.0;
            position.y += 1.0;
        }
    }
}

struct RemoveEntitiesSystem;

impl System for RemoveEntitiesSystem {
    fn run(&mut self, entities_and_components: &mut EntitiesAndComponents) {
        while let Some(entity) = entities_and_components.get_nth_entity(0) {
            entities_and_components.remove_entity(entity);
        }
    }
}

fn main() {
    // test the performance with a varrying number of components

    for i in 0..500 {
        if i % 10 != 0 {
            continue;
        }

        let mut total_time_search = 0;
        let mut total_time_add = 0;
        let mut total_time_systems_run = 0;
        let mut overall_time = 0;
        // arbitrary number of iterations to get a more accurate result
        let overall_start_time = Instant::now();
        for _ in 0..1000 {
            let mut world = GameEngine::new();
            //world.add_system(Box::new(PositionSystem {}));
            world.add_system(Box::new(RemoveEntitiesSystem {}));
            {
                let entities_and_components = &mut world.entities_and_components;

                let start_time = Instant::now();
                // add entities and components
                for j in 0..i {
                    if j % 2 == 0 {
                        entities_and_components.add_entity_with((
                            Position { x: 0.0, y: 0.0 },
                            Health { health: 100.0 },
                        ));
                    } else {
                        entities_and_components.add_entity_with((Velocity { x: 0.0, y: 0.0 },));
                    }
                }
                total_time_add += start_time.elapsed().as_micros();

                /*
                let start_time = Instant::now();
                // get entities and components a constant number of times
                for _ in 0..100 {
                    for j in
                        0..entities_and_components.get_entity_count_with_component::<Position>()
                    {
                        let entity =
                            entities_and_components.get_entity_with_component::<Position>(j).expect(
                                "Failed to get entity with component Position. This should never happen.",
                            );

                        let (transform,) =
                            entities_and_components.get_components_mut::<(Position,)>(entity);
                    }
                }
                total_time_search += start_time.elapsed().as_micros();
                */

                let start_time = Instant::now();
                for _ in 0..100 {
                    world.run();
                }
                total_time_systems_run += start_time.elapsed().as_micros();
            }
        }
        overall_time = overall_start_time.elapsed().as_micros();

        print!("\n{}: ", i);
        /*for _ in 0..(total_time_search / 1000) {
            print!("|");
        }
        print!(" {}ms for {} searches \n", total_time_search, 100000);*/

        for _ in 0..(total_time_add / 10000) {
            print!("{}", "|".blue());
        }
        for _ in 0..(total_time_systems_run / 10000) {
            print!("{}", "|".green());
        }
        println!();
        for _ in 0..((overall_time) / 10000) {
            print!("{}", "|".red());
        }
        print!(
            "      {}ms for {} entities added {}ms for systems run, overall {}ms \n",
            total_time_add,
            (i * 1000),
            total_time_systems_run,
            overall_time
        );
    }
}
