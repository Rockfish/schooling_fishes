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
    let mut matTransform = C2DMatrix::identity();

    //rotate
    matTransform = matTransform.Rotate(AgentHeading, AgentSide);

    //and translate
    matTransform = matTransform.Translate(AgentPosition.x, AgentPosition.y);

    //now transform the vertices
    let TransPoint = matTransform.TransformVector2Ds(TransPoint);

    return TransPoint;
}

/*
//--------------------------- WorldTransform -----------------------------
//
//  given a std::vector of 2D vectors, a position and  orientation
//  this function transforms the 2D vectors into the object's world space
//------------------------------------------------------------------------
inline std::vector<Vector2D> WorldTransform(std::vector<Vector2D> &points,
											const Vector2D   &pos,
											const Vector2D   &forward,
											const Vector2D   &side)
{
	//copy the original vertices into the buffer about to be transformed
	std::vector<Vector2D> TranVector2Ds = points;

	//create a transformation matrix
	C2DMatrix matTransform;

	//rotate
	matTransform.Rotate(forward, side);

	//and translate
	matTransform.Translate(pos.x, pos.y);

	//now transform the object's vertices
	matTransform.TransformVector2Ds(TranVector2Ds);

	return TranVector2Ds;
}
 */