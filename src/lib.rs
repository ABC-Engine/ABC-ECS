#[doc = include_str!("../README.md")]
use anymap::Map;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rustc_hash::FxHashMap;
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::any::{Any, TypeId};
mod macros;
pub use macros::*;
use rayon::prelude::ParallelSliceMut;

// The Entity will just be an ID that can be
// indexed into arrays of components for now...
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Entity {
    pub entity_id: DefaultKey,
}

pub trait Resource: 'static + Send + Sync {
    fn update(&mut self) {}
    fn as_any(&self) -> &dyn Any;
}

pub struct EntitiesAndComponents {
    entities: SlotMap<DefaultKey, Entity>,
    pub(crate) components: SlotMap<DefaultKey, Map<dyn Any + Send + Sync + 'static>>, // where components[entity_id][component_id]
    entities_with_components: FxHashMap<TypeId, SecondaryMap<DefaultKey, Entity>>,
    type_ids_on_entity: SecondaryMap<DefaultKey, Vec<TypeId>>,
    /// resources holds all the resources that are not components and do not have any relation to entities
    /// they are read only and can be accessed by any system
    /// Resources have their own trait, Resource, which has an update method that is called every frame
    pub(crate) resources: FxHashMap<TypeId, Box<dyn Resource>>,
}

impl EntitiesAndComponents {
    pub fn new() -> Self {
        // not sure what the capacity should be here
        EntitiesAndComponents {
            entities: SlotMap::with_capacity(100),
            components: SlotMap::with_capacity(100),
            entities_with_components: FxHashMap::with_capacity_and_hasher(3, Default::default()),
            type_ids_on_entity: SecondaryMap::new(),
            resources: FxHashMap::default(),
        }
    }

    /// Adds an entity to the game engine
    /// Returns the entity
    pub fn add_entity(&mut self) -> Entity {
        let entity_id = self.components.insert(Map::new());
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
    pub fn get_all_components(
        &self,
        entity: Entity,
    ) -> &anymap::Map<(dyn Any + Send + Sync + 'static)> {
        self.components.get(entity.entity_id).unwrap_or_else(|| {
            panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
        })
    }

    /// Gets a mutable reference to the components on an entity
    /// If the entity does not exist, it will panic
    pub fn get_all_components_mut(
        &mut self,
        entity: Entity,
    ) -> &mut anymap::Map<(dyn Any + Send + Sync + 'static)> {
        self.components
            .get_mut(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            })
    }

    /// Gets a reference to a component on an entity
    /// If the component does not exist on the entity, it will return None
    pub fn try_get_component<T: Component + Send + Sync>(&self, entity: Entity) -> Option<&Box<T>> {
        self.components
            .get(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            })
            .get::<Box<T>>()
    }

    /// Gets a mutable reference to a component on an entity
    /// If the component does not exist on the entity, it will return None
    pub fn try_get_component_mut<T: Component + Send + Sync>(
        &mut self,
        entity: Entity,
    ) -> Option<&mut Box<T>> {
        self.components
            .get_mut(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            })
            .get_mut::<Box<T>>()
    }

    /// Gets a tuple of references to components on an entity
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
    pub fn add_component_to<T: Component + Send + Sync>(&mut self, entity: Entity, component: T) {
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

    pub fn remove_component_from<T: Component + Send + Sync>(&mut self, entity: Entity) {
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
    ) -> std::iter::Flatten<std::option::IntoIter<slotmap::secondary::Values<'_, DefaultKey, Entity>>>
    {
        match self.entities_with_components.get(&TypeId::of::<T>()) {
            Some(entities) => Some(entities.values()).into_iter().flatten(),
            None => None.into_iter().flatten(), // this is a hack so that it returns an empty iterator
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

    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        match self.resources.get(&TypeId::of::<T>()) {
            Some(resource) => {
                let resource = (&**resource)
                    .as_any()
                    .downcast_ref::<T>()
                    .unwrap_or_else(|| {
                        panic!(
                            "Resource of type {type:?} does not exist, was the type edited?",
                            type = std::any::type_name::<T>()
                        );
                    });
                Some(resource)
            }
            None => None,
        }
    }

    pub fn add_resource<T: Resource>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }
}

