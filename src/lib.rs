use anymap::AnyMap;
use slotmap::{DefaultKey, SlotMap};
use std::any;

pub struct EntitiesAndComponents {
    // Maybe there should be an object that takes a component
    // and has a list of which entities have that component?
    // This would make it easier to iterate over all entities with a certain component,
    // without having to iterate over all entities
    entities: Vec<Entity>,
    components: SlotMap<DefaultKey, AnyMap>, // where components[entity_id][component_id]
}

impl EntitiesAndComponents {
    pub fn new() -> Self {
        EntitiesAndComponents {
            entities: vec![],
            components: SlotMap::new(),
        }
    }

    /// Gets a reference to all the entities in the game engine
    pub fn get_entities(&self) -> &Vec<Entity> {
        &self.entities
    }
    /// Gets a copy of an entity at a certain index
    pub fn get_entity(&self, index: usize) -> Entity {
        self.entities[index].clone()
    }
    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Gets a reference to all the components on an entity
    /// Returns an AnyMap, which can be used to get a reference to a component
    /// This should rarely if ever be used
    pub fn get_components(&self, entity: Entity) -> &AnyMap {
        self.components
            .get(entity.entity_id)
            .expect("Entity ID does not exist, was the Entity ID edited?")
    }

    /// Gets a mutable reference to the components on an entity
    /// If the entity does not exist, it will panic
    pub fn get_components_mut(&mut self, entity: Entity) -> &mut AnyMap {
        self.components
            .get_mut(entity.entity_id)
            .expect("Entity ID does not exist, was the Entity ID edited?")
    }

    /// Gets a reference to a component on an entity
    /// If the component does not exist on the entity, it will return None
    pub fn try_get_component<T: 'static>(&self, entity: Entity) -> Option<&Box<T>> {
        self.components
            .get(entity.entity_id)
            .expect("Entity ID does not exist, was the Entity ID edited?")
            .get::<Box<T>>()
    }

    /// Gets a mutable reference to a component on an entity
    /// If the component does not exist on the entity, it will return None
    pub fn try_get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut Box<T>> {
        self.components
            .get_mut(entity.entity_id)
            .expect("Entity ID does not exist, was the Entity ID edited?")
            .get_mut::<Box<T>>()
    }

    /// Gets a reference to a component on an entity
    /// If the component does not exist on the entity, it will panic
    pub fn get_component<T: 'static>(&self, entity: Entity) -> &T {
        self.components
            .get(entity.entity_id)
            .expect("Entity ID does not exist, was the Entity ID edited?")
            .get::<Box<T>>()
            .expect(
                "Component does not exist on the object, was the Component added to the entity?",
            )
    }

    /// Gets a mutable reference to a component on an entity
    /// If the component does not exist on the entity, it will panic
    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> &mut T {
        self.components
            .get_mut(entity.entity_id)
            .expect("Entity ID does not exist, was the Entity ID edited?")
            .get_mut::<Box<T>>()
            .expect(
                "Component does not exist on the object, was the Component added to the entity?",
            )
    }

    /// Adds an entity to the game engine
    /// Returns the entity
    pub fn add_entity(&mut self) -> Entity {
        let entity_id = self.components.insert(AnyMap::new());
        self.entities.push(Entity { entity_id });

        Entity { entity_id }
    }

    /// Adds a component to an entity
    /// If the component already exists on the entity, it will be overwritten
    pub fn add_component_to<T: Component>(&mut self, entity: Entity, component: T) {
        let components = self
            .components
            .get_mut(entity.entity_id)
            .expect("Entity ID does not exist, was the Entity ID edited?");

        components.insert(Box::new(component));
    }
}

pub trait Components: 'static {
    fn get_components(&self) -> Vec<&dyn any::Any>;
    fn get_mut_components(&mut self) -> Vec<&mut dyn any::Any>;
}

