use glam::Vec2;

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
}