pub struct SingleMutEntity<'a> {
    entity: Entity,
    entities_and_components: &'a mut EntitiesAndComponents,
}

// for safety reasons, we need to make sure we only access data pertaining to this entity
impl<'a> SingleMutEntity<'a> {
    pub fn get_component<T: Component + Send + Sync>(&self) -> &T {
        self.entities_and_components
            .try_get_component::<T>(self.entity)
            .unwrap_or_else(|| {
                panic!(
                    "Component of type {type:?} does not exist on entity {entity:?}",
                    type = std::any::type_name::<T>(),
                    entity = self.entity
                );
            })
    }

    pub fn try_get_component<T: Component + Send + Sync>(&self) -> Option<&Box<T>> {
        self.entities_and_components
            .try_get_component::<T>(self.entity)
    }

    pub fn get_component_mut<T: Component + Send + Sync>(&mut self) -> &mut T {
        self.entities_and_components
            .try_get_component_mut::<T>(self.entity)
            .unwrap_or_else(|| {
                panic!(
                    "Component of type {type:?} does not exist on entity {entity:?}",
                    type = std::any::type_name::<T>(),
                    entity = self.entity
                );
            })
    }

    pub fn try_get_component_mut<T: Component + Send + Sync>(&mut self) -> Option<&mut Box<T>> {
        self.entities_and_components
            .try_get_component_mut::<T>(self.entity)
    }

    pub fn get_components<'b, T: ComponentsRef<'b> + 'static>(&'b self) -> T::Result {
        <T>::get_components(self.entities_and_components, self.entity)
    }

    pub fn try_get_components<'b, T: TryComponentsRef<'b> + 'static>(&'b self) -> T::Result {
        <T>::try_get_components(self.entities_and_components, self.entity)
    }

    pub fn get_components_mut<'b, T: ComponentsMut<'b> + 'static>(&'b mut self) -> T::Result {
        <T>::get_components_mut(self.entities_and_components, self.entity)
    }

    pub fn try_get_components_mut<'b, T: TryComponentsMut<'b> + 'static>(
        &'b mut self,
    ) -> T::Result {
        <T>::try_get_components_mut(self.entities_and_components, self.entity)
    }

    pub fn remove_component<T: Component + Send + Sync>(&mut self) {
        self.entities_and_components
            .remove_component_from::<T>(self.entity);
    }

    pub fn add_component<T: Component + Send + Sync>(&mut self, component: T) {
        self.entities_and_components
            .add_component_to(self.entity, component);
    }

    pub fn has_component<T: Component + Send + Sync>(&self) -> bool {
        self.entities_and_components
            .try_get_component::<T>(self.entity)
            .is_some()
    }

    pub fn remove_entity(&mut self) {
        self.entities_and_components.remove_entity(self.entity);
    }
}

struct EntitiesAndComponentPtr {
    entities_and_components: *mut EntitiesAndComponents,
}

impl EntitiesAndComponentPtr {
    pub fn as_mut(&self) -> &mut EntitiesAndComponents {
        unsafe { &mut *self.entities_and_components }
    }
}

// impl send and sync for EntitiesAndComponentPtr
unsafe impl Send for EntitiesAndComponentPtr {}
unsafe impl Sync for EntitiesAndComponentPtr {}

pub struct World {
    pub entities_and_components: EntitiesAndComponents,
    systems: Vec<Box<dyn System + Sync + Send>>,
}

impl World {
    pub fn new() -> Self {
        World {
            entities_and_components: EntitiesAndComponents::new(),
            systems: vec![],
        }
    }

