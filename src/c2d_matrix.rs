//------------------------------------------------------------------------
//
//  Name:   C2DMatrix.h
//
//  Original Author: Mat Buckland 2002
//
//  Desc:   2D Matrix class
//
//------------------------------------------------------------------------

use glam::{vec2, Vec2};

#[derive(Debug)]
pub struct C2DMatrix {
    _11: f32,
    _12: f32,
    _13: f32,
    _21: f32,
    _22: f32,
    _23: f32,
    _31: f32,
    _32: f32,
    _33: f32,
}

impl C2DMatrix {

    pub fn identity() -> Self {
        C2DMatrix {
            _11: 1.0,
            _12: 0.0,
            _13: 0.0,
            _21: 0.0,
            _22: 1.0,
            _23: 0.0,
            _31: 0.0,
            _32: 0.0,
            _33: 1.0,
        }
    }

    pub fn MatrixMultiply(&self, mIn: C2DMatrix) -> C2DMatrix {
        let new_mat = C2DMatrix {
            //first row
            _11: (self._11 * mIn._11) + (self._12 * mIn._21) + (self._13 * mIn._31),
            _12: (self._11 * mIn._12) + (self._12 * mIn._22) + (self._13 * mIn._32),
            _13: (self._11 * mIn._13) + (self._12 * mIn._23) + (self._13 * mIn._33),

            //second
            _21: (self._21 * mIn._11) + (self._22 * mIn._21) + (self._23 * mIn._31),
            _22: (self._21 * mIn._12) + (self._22 * mIn._22) + (self._23 * mIn._32),
            _23: (self._21 * mIn._13) + (self._22 * mIn._23) + (self._23 * mIn._33),

            //third
            _31: (self._31 * mIn._11) + (self._32 * mIn._21) + (self._33 * mIn._31),
            _32: (self._31 * mIn._12) + (self._32 * mIn._22) + (self._33 * mIn._32),
            _33: (self._31 * mIn._13) + (self._32 * mIn._23) + (self._33 * mIn._33),
        };

        new_mat
    }

    pub fn Rotate(&self, fwd: Vec2, side: Vec2) -> C2DMatrix {
        let mat = C2DMatrix {
            _11: fwd.x,
            _12: fwd.y,
            _13: 0.0,
            _21: side.x,
            _22: side.y,
            _23: 0.0,
            _31: 0.0,
            _32: 0.0,
            _33: 1.0,
        };

        //and multiply
        self.MatrixMultiply(mat)
    }

    pub(crate) fn Translate(&self, x: f32, y: f32) -> C2DMatrix {
        let mat = C2DMatrix {
            _11: 1.0,
            _12: 0.0,
            _13: 0.0,
            _21: 0.0,
            _22: 1.0,
            _23: 0.0,
            _31: x,
            _32: y,
            _33: 1.0,
        };

        //and multiply
        self.MatrixMultiply(mat)
    }

    pub(crate) fn TransformVector2Ds(&self, vPoint: Vec2) -> Vec2 {
        let tempX = (self._11 * vPoint.x) + (self._21 * vPoint.y) + (self._31);
        let tempY = (self._12 * vPoint.x) + (self._22 * vPoint.y) + (self._32);
        vec2(tempX, tempY)
    }
}

