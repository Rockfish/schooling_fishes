use std::default::Default;
use crate::utils::max;
use glam::{vec2, Vec2};

static mut NEXT_ID: i32 = -1;

pub enum EntityType {
    default_entity_type = -1,
}

#[derive(Debug)]
pub struct BaseEntity {
    // each entity has a unique ID
    pub id: i32,

    // every entity has a type associated with it (health, troll, ammo etc)
    pub entity_type: i32,

    // this is a generic flag.
    pub tag: bool,

    // its location in the environment
    pub position: Vec2,

    // a normalized vector pointing in the direction the entity is heading.
    pub heading: Vec2,

    // a vector perpendicular to the heading vector
    pub side_vec: Vec2,

    pub scale: Vec2,

    // the length of this object's bounding radius
    pub bounding_radius: f32,
}

impl BaseEntity {
    // pub fn new() -> Self {
    //     let heading = Vec2::default();
    //     BaseEntity {
    //         id: Self::next_valid_id(),
    //         entity_type: EntityType::default_entity_type as i32,
    //         tag: false,
    //         position: Vec2::default(),
    //         heading,
    //         side_vec: heading.perp(),
    //         scale: Default::default(),
    //         bounding_radius: 0.0,
    //     }
    // }

    // pub fn with_type(entity_type: i32) -> Self {
    //     BaseEntity {
    //         id: Self::next_valid_id(),
    //         entity_type,
    //         tag: false,
    //         position: Default::default(),
    //         heading: Default::default(),
    //         scale: Default::default(),
    //         bounding_radius: 0.0,
    //     }
    // }

    pub fn new(entity_type: i32, pos: Vec2, heading: Vec2, bounding_radius: f32) -> Self {
        BaseEntity {
            id: Self::next_valid_id(),
            entity_type,
            tag: false,
            position: pos,
            heading,
            side_vec: heading.perp(),
            scale: Default::default(),
            bounding_radius,
        }
    }

    //this can be used to create an entity with a 'forced' ID. It can be used
    //when a previously created entity has been removed and deleted from the
    //game for some reason. For example, The Raven map editor uses this ctor
    //in its undo/redo operations.
    //USE WITH CAUTION!
    // pub fn with_forced_id(entity_type: i32, forced_id: i32) -> Self {
    //     BaseEntity {
    //         id: forced_id,
    //         entity_type,
    //         tag: false,
    //         position: Default::default(),
    //         heading: Default::default(),
    //         scale: Default::default(),
    //         bounding_radius: 0.0,
    //     }
    // }

    pub fn next_valid_id() -> i32 {
        unsafe {
            NEXT_ID += 1;
            NEXT_ID
        }
    }
}

impl EntityBase for BaseEntity {
    fn id(&self) -> i32 {
        self.id
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn heading(&self) -> Vec2 {
        self.heading
    }

    fn bounding_radius(&self) -> f32 {
        self.bounding_radius
    }

    fn tag(&mut self) {
        self.tag = true;
    }

    fn untag(&mut self) {
        self.tag = false;
    }

    fn is_tagged(&self) -> bool {
        self.tag
    }

    fn scale(&self) -> Vec2 {
        self.scale
    }

    fn set_scale_vec(&mut self, val: Vec2) {
        self.bounding_radius *= max(val.x, val.y) / max(self.scale.x, self.scale.y);
        self.scale = val;
    }

    fn set_scale_float(&mut self, val: f32) {
        self.bounding_radius *= val / max(self.scale.x, self.scale.y);
        self.scale = vec2(val, val);
    }

    fn entity_type(&self) -> i32 {
        self.entity_type
    }

    fn set_entity_type(&mut self, new_type: i32) {
        self.entity_type = new_type;
    }
}

pub trait EntityBase {
    fn id(&self) -> i32;

    fn position(&self) -> Vec2;

    fn heading(&self) -> Vec2;

    fn bounding_radius(&self) -> f32;

    fn tag(&mut self);

    fn untag(&mut self);
    fn is_tagged(&self) -> bool;

    fn scale(&self) -> Vec2;

    fn set_scale_vec(&mut self, val: Vec2);

    fn set_scale_float(&mut self, val: f32);

    fn entity_type(&self) -> i32;

    fn set_entity_type(&mut self, new_type: i32);
}
