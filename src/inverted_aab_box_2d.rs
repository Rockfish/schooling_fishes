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

    //returns true if the bbox described by other intersects with this one
    pub fn isOverlappedWith(&self, other: &InvertedAABBox2D) -> bool {
        !((other.m_vTopLeft.y > self.m_vBottomRight.y)
            || (other.m_vBottomRight.y < self.m_vTopLeft.y)
            || (other.m_vTopLeft.x > self.m_vBottomRight.x)
            || (other.m_vBottomRight.x < self.m_vTopLeft.x))
    }
}