/*
C2DMatrix()
{
//initialize the matrix to an identity matrix
Identity();
}

//create an identity matrix
inline void Identity();

//create a transformation matrix
inline void Translate(float x, float y);

//create a scale matrix
inline void Scale(float xScale, float yScale);

//create a rotation matrix
inline void  Rotate(float rotation);

//create a rotation matrix from a fwd and side 2D vector
inline void  Rotate(const Vector2D &fwd, const Vector2D &side);

//applys a transformation matrix to a std::vector of points
inline void TransformVector2Ds(std::vector<Vector2D> &vPoints);

//applys a transformation matrix to a point
inline void TransformVector2Ds(Vector2D &vPoint);

//accessors to the matrix elements
void _11(float val){self._11 = val;}
void _12(float val){self._12 = val;}
void _13(float val){self._13 = val;}

void _21(float val){self._21 = val;}
void _22(float val){self._22 = val;}
void _23(float val){self._23 = val;}

void _31(float val){self._31 = val;}
void _32(float val){self._32 = val;}
void _33(float val){self._33 = val;}

};

//multiply two matrices together
inline void C2DMatrix::MatrixMultiply(Matrix &mIn)
{
C2DMatrix::Matrix mat_temp;

//first row
mat_temp._11 = (self._11*mIn._11) + (self._12*mIn._21) + (self._13*mIn._31);
mat_temp._12 = (self._11*mIn._12) + (self._12*mIn._22) + (self._13*mIn._32);
mat_temp._13 = (self._11*mIn._13) + (self._12*mIn._23) + (self._13*mIn._33);

//second
mat_temp._21 = (self._21*mIn._11) + (self._22*mIn._21) + (self._23*mIn._31);
mat_temp._22 = (self._21*mIn._12) + (self._22*mIn._22) + (self._23*mIn._32);
mat_temp._23 = (self._21*mIn._13) + (self._22*mIn._23) + (self._23*mIn._33);

//third
mat_temp._31 = (self._31*mIn._11) + (self._32*mIn._21) + (self._33*mIn._31);
mat_temp._32 = (self._31*mIn._12) + (self._32*mIn._22) + (self._33*mIn._32);
mat_temp._33 = (self._31*mIn._13) + (self._32*mIn._23) + (self._33*mIn._33);

self = mat_temp;
}

//applies a 2D transformation matrix to a std::vector of Vector2Ds
inline void C2DMatrix::TransformVector2Ds(std::vector<Vector2D> &vPoint)
{
for (unsigned int i=0; i<vPoint.size(); ++i)
{
float tempX =(self._11*vPoint[i].x) + (self._21*vPoint[i].y) + (self._31);

float tempY = (self._12*vPoint[i].x) + (self._22*vPoint[i].y) + (self._32);

vPoint[i].x = tempX;

vPoint[i].y = tempY;
}
}

//applies a 2D transformation matrix to a single Vector2D
inline void C2DMatrix::TransformVector2Ds(Vector2D &vPoint)
{
float tempX =(self._11*vPoint.x) + (self._21*vPoint.y) + (self._31);

float tempY = (self._12*vPoint.x) + (self._22*vPoint.y) + (self._32);

vPoint.x = tempX;

vPoint.y = tempY;
}

//create an identity matrix
inline void C2DMatrix::Identity()
{
self._11 = 1; self._12 = 0; self._13 = 0;

self._21 = 0; self._22 = 1; self._23 = 0;

self._31 = 0; self._32 = 0; self._33 = 1;
}

//create a transformation matrix
inline void C2DMatrix::Translate(float x, float y)
{
Matrix mat;

mat._11 = 1; mat._12 = 0; mat._13 = 0;

mat._21 = 0; mat._22 = 1; mat._23 = 0;

mat._31 = x;    mat._32 = y;    mat._33 = 1;

//and multiply
MatrixMultiply(mat);
}

//create a scale matrix
inline void C2DMatrix::Scale(float xScale, float yScale)
{
C2DMatrix::Matrix mat;

mat._11 = xScale; mat._12 = 0; mat._13 = 0;

mat._21 = 0; mat._22 = yScale; mat._23 = 0;

mat._31 = 0; mat._32 = 0; mat._33 = 1;

//and multiply
MatrixMultiply(mat);
}


//create a rotation matrix
inline void C2DMatrix::Rotate(float rot)
{
C2DMatrix::Matrix mat;

float Sin = sin(rot);
float Cos = cos(rot);

mat._11 = Cos;  mat._12 = Sin; mat._13 = 0;

mat._21 = -Sin; mat._22 = Cos; mat._23 = 0;

mat._31 = 0; mat._32 = 0;mat._33 = 1;

//and multiply
MatrixMultiply(mat);
}


//create a rotation matrix from a 2D vector
inline void C2DMatrix::Rotate(const Vector2D &fwd, const Vector2D &side)
{
C2DMatrix::Matrix mat;

mat._11 = fwd.x;  mat._12 = fwd.y; mat._13 = 0;

mat._21 = side.x; mat._22 = side.y; mat._23 = 0;

mat._31 = 0; mat._32 = 0;mat._33 = 1;

//and multiply
MatrixMultiply(mat);
}

 */
