use anymap::AnyMap;
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::{any::TypeId, collections::HashMap};

// The Entity will just be an ID that can be
// indexed into arrays of components for now...
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Entity {
    entity_id: DefaultKey,
}

pub struct EntitiesAndComponents {
    // Maybe there should be an object that takes a component
    // and has a list of which entities have that component?
    // This would make it easier to iterate over all entities with a certain component,
    // without having to iterate over all entities
    entities: SlotMap<DefaultKey, Entity>,
    components: SlotMap<DefaultKey, AnyMap>, // where components[entity_id][component_id]
    entities_with_components: HashMap<TypeId, Vec<Entity>>,
    type_ids_on_entity: SecondaryMap<DefaultKey, Vec<TypeId>>,
}

impl EntitiesAndComponents {
    pub fn new() -> Self {
        EntitiesAndComponents {
            entities: SlotMap::new(),
            components: SlotMap::new(),
            entities_with_components: HashMap::new(),
            type_ids_on_entity: SecondaryMap::new(),
        }
    }

    /// Adds an entity to the game engine
    /// Returns the entity
    pub fn add_entity(&mut self) -> Entity {
        let entity_id = self.components.insert(AnyMap::new());
        self.entities.insert(Entity { entity_id });
        self.type_ids_on_entity.insert(entity_id, vec![]);

        Entity { entity_id }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        for type_id in self.type_ids_on_entity[entity.entity_id].clone() {
            match self.entities_with_components.get_mut(&type_id) {
                Some(entities) => {
                    entities.retain(|e| *e != entity);
                }
                None => {}
            }
        }
        self.type_ids_on_entity.remove(entity.entity_id);
        self.components.remove(entity.entity_id);
        self.entities.remove(entity.entity_id);
    }

    /// Gets a reference to all the entities in the game engine
    /// Should rarely if ever be used
    pub fn get_entities(&self) -> Vec<Entity> {
        // clone the entities vector
        self.entities.values().cloned().collect::<Vec<Entity>>()
    }

    /// Gets a copy of an entity at a certain index
    pub fn get_nth_entity(&self, index: usize) -> Option<Entity> {
        // get the nth entity
        if let Some(entity) = self.entities.values().nth(index) {
            Some(entity.clone())
        } else {
            None
        }
    }

    /// Gets the number of entities in the game engine
    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Gets a reference to all the components on an entity
    /// Returns an AnyMap, which can be used to get a reference to a component
    /// This should rarely if ever be used
    pub fn get_components(&self, entity: Entity) -> &AnyMap {
        self.components.get(entity.entity_id).expect(
            format!("Entity ID {entity:?} does not exist, was the Entity ID edited?").as_str(),
        )
    }

    /// Gets a mutable reference to the components on an entity
    /// If the entity does not exist, it will panic
    pub fn get_components_mut(&mut self, entity: Entity) -> &mut AnyMap {
        self.components.get_mut(entity.entity_id).expect(
            format!("Entity ID {entity:?} does not exist, was the Entity ID edited?").as_str(),
        )
    }

