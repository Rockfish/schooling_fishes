use glam::{Mat4, Vec2, vec3};

//-------------------------- Vec2DRotateAroundOrigin --------------------------
//
//  rotates a vector ang rads around the origin
//-----------------------------------------------------------------------------
pub fn Vec2DRotateAroundOrigin(v: Vec2, ang: f32)
{
//create a transformation matrix
    let mat = Mat4::from_axis_angle(vec3(v.x, v.y, 0.0), ang);

//rotate
mat.Rotate(ang);

//now transform the object's vertices
mat.TransformVector2Ds(v);
}