/// This macro is used to get a variable ammount of components from an entity
/// It returns a tuple of references to the components
/// ```rust
/// use ABC_ECS::{get_components, GameEngine, Component};
///
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// impl Component for Position {}
///
/// struct Velocity {
///     x: f32,
///     y: f32,
/// }
///
/// impl Component for Velocity {}
///
///
/// fn main() {
///     let mut engine = GameEngine::new();
///     let entities_and_components = &mut engine.entities_and_components;
///
///     let entity = entities_and_components.add_entity();
///
///     entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
///     entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });
///
///     let (position, velocity) = get_components!(engine.entities_and_components, entity, Position, Velocity);
///
///     println!("Position: {}, {}", position.x, position.y);
///     println!("Velocity: {}, {}", velocity.x, velocity.y);
/// }
/// ```
#[macro_export]
macro_rules! get_components {
    ($engine:expr, $entity:expr, $($component:ty),*) => {
        {
            let mut all_types = vec![];
            $(
                all_types.push(std::any::TypeId::of::<$component>());
            )*

            for i in 0..all_types.len() {
                for j in i+1..all_types.len() {
                    assert_ne!(all_types[i], all_types[j], "You cannot borrow the same component more than once!");
                }
            }

            (
                $(
                    {
                        let pointer: *const $component = &*$engine.get_component::<$component>($entity);
                        let reference = unsafe { &*pointer };
                        reference
                    },
                )*
            )
        }
    };
}

/// This macro is used to muttably borrow a variable ammount of components from an entity
///
/// It returns a tuple of references to the components
/// ```rust
/// use ABC_ECS::{get_components_mut, GameEngine, Component};
///
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// impl Component for Position {}
///
/// struct Velocity {
///     x: f32,
///     y: f32,
/// }
///
/// impl Component for Velocity {}
///
///
/// fn main() {
///     let mut engine = GameEngine::new();
///     let entities_and_components = &mut engine.entities_and_components;
///
///     let entity = entities_and_components.add_entity();
///
///     entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
///     entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });
///
///     let (position, velocity) = get_components_mut!(engine.entities_and_components, entity, Position, Velocity);
///
///     position.x += velocity.x;
///     position.y += velocity.y;
///
///     println!("Position: {}, {}", position.x, position.y);
/// }
/// ```
///
/// WARNING: This macro is not safe to use if you are borrowing the same component mutably more than once
///
/// It will panic if you do this in a single call to the macro, but it will not panic if you do it in seperate calls
#[macro_export]
macro_rules! get_components_mut {
    ($engine:expr, $entity:expr, $($component:ty),*) => {
        {
            let mut all_types = vec![];
            $(
                all_types.push(std::any::TypeId::of::<$component>());
            )*

            for i in 0..all_types.len() {
                for j in i+1..all_types.len() {
                    assert_ne!(all_types[i], all_types[j], "You cannot borrow the same component mutably more than once!");
                }
            }

            (
                $(
                    {
                        let pointer: *mut $component = &mut *$engine.get_component_mut::<$component>($entity);
                        let reference = unsafe { &mut *pointer };
                        reference
                    },
                )*
            )
        }
    };
}

pub struct GameEngine {
    pub entities_and_components: EntitiesAndComponents,
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

pub trait Component: 'static {}

// The Entity will just be an ID that can be
// indexed into arrays of components for now...
#[derive(Clone, Copy)]
pub struct Entity {
    entity_id: DefaultKey,
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

    impl Component for Position {}

    struct Velocity {
        x: f32,
        y: f32,
    }

    impl Component for Velocity {}

    struct MovementSystem {}

    impl System for MovementSystem {
        fn run(&self, engine: &mut EntitiesAndComponents) {
            for i in 0..engine.entities.len() {
                let entity = engine.entities[i];

                // be very careful when using this macro like this
                // using it this way could cause a data race if you are not careful
                let (velocity,) = get_components!(engine, entity, Velocity);
                let (position,) = get_components_mut!(engine, entity, Position);

                position.x += velocity.x;
                position.y += velocity.y;

                println!("Position: {}, {}", position.x, position.y);
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

        // not sure how to test this yet...
        // in the current state of the code, this causes a mutable borrow error
        // not sure if that is a positive design choice or not

        /*let position = entities_and_components.get_component::<Position>(entity);
        assert_eq!(position.x, 5.0);
        assert_eq!(position.y, 5.0);

        let velocity = entities_and_components.get_component::<Velocity>(entity);
        assert_eq!(velocity.x, 1.0);
        assert_eq!(velocity.y, 1.0);*/
    }

    #[test]
    fn test_overriding_components() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Position { x: 6.0, y: 1.0 });

        let position = entities_and_components.get_component::<Position>(entity);
        assert_eq!(position.x, 6.0);
        assert_eq!(position.y, 1.0);
    }
}
