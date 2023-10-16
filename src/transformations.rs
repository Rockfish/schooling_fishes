use crate::c2d_matrix::C2DMatrix;
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

//--------------------- PointToWorldSpace --------------------------------
//
//  Transforms a point from the agent's local space into world space
//------------------------------------------------------------------------
pub fn PointToWorldSpace(point: Vec2, AgentHeading: Vec2, AgentSide: Vec2, AgentPosition: Vec2) -> Vec2 {
    //make a copy of the point
    let TransPoint = point;

    //create a transformation matrix
    //C2DMatrix
    let mut matTransform = C2DMatrix::default();

    //rotate
    matTransform = matTransform.Rotate(AgentHeading, AgentSide);

    //and translate
    matTransform = matTransform.Translate(AgentPosition.x, AgentPosition.y);

    //now transform the vertices
    let TransPoint = matTransform.TransformVector2Ds(TransPoint);

    return TransPoint;
}
