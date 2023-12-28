use anymap::AnyMap;
use rustc_hash::FxHashMap;
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::{any::TypeId, collections::HashMap};
mod macros;
use macros::*;

// The Entity will just be an ID that can be
// indexed into arrays of components for now...
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Entity {
    pub entity_id: DefaultKey,
}

pub struct EntitiesAndComponents {
    // Maybe there should be an object that takes a component
    // and has a list of which entities have that component?
    // This would make it easier to iterate over all entities with a certain component,
    // without having to iterate over all entities
    entities: SlotMap<DefaultKey, Entity>,
    pub(crate) components: SlotMap<DefaultKey, AnyMap>, // where components[entity_id][component_id]
    entities_with_components: FxHashMap<TypeId, SecondaryMap<DefaultKey, Entity>>,
    type_ids_on_entity: SecondaryMap<DefaultKey, Vec<TypeId>>,
}

impl EntitiesAndComponents {
    pub fn new() -> Self {
        // not sure what the capacity should be here
        EntitiesAndComponents {
            entities: SlotMap::with_capacity(100),
            components: SlotMap::with_capacity(100),
            entities_with_components: FxHashMap::with_capacity_and_hasher(3, Default::default()),
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

    pub fn add_entity_with<T: OwnedComponents<Input = T>>(&mut self, components: T) -> Entity {
        let entity = <T>::make_entity_with_components(self, components);
        entity
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        for type_id in self.type_ids_on_entity[entity.entity_id].clone() {
            match self.entities_with_components.get_mut(&type_id) {
                Some(entities) => {
                    entities.remove(entity.entity_id);
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
    pub fn get_all_components(&self, entity: Entity) -> &AnyMap {
        self.components.get(entity.entity_id).unwrap_or_else(|| {
            panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
        })
    }

    /// Gets a mutable reference to the components on an entity
    /// If the entity does not exist, it will panic
    pub fn get_all_components_mut(&mut self, entity: Entity) -> &mut AnyMap {
        self.components
            .get_mut(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            })
    }

    /// Gets a reference to a component on an entity
    /// If the component does not exist on the entity, it will return None
    pub fn try_get_component<T: 'static>(&self, entity: Entity) -> Option<&Box<T>> {
        self.components
            .get(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            })
            .get::<Box<T>>()
    }

    /// Gets a mutable reference to a component on an entity
    /// If the component does not exist on the entity, it will return None
    pub fn try_get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut Box<T>> {
        self.components
            .get_mut(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            })
            .get_mut::<Box<T>>()
    }

    /// Gets a reference to a component on an entity
    /// If the component does not exist on the entity, it will panic
    pub fn get_components<'a, T: ComponentsRef<'a> + 'static>(
        &'a self,
        entity: Entity,
    ) -> T::Result {
        <T>::get_components(self, entity)
    }

    /// Gets a mutable reference to a component on an entity
    /// If the component does not exist on the entity, it will panic
    pub fn get_components_mut<'a, T: ComponentsMut<'a> + 'static>(
        &'a mut self,
        entity: Entity,
    ) -> T::Result {
        <T>::get_components_mut(self, entity)
    }

    pub fn try_get_components<'a, T: TryComponentsRef<'a> + 'static>(
        &'a self,
        entity: Entity,
    ) -> T::Result {
        <T>::try_get_components(self, entity)
    }

    pub fn try_get_components_mut<'a, T: TryComponentsMut<'a> + 'static>(
        &'a mut self,
        entity: Entity,
    ) -> T::Result {
        <T>::try_get_components_mut(self, entity)
    }

    /// Adds a component to an entity
    /// If the component already exists on the entity, it will be overwritten
    pub fn add_component_to<T: Component>(&mut self, entity: Entity, component: T) {
        // add the component to the entity
        let components = self
            .components
            .get_mut(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            });
        components.insert(Box::new(component));

        // add the entity to the list of entities with the component
        match self.entities_with_components.entry(TypeId::of::<T>()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().insert(entity.entity_id, entity);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut new_map = SecondaryMap::new();
                new_map.insert(entity.entity_id, entity);
                entry.insert(new_map);
            }
        }
        self.type_ids_on_entity[entity.entity_id].push(TypeId::of::<T>());
    }

    pub fn remove_component_from<T: Component>(&mut self, entity: Entity) {
        // remove the component from the entity
        let components = self
            .components
            .get_mut(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            });
        components.remove::<Box<T>>();

        // remove the entity from the list of entities with the component
        match self.entities_with_components.get_mut(&TypeId::of::<T>()) {
            Some(entities) => {
                entities.remove(entity.entity_id);
            }
            None => {}
        }
        // this is O(n) but, depending on the number of components on an entity, n should be small
        self.type_ids_on_entity[entity.entity_id].retain(|t| *t != TypeId::of::<T>());
    }

    /// returns an iterator over all entities with a certain component
    pub fn get_entities_with_component<T: Component>(
        &self,
    ) -> Option<slotmap::secondary::Values<'_, DefaultKey, Entity>> {
        match self.entities_with_components.get(&TypeId::of::<T>()) {
            Some(entities) => Some(entities.values()),
            None => None,
        }
    }

    pub fn get_entity_count_with_component<T: Component>(&self) -> usize {
        match self.entities_with_components.get(&TypeId::of::<T>()) {
            Some(entities) => entities.len(),
            None => 0,
        }
    }

    /// gets the nth entity with a certain component
    /// O(n) use get_entities_with_component if you need to iterate over all entities with a certain component
    pub fn get_entity_with_component<T: Component>(&self, index: usize) -> Option<Entity> {
        match self.entities_with_components.get(&TypeId::of::<T>()) {
            Some(entities) => {
                if let Some(entity) = entities.values().nth(index) {
                    Some(entity.clone())
                } else {
                    None
                }
            }
            None => None,
        }
    }
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