    /// Gets a reference to a component on an entity
    /// If the component does not exist on the entity, it will return None
    pub fn try_get_component<T: 'static>(&self, entity: Entity) -> Option<&Box<T>> {
        self.components
            .get(entity.entity_id)
            .expect(
                format!("Entity ID {entity:?} does not exist, was the Entity ID edited?").as_str(),
            )
            .get::<Box<T>>()
    }

    /// Gets a mutable reference to a component on an entity
    /// If the component does not exist on the entity, it will return None
    pub fn try_get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut Box<T>> {
        self.components
            .get_mut(entity.entity_id)
            .expect(
                format!("Entity ID {entity:?} does not exist, was the Entity ID edited?").as_str(),
            )
            .get_mut::<Box<T>>()
    }

    /// Gets a reference to a component on an entity
    /// If the component does not exist on the entity, it will panic
    pub fn get_component<T: 'static>(&self, entity: Entity) -> &T {
        let type_name = std::any::type_name::<T>();
        self.components
            .get(entity.entity_id)
            .expect(format!("Entity ID {entity:?} does not exist, was the Entity ID edited?").as_str())
            .get::<Box<T>>()
            .expect(
                &format!(
                    "Component {type_name} does not exist on the object, was the Component added to the entity?"
                ),
            )
    }

    /// Gets a mutable reference to a component on an entity
    /// If the component does not exist on the entity, it will panic
    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> &mut T {
        let type_name = std::any::type_name::<T>();
        self.components
            .get_mut(entity.entity_id)
            .expect(format!("Entity ID {entity:?} does not exist, was the Entity ID edited?").as_str())
            .get_mut::<Box<T>>()
            .expect(&format!(
                "Component {type_name} does not exist on the object, was the Component added to the entity?"
            ))
    }

    /// Adds a component to an entity
    /// If the component already exists on the entity, it will be overwritten
    pub fn add_component_to<T: Component>(&mut self, entity: Entity, component: T) {
        // add the component to the entity
        let components = self.components.get_mut(entity.entity_id).expect(
            format!("Entity ID {entity:?} does not exist, was the Entity ID edited?").as_str(),
        );
        components.insert(Box::new(component));

        // add the entity to the list of entities with the component
        match self.entities_with_components.entry(TypeId::of::<T>()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().push(entity);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(vec![entity]);
            }
        }
    }

    pub fn remove_component_from<T: Component>(&mut self, entity: Entity) {
        // remove the component from the entity
        let components = self.components.get_mut(entity.entity_id).expect(
            format!("Entity ID {entity:?} does not exist, was the Entity ID edited?").as_str(),
        );
        components.remove::<Box<T>>();

        // remove the entity from the list of entities with the component
        match self.entities_with_components.get_mut(&TypeId::of::<T>()) {
            Some(entities) => {
                entities.retain(|e| *e != entity);
            }
            None => {}
        }
    }

    /// returns a vector of all the entities that have a certain component
    /// if no entities have the component, it will return an empty vector
    /// clones the vector, so it is not very efficient
    pub fn get_entities_with_component<T: Component>(&self) -> Vec<Entity> {
        match self.entities_with_components.get(&TypeId::of::<T>()) {
            Some(entities) => entities.clone(),
            None => vec![],
        }
    }

    pub fn get_entity_count_with_component<T: Component>(&self) -> usize {
        match self.entities_with_components.get(&TypeId::of::<T>()) {
            Some(entities) => entities.len(),
            None => 0,
        }
    }

    pub fn get_nth_entity_with_component<T: Component>(&self, index: usize) -> Option<Entity> {
        match self.entities_with_components.get(&TypeId::of::<T>()) {
            Some(entities) => {
                if let Some(entity) = entities.get(index) {
                    Some(entity.clone())
                } else {
                    None
                }
            }
            None => None,
        }
    }
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

/// This is a macro used to try to get a variable ammount of components from an entity
/// It returns a tuple of option references to the components
/// ```rust
/// use ABC_ECS::{try_get_components, GameEngine, Component};
/// struct Position {
///    x: f32,
///    y: f32,
/// }
/// impl Component for Position {}
/// struct Velocity {
///   x: f32,
///   y: f32,
/// }
/// impl Component for Velocity {}
/// fn main() {
///    let mut engine = GameEngine::new();
///   let entities_and_components = &mut engine.entities_and_components;
///   let entity = entities_and_components.add_entity();
///   entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
///   entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });
///   let (position, velocity) = try_get_components!(engine.entities_and_components, entity, Position, Velocity);
///   assert_eq!(position.unwrap().x, 0.0);
///   assert_eq!(position.unwrap().y, 0.0);
///   assert_eq!(velocity.unwrap().x, 1.0);
///   assert_eq!(velocity.unwrap().y, 1.0);
/// }
/// ```
/// WARNING: This macro is not safe to use if you are borrowing the same component mutably more than once
/// It will panic if you do this in a single call to the macro, but it will not panic if you do it in seperate calls
#[macro_export]
macro_rules! try_get_components {
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
                        if let Some(component) = $engine.try_get_component::<$component>($entity) {
                            let pointer: *const $component = &**component;
                            let reference = unsafe { &*pointer };
                            Some(reference)
                        } else {
                            None
                        }
                        /*let pointer: *const $component = &*$engine.try_get_component::<$component>($entity).expect("Component does not exist on the entity");
                        let reference = unsafe { &*pointer };
                        reference*/
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
        for system in &mut self.systems {
            // not sure what to do about the mutability here...
            // maybe seperate the systems and the entities and components?
            system.run(&mut self.entities_and_components);
        }
    }
}

pub trait Component: 'static {}

/// Systems access and change components on objects
pub trait System {
    fn run(&mut self, engine: &mut EntitiesAndComponents);
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
        fn run(&mut self, engine: &mut EntitiesAndComponents) {
            for i in 0..engine.entities.len() {
                let entity = engine.get_nth_entity(i).unwrap(); // this should never panic

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
