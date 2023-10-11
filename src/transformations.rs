use glam::{Mat2, Vec2};

//-------------------------- Vec2DRotateAroundOrigin --------------------------
//
//  rotates a vector ang rads around the origin
//-----------------------------------------------------------------------------
pub fn Vec2DRotateAroundOrigin(v: Vec2, ang: f32) -> Vec2 {
    //create a transformation matrix
    let mat = Mat2::from_angle(ang);
    // rotate
    mat.mul_vec2(v)
}
