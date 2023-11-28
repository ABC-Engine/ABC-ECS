use std::any;

pub struct EntitiesAndComponents {
    entities: Vec<Entity>,
    components: Vec<Vec<Box<dyn Component>>>, // where components[entity_id][component_id (which shouldn't need to indexed into)]
}

impl EntitiesAndComponents {
    pub fn new() -> Self {
        EntitiesAndComponents {
            entities: vec![],
            components: vec![],
        }
    }
    pub fn get_components(&self, entity: Entity) -> &Vec<Box<dyn Component>> {
        self.components
            .get(entity.entity_id)
            .expect("Entity ID does not exist, was the Entity ID edited?")
    }

    pub fn add_entity(&mut self) -> Entity {
        let entity_id = self.entities.len();
        self.entities.push(Entity {
            entity_id: entity_id,
        });
        self.components.push(vec![]);

        Entity {
            entity_id: entity_id,
        }
    }

    pub fn add_component_to<T: Component>(&mut self, entity: Entity, component: T) {
        let components = self
            .components
            .get_mut(entity.entity_id)
            .expect("Entity ID does not exist, was the Entity ID edited?");

        components.push(Box::new(component));
    }
}

struct GameEngine {
    entities_and_components: EntitiesAndComponents,
    systems: Vec<Box<dyn System>>,
}

impl GameEngine {
    pub fn new() -> Self {
        GameEngine {
            entities_and_components: EntitiesAndComponents::new(),
            systems: vec![],
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn run(&mut self) {
        for system in &self.systems {
            // not sure what to do about the mutability here...
            // maybe seperate the systems and the entities and components?
            system.run(&mut self.entities_and_components);
        }
    }
}

pub trait Component: 'static {
    fn as_any(&self) -> &dyn any::Any;
}

// The Entity will just be an ID that can be
// indexed into arrays of components for now...
#[derive(Clone, Copy)]
pub struct Entity {
    entity_id: usize,
}

/// Systems access and change components on objects
pub trait System {
    fn run(&self, engine: &mut EntitiesAndComponents);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {
        fn as_any(&self) -> &dyn any::Any {
            self
        }
    }

    struct Velocity {
        x: f32,
        y: f32,
    }

    impl Component for Velocity {
        fn as_any(&self) -> &dyn any::Any {
            self
        }
    }

    struct MovementSystem {}

    impl System for MovementSystem {
        fn run(&self, engine: &mut EntitiesAndComponents) {
            println!("Running Movement System");
            for entity in &engine.entities {
                for component in engine.get_components(*entity) {
                    if let Some(position) = component.as_any().downcast_ref::<Position>() {
                        println!("Position: {}, {}", position.x, position.y);
                    }
                }
            }
        }
    }

    #[test]
    fn it_works() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

        engine.add_system(Box::new(MovementSystem {}));

        for i in 0..5 {
            engine.run();
        }
    }
}