    pub fn add_system<T: System + Send + Sync + 'static>(&mut self, system: T) {
        self.systems.push(Box::new(system));
    }

    pub fn run(&mut self) {
        for resource in self.entities_and_components.resources.values_mut() {
            resource.update();
        }

        if self.systems.is_empty() {
            return;
        }

        // run the prestep function for each systems in parallel
        {
            // check which systems implement the prestep function and collect mutable references to them
            let mut systems_with_prestep = self
                .systems
                .iter_mut()
                .filter(|system| system.implements_prestep())
                .collect::<Vec<&mut Box<dyn System + Sync + Send>>>();

            systems_with_prestep
                .par_iter_mut()
                .for_each(|system| system.prestep(&self.entities_and_components));
        }

        {
            // check which systems implement the single_entity_step function and collect mutable references to them
            let systems_with_single_entity_step = self
                .systems
                .iter()
                .filter(|system| system.implements_single_entity_step())
                .collect::<Vec<&Box<dyn System + Sync + Send>>>();

            if !systems_with_single_entity_step.is_empty() {
                let entities_and_components_ptr = &mut self.entities_and_components as *mut _;
                let entities_and_components_ptr = EntitiesAndComponentPtr {
                    entities_and_components: entities_and_components_ptr,
                };

                /*let chunk_size = ((self.entities_and_components.get_entity_count())
                / (self.num_cpus * 2))
                .max(20);*/
                let chunk_size = 5;

                // run the single_entity_step function for each entity in parallel
                let entities = &mut self.entities_and_components.get_entities();
                let par_chunks = entities.par_chunks_mut(chunk_size);
                par_chunks.for_each(|entity_chunk| {
                    for entity in entity_chunk {
                        for system in systems_with_single_entity_step.as_slice() {
                            let mut single_entity = SingleMutEntity {
                                entity: *entity,
                                entities_and_components: entities_and_components_ptr.as_mut(),
                            };

                            system.single_entity_step(&mut single_entity);
                        }
                    }
                });
            }
        }

        for system in &mut self.systems {
            system.run(&mut self.entities_and_components);
        }
    }
}

pub trait Component: 'static {}

impl<T: 'static> Component for T {}

/// Systems access and change components on objects
/// Be careful to implement get_allow_entity_based_multithreading as true if you want to use the single_entity_step function
/// If you don't it will still work but, it will be slower (in most cases)
pub trait System {
    /// This function can collect data that will be used in the single_entity_step function
    /// This allows both functions to be called in parallel, without a data race
    /// If you implement this function, make sure to implement implements_prestep as true
    fn prestep(&mut self, engine: &EntitiesAndComponents) {}
    /// Should just return true or false based on whether or not the system implements the prestep function
    fn implements_prestep(&self) -> bool {
        false
    }
    /// If you implement this function, it will be called for each entity in parallel, but make sure to implement get_allow_single_entity_step as true
    fn single_entity_step(&self, single_entity: &mut SingleMutEntity) {}
    /// Should just return true or false based on whether or not the system implements the single_entity_step function
    fn implements_single_entity_step(&self) -> bool {
        false
    }
    fn run(&mut self, engine: &mut EntitiesAndComponents) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
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

    struct ParallelMovementSystem {}

