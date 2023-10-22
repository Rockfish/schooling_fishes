use glam::Vec2;

#[derive(Debug)]
pub struct InvertedAABBox2D {
    m_vTopLeft: Vec2,
    m_vBottomRight: Vec2,
    m_vCenter: Vec2,
}

impl InvertedAABBox2D {
    pub fn new(top_left: Vec2, bottom_right: Vec2) -> InvertedAABBox2D {
        InvertedAABBox2D {
            m_vTopLeft: top_left,
            m_vBottomRight: bottom_right,
            m_vCenter: ((top_left + bottom_right) / 2.0),
        }
    }

    //returns true if the bounding box described by other intersects with this one
    pub fn isOverlappedWith(&self, other: &InvertedAABBox2D) -> bool {
        !((other.m_vTopLeft.y > self.m_vBottomRight.y)
            || (other.m_vBottomRight.y < self.m_vTopLeft.y)
            || (other.m_vTopLeft.x > self.m_vBottomRight.x)
            || (other.m_vBottomRight.x < self.m_vTopLeft.x))
    }

    pub fn top(&self) -> f32 {
        self.m_vTopLeft.y
    }

    pub fn left(&self) -> f32 {
        self.m_vTopLeft.x
    }

    pub fn bottom(&self) -> f32 {
        self.m_vBottomRight.y
    }

    pub fn right(&self) -> f32 {
        self.m_vBottomRight.x
    }

    pub fn render(&self) {
        let _box_verts = [
            self.left(), self.top(), self.right(), self.top(),
            self.left(), self.bottom(), self.right(), self.bottom(),
            self.left(), self.top(), self.left(), self.bottom(),
            self.right(), self.top(), self.right(), self.bottom()
        ];

        todo!();
    }
}