impl<T: 'static> Component for T {}

/// Systems access and change components on objects
pub trait System {
    fn run(&mut self, engine: &mut EntitiesAndComponents);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    //impl Component for Position {}

    #[derive(Debug, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    //impl Component for Velocity {}

    struct MovementSystem {}

    impl System for MovementSystem {
        fn run(&mut self, engine: &mut EntitiesAndComponents) {
            for i in 0..engine.entities.len() {
                let entity = engine.get_nth_entity(i).unwrap(); // this should never panic

                // be very careful when using this macro like this
                // using it this way could cause a data race if you are not careful
                //let (velocity,) = get_components!(engine, entity, Velocity);
                let (position, velocity) =
                    <(Position, Velocity)>::get_components_mut(engine, entity);

                position.x += velocity.x;
                position.y += velocity.y;

                println!("Position: {}, {}", position.x, position.y);
            }
        }
    }

    #[test]
    fn test_components_mut() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

        engine.add_system(Box::new(MovementSystem {}));

        for _ in 0..5 {
            engine.run();
        }
    }

    #[test]
    fn test_try_get_components() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

        let (position, velocity) =
            <(Position, Velocity)>::try_get_components(entities_and_components, entity);

        assert_eq!(position.unwrap().x, 0.0);
        assert_eq!(position.unwrap().y, 0.0);
        assert_eq!(velocity.unwrap().x, 1.0);
        assert_eq!(velocity.unwrap().y, 1.0);
    }

    #[test]
    fn test_overriding_components() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Position { x: 6.0, y: 1.0 });

        let (position,) = entities_and_components.get_components::<(Position,)>(entity);
        assert_eq!(position.x, 6.0);
        assert_eq!(position.y, 1.0);
    }

    #[test]
    fn test_multiple_entities() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();
        let entity_2 = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

        entities_and_components.add_component_to(entity_2, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity_2, Velocity { x: 1.0, y: 1.0 });

        // this should compile but, currently you can't borrow from two different entities mutably at the same time
        let (position,) = entities_and_components.get_components_mut::<(Position,)>(entity);

        println!("Position: {}, {}", position.x, position.y);
    }

    #[test]
    fn test_add_entity_with_components() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components
            .add_entity_with((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));

        let (position, velocity) =
            entities_and_components.get_components::<(Position, Velocity)>(entity);

        assert_eq!(position.x, 0.0);
        assert_eq!(position.y, 0.0);
        assert_eq!(velocity.x, 1.0);
        assert_eq!(velocity.y, 1.0);
    }

    #[test]
    fn test_entity_removal() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components
            .add_entity_with((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));

        let (position, velocity) =
            entities_and_components.get_components::<(Position, Velocity)>(entity);

        assert_eq!(position.x, 0.0);
        assert_eq!(position.y, 0.0);
        assert_eq!(velocity.x, 1.0);
        assert_eq!(velocity.y, 1.0);

        entities_and_components.remove_entity(entity);

        assert_eq!(entities_and_components.get_entity_count(), 0);

        let entity = entities_and_components.add_entity();

        // make sure the new entity doesn't have the old entity's components
        let (position, velocity) =
            entities_and_components.try_get_components::<(Position, Velocity)>(entity);

        assert_eq!(position, None);
        assert_eq!(velocity, None);
    }

    // this test should not compile
    /*#[test]
    fn test_compile_fail_multiple_muts() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 1.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

        let (position, velocity) =
            get_components_mut!(engine.entities_and_components, entity, Position, Velocity);

        let (position_2, velocity_2) =
            get_components_mut!(engine.entities_and_components, entity, Position, Velocity);

        position.x += position_2.x;
        position.y += position_2.y;

        println!("Position: {}, {}", position.x, position_2.y);
    }*/

    // this test should not compile
    /*#[test]
    fn test_lifetimes() {
        let (position, velocity): (&mut Position, &mut Velocity);
        {
            let mut engine = GameEngine::new();
            let entities_and_components = &mut engine.entities_and_components;

            let entity = entities_and_components.add_entity();

            entities_and_components.add_component_to(entity, Position { x: 1.0, y: 0.0 });
            entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

            let (position, velocity) =
                <(Position, Velocity)>::get_components(entities_and_components, entity);

            //(position, velocity) =
            //    get_components_mut!(engine.entities_and_components, entity, Position, Velocity);
        }

        // should not be possible, but the lifetimes aren't linked
        position.x += velocity.x;
        position.y += velocity.y;

        println!("Position: {}, {}", position.x, position.y);
    }*/
}
