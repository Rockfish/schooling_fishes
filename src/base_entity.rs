use glam::{vec2, Vec2};
use crate::utils::max;

static mut NEXT_ID: i32 = -1;

pub enum EntityType {
    default_entity_type = -1,
}

pub struct BaseGameEntity {
    //each entity has a unique ID
    pub m_ID: i32,

    //every entity has a type associated with it (health, troll, ammo etc)
    pub m_EntityType: i32,

    //this is a generic flag.
    pub m_bTag: bool,

    //its location in the environment
    pub m_vPos: Vec2,

    pub m_vScale: Vec2,

    //the length of this object's bounding radius
    pub m_dBoundingRadius: f32,
}

impl BaseGameEntity {
    pub fn new() -> Self {
        BaseGameEntity {
            m_ID: Self::next_valid_id(),
            m_EntityType: EntityType::default_entity_type as i32,
            m_bTag: false,
            m_vPos: Default::default(),
            m_vScale: Default::default(),
            m_dBoundingRadius: 0.0,
        }
    }

    pub fn with_type(entity_type: i32) -> Self {
        BaseGameEntity {
            m_ID: Self::next_valid_id(),
            m_EntityType: entity_type,
            m_bTag: false,
            m_vPos: Default::default(),
            m_vScale: Default::default(),
            m_dBoundingRadius: 0.0,
        }
    }

    pub fn with_type_and_position(entity_type: i32, pos: Vec2, r: f32) -> Self {
        BaseGameEntity {
            m_ID: Self::next_valid_id(),
            m_EntityType: entity_type,
            m_bTag: false,
            m_vPos: pos,
            m_vScale: Default::default(),
            m_dBoundingRadius: r,
        }
    }

    //this can be used to create an entity with a 'forced' ID. It can be used
    //when a previously created entity has been removed and deleted from the
    //game for some reason. For example, The Raven map editor uses this ctor
    //in its undo/redo operations.
    //USE WITH CAUTION!
    pub fn with_forced_id(entity_type: i32, forced_id: i32) -> Self {
        BaseGameEntity {
            m_ID: forced_id,
            m_EntityType: entity_type,
            m_bTag: false,
            m_vPos: Default::default(),
            m_vScale: Default::default(),
            m_dBoundingRadius: 0.0,
        }
    }

    pub fn next_valid_id() -> i32 {
        unsafe {
            NEXT_ID += 1;
            NEXT_ID
        }
    }
}

impl EntityBase for BaseGameEntity {
    fn ID(&self) -> i32 {
        self.m_ID
    }

    fn Pos(&self) -> Vec2 {
        self.m_vPos
    }

    fn BRadius(&self) -> f32 {
        self.m_dBoundingRadius
    }

    fn Tag(&mut self) {
        self.m_bTag = true;
    }

    fn UnTag(&mut self) {
        self.m_bTag = false;
    }

    fn Scale(&self) -> Vec2 {
        self.m_vScale
    }

    fn SetScale_vec(&mut self, val: Vec2) {
        self.m_dBoundingRadius *= max(val.x, val.y) / max(self.m_vScale.x, self.m_vScale.y);
        self.m_vScale = val;
    }

    fn SetScale_float(&mut self, val: f32) {
        self.m_dBoundingRadius *= val / max(self.m_vScale.x, self.m_vScale.y);
        self.m_vScale = vec2(val, val);
    }

    fn EntityType(&self) -> i32 {
        self.m_EntityType
    }

    fn SetEntityType(&mut self, new_type: i32) {
        self.m_EntityType = new_type;
    }
}

pub trait EntityBase {

    fn ID(&self) -> i32;

    fn Pos(&self) -> Vec2;  // TODO: revisit returning cloned Vec2

    fn BRadius(&self) -> f32;

    fn Tag(&mut self);

    fn UnTag(&mut self);

    fn Scale(&self) -> Vec2;

    fn SetScale_vec(&mut self, val: Vec2);

    fn SetScale_float(&mut self, val: f32);

    fn EntityType(&self) -> i32;

    fn SetEntityType(&mut self, new_type: i32);
}