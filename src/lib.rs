struct GameEngine {
    entities: Vec<Entity>,
    components: Vec<Vec<Box<dyn Component>>>, // where components[entity_id][component_id (which shouldn't need to indexed into)]
    systems: Vec<Box<dyn System>>,
}

impl GameEngine {
    pub fn new() -> Self {
        GameEngine {
            entities: vec![],
            components: vec![],
            systems: vec![],
        }
    }

    pub fn get_components(&self, entity: Entity) -> &Vec<Box<dyn Component>> {
        self.components
            .get(entity.entity_id)
            .expect("Entity ID does not exist, was the Entity ID edited?")
    }

    // TODO: add_entity
    // how should the arguments to this function be handled?
}

pub trait Component {}

// The Entity will just be an ID that can be
// indexed into arrays of components for now...
pub struct Entity {
    entity_id: usize,
}

/// Systems access and change components on objects
pub trait System {
    fn run(&self, entities: &mut Vec<Entity>);
}

#[cfg(test)]
mod tests {}
