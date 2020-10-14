use std::hash::{Hash, Hasher};

static mut ID_COUNTER: u32 = 0;

fn generate_id() -> EntityId {
    #![allow(unused)] // due to unsafe
    let mut id = 0;
    // Not thread safe TODO: consider a better way to do this e.g. inject, or use a mutex
    unsafe {
        ID_COUNTER += 1;
        id = ID_COUNTER;
    }
    EntityId(id)
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct EntityId(u32);

#[derive(PartialEq, Clone)]
pub enum EntityKind {
    Baddie,
    Wall,
    Bullet,
    Cannon,

    // For proxies. Consider using Option if it becomes more widely used.
    UNDEFINED,
}

// TODO: Hash by id to satisfy HashSet in World  (i.e. ignore kind, if implemented) or... get rid and just use EntityId
/// An entity that exists in the world.
#[derive(Clone)]
pub struct Entity {
    id: EntityId,
    kind: EntityKind,
}

impl Hash for Entity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Entity {}

impl Entity {
    /// Creates a new entity.
    pub fn new(kind: EntityKind) -> Self {
        Self {
            id: generate_id(),
            kind
        }
    }

    /// Creates a dummy entity that can be used as a proxy for others, currently just for hashing purposes
    pub fn from_id(id: EntityId) -> Self {
        Self {
            id, kind: EntityKind::UNDEFINED
        }
    }

    /// Returns the entity's ID.
    pub fn get_id(&self) -> EntityId {
        self.id
    }

    pub fn get_kind(&self) -> &EntityKind {
        &self.kind
    }
}