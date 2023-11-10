use glam::Vec2;

static mut NEXT_ID: i32 = -1;

pub enum EntityType {
    default_entity_type = -1,
}
pub fn next_valid_id() -> i32 {
    unsafe {
        NEXT_ID += 1;
        NEXT_ID
    }
}

pub trait EntityBase {
    fn id(&self) -> i32;

    fn entity_type(&self) -> i32;

    fn position(&self) -> Vec2;

    fn bounding_radius(&self) -> f32;

    fn tag(&mut self);

    fn untag(&mut self);
    fn is_tagged(&self) -> bool;

    fn scale(&self) -> Vec2;

    fn set_scale_vec(&mut self, val: Vec2);

    fn set_scale_float(&mut self, val: f32);

    // fn set_entity_type(&mut self, new_type: i32);
}

pub trait EntityMovable: EntityBase {
    fn mass(&self) -> f32;
    fn velocity(&self) -> Vec2;
    fn speed(&self) -> f32;
    fn heading(&self) -> Vec2;
    fn side(&self) -> Vec2;
    fn max_force(&self) -> f32;
    fn max_speed(&self) -> f32;
    fn set_max_speed(&mut self, speed: f32);
}

pub trait EntitySteerable: EntityMovable {}
// impl<T> EntitySteerable for T where T: EntityMovable {}
