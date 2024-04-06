use bevy_ecs::prelude::Component;
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

#[derive(Component)]
struct BevyPosition {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct BevyVelocity {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct BevyHealth {
    health: f32,
}

fn bevy_position_system(mut query: bevy_ecs::prelude::Query<(&mut BevyPosition,)>) {
    for (mut position,) in query.iter_mut() {
        std::thread::sleep(std::time::Duration::from_nanos(10));
        position.x *= factorial(position.x.powf(2.5));
        position.y *= factorial(position.y.powf(2.5));
    }
}

struct PositionSystem;

impl System for PositionSystem {
    fn single_entity_step(&self, single_entity: &mut SingleMutEntity) {
        if let Some(position) = single_entity.try_get_component_mut::<Position>() {
            std::thread::sleep(std::time::Duration::from_nanos(10));
            position.x *= factorial(position.x.powf(2.5));
            position.y *= factorial(position.y.powf(2.5));
        }
    }
    fn implements_single_entity_step(&self) -> bool {
        true
    }
}

fn factorial(n: f32) -> f32 {
    if n == 0.0 {
        return 1.0;
    }
    return n * factorial(n - 1.0);
}

struct SingleThreadedPositionSystem;

impl System for SingleThreadedPositionSystem {
    fn run(&mut self, entities_and_components: &mut EntitiesAndComponents) {
        for i in 0..entities_and_components.get_entity_count_with_component::<Position>() {
            let entity = entities_and_components
                .get_entity_with_component::<Position>(i)
                .expect("Failed to get entity with component Position. This should never happen.");

            let (position,) = entities_and_components.get_components_mut::<(Position,)>(entity);

            std::thread::sleep(std::time::Duration::from_nanos(10));
            position.x *= factorial(position.x.powf(2.5));
            position.y *= factorial(position.y.powf(2.5));
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

struct RemoveEntitiesParallelSystem;

impl System for RemoveEntitiesParallelSystem {
    fn single_entity_step(&self, single_entity: &mut SingleMutEntity) {
        single_entity.remove_entity();
    }

    fn implements_single_entity_step(&self) -> bool {
        true
    }
}

const NUMITER: usize = 1;

fn main() {
    // test the performance with a varrying number of components

    for i in 0..5000 {
        if i % 100 != 0 {
            continue;
        }

        let mut total_time_add = 0;
        let mut total_time_systems_run = 0.0;
        // arbitrary number of iterations to get a more accurate result
        for _ in 0..NUMITER {
            let mut world = World::new();
            world.add_system(PositionSystem {});
            //world.add_system(SingleThreadedPositionSystem {});
            //world.add_system(RemoveEntitiesSystem {});
            //world.add_system(RemoveEntitiesParallelSystem {});

            let entities_and_components = &mut world.entities_and_components;

            let start_time = Instant::now();
            // add entities and components
            for j in 0..i {
                if j % 2 == 0 {
                    entities_and_components
                        .add_entity_with((Position { x: 0.0, y: 0.0 }, Health { health: 100.0 }));
                } else {
                    entities_and_components.add_entity_with((Velocity { x: 0.0, y: 0.0 },));
                }
            }
            total_time_add += start_time.elapsed().as_micros();

            let start_time = Instant::now();
            for _ in 0..100 {
                world.run();
            }
            total_time_systems_run += start_time.elapsed().as_micros() as f32;
        }
        total_time_systems_run /= 100.0;

        // same thing but with bevy ecs
        let mut total_time_add_bevy = 0;
        let mut total_time_systems_run_bevy = 0.0;

        for _ in 0..NUMITER {
            let mut world = bevy_ecs::prelude::World::new();

            let start_time = Instant::now();
            for j in 0..i {
                if j % 2 == 0 {
                    world.spawn((
                        BevyPosition { x: 0.0, y: 0.0 },
                        BevyHealth { health: 100.0 },
                    ));
                } else {
                    world.spawn((BevyVelocity { x: 0.0, y: 0.0 },));
                }
            }
            total_time_add_bevy += start_time.elapsed().as_micros();

            let mut schedule = bevy_ecs::prelude::Schedule::default();
            schedule.add_systems(bevy_position_system);

            let start_time = Instant::now();
            for _ in 0..100 {
                schedule.run(&mut world);
            }
            total_time_systems_run_bevy += start_time.elapsed().as_micros() as f32;
        }
        total_time_systems_run_bevy /= 100.0;

        print!("\n{}: ", i);

        for _ in 0..(total_time_add as u32 / 10000) {
            print!("{}", "|".blue());
        }
        for _ in 0..(total_time_systems_run as u32 / 10000) {
            print!("{}", "|".green());
        }
        println!();
        print!(
            "      {}μs for {} entities added {}μs for systems run, overall {}μs for ABC ECS\n",
            total_time_add as f32 / 1000.0,
            i,
            total_time_systems_run / 1000.0,
            (total_time_add + total_time_systems_run as u128) as f32 / 1000.0
        );

        for _ in 0..(total_time_add_bevy as u32 / 10000) {
            print!("{}", "|".blue());
        }

        for _ in 0..(total_time_systems_run_bevy as u32 / 10000) {
            print!("{}", "|".green());
        }

        println!();

        print!(
            "      {}μs for {} entities added {}μs for systems run, overall {}μs for Bevy ECS\n",
            total_time_add_bevy as f32 / 1000.0,
            i,
            total_time_systems_run_bevy / 1000.0,
            (total_time_add_bevy + total_time_systems_run_bevy as u128) as f32 / 1000.0
        );
    }
}