    impl System for ParallelMovementSystem {
        fn single_entity_step(&self, single_entity: &mut SingleMutEntity) {
            let (position, velocity) = single_entity.get_components_mut::<(Position, Velocity)>();

            position.x += velocity.x;
            position.y += velocity.y;

            println!("Position: {}, {}", position.x, position.y);
        }
        fn implements_single_entity_step(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_components_mut() {
        let mut engine = World::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

        engine.add_system(MovementSystem {});

        for _ in 0..5 {
            engine.run();
        }
    }

    #[test]
    fn test_try_get_components() {
        let mut engine = World::new();
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
        let mut engine = World::new();
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
        let mut engine = World::new();
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
        let mut engine = World::new();
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
        let mut engine = World::new();
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

    #[test]
    fn test_get_entities_with_component() {
        let mut engine = World::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();
        let entity_2 = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

        entities_and_components.add_component_to(entity_2, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity_2, Velocity { x: 1.0, y: 1.0 });

        let entities = entities_and_components.get_entities_with_component::<Position>();

        assert_eq!(entities.count(), 2);
    }

    #[test]
    #[should_panic]
    fn test_generation_values() {
        let mut engine = World::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity_1 = entities_and_components.add_entity();
        let entity_2 = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity_1, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity_1, Velocity { x: 1.0, y: 1.0 });

        entities_and_components.add_component_to(entity_2, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity_2, Velocity { x: 1.0, y: 1.0 });

        // remove the first entity
        entities_and_components.remove_entity(entity_1);

        // add a new entity
        let entity_3 = entities_and_components.add_entity();

        // make sure the new entity doesn't have the old entity's components
        let (position, velocity) =
            entities_and_components.try_get_components::<(Position, Velocity)>(entity_3);

        assert_eq!(position, None);
        assert_eq!(velocity, None);

        let (position, velocity) =
            entities_and_components.try_get_components::<(Position, Velocity)>(entity_1);
    }

    #[test]
    fn test_resources() {
        struct TestResource {
            value: i32,
        }

        impl Resource for TestResource {
            fn update(&mut self) {
                self.value += 1;
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        let mut engine = World::new();
        {
            let entities_and_components = &mut engine.entities_and_components;

            let resource = TestResource { value: 0 };

            entities_and_components.add_resource(resource);

            let resource = entities_and_components
                .get_resource::<TestResource>()
                .unwrap();

            assert_eq!(resource.value, 0);
        }

        for _ in 0..5 {
            engine.run();
        }

        {
            let entities_and_components = &mut engine.entities_and_components;

            let resource = entities_and_components
                .get_resource::<TestResource>()
                .unwrap();

            assert_eq!(resource.value, 5);
        }
    }

    #[test]
    fn test_parallel_systems() {
        let mut engine = World::new();
        let entity;
        {
            let entities_and_components = &mut engine.entities_and_components;

            entity = entities_and_components.add_entity();
            let entity_2 = entities_and_components.add_entity();

            entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
            entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

            entities_and_components.add_component_to(entity_2, Position { x: 0.0, y: 0.0 });
            entities_and_components.add_component_to(entity_2, Velocity { x: 1.0, y: 1.0 });

            engine.add_system(ParallelMovementSystem {});
        }

        for _ in 0..5 {
            engine.run();
        }

        {
            let entities_and_components = &mut engine.entities_and_components;

            let (position, velocity) =
                entities_and_components.get_components::<(Position, Velocity)>(entity);

            assert_eq!(position.x, 5.0);
            assert_eq!(position.y, 5.0);
            assert_eq!(velocity.x, 1.0);
            assert_eq!(velocity.y, 1.0);
        }
    }

    struct PrestepSystem {
        postions: Vec<Position>,
    }

    impl System for PrestepSystem {
        fn prestep(&mut self, engine: &EntitiesAndComponents) {
            self.postions.clear();

            for entity in engine.get_entities_with_component::<Position>() {
                let (position,) = engine.get_components::<(Position,)>(*entity);
                self.postions.push(position.clone());
            }
        }

        fn implements_prestep(&self) -> bool {
            true
        }

        fn run(&mut self, engine: &mut EntitiesAndComponents) {
            for position in &self.postions {
                engine.add_entity_with((position.clone(),));
            }
        }
    }

    #[test]
    fn test_prestep() {
        let mut engine = World::new();
        {
            let entities_and_components = &mut engine.entities_and_components;

            let entity = entities_and_components.add_entity();
            let entity_2 = entities_and_components.add_entity();

            entities_and_components.add_component_to(entity, Position { x: 0.0, y: 1.0 });
            entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

            entities_and_components.add_component_to(entity_2, Position { x: 1.0, y: 0.0 });
            entities_and_components.add_component_to(entity_2, Velocity { x: 1.0, y: 1.0 });

            engine.add_system(PrestepSystem {
                postions: Vec::new(),
            });
        }

        for _ in 0..1 {
            engine.run();
        }

        {
            let entities_and_components = &mut engine.entities_and_components;
            let first_added_entity = entities_and_components.get_nth_entity(0);
            let second_added_entity = entities_and_components.get_nth_entity(1);

            let (position,) =
                entities_and_components.get_components::<(Position,)>(first_added_entity.unwrap());
            let (position_2,) =
                entities_and_components.get_components::<(Position,)>(second_added_entity.unwrap());

            assert_eq!(position.x, 0.0);
            assert_eq!(position.y, 1.0);
            assert_eq!(position_2.x, 1.0);
            assert_eq!(position_2.y, 0.0);
        }
    }
}
