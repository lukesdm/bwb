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
pub struct Entity {
    id: EntityId,
    kind: EntityKind,
}

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
            id, kind: Kind::UNDEFINED
        }
    }
}