use std::f32::consts::TAU;
use glam::{Vec2, vec2};
use rand::thread_rng;
use crate::transformations::Vec2DRotateAroundOrigin;
use crate::utils::{min, RandInRange};

#[derive(Debug, Default)]
pub struct Path {
    m_WayPoints: Vec<Vec2>,

    //points to the current waypoint
    // std::list<Vector2D>::iterator  curWaypoint;
    curWaypoint: usize,

    m_bLooped: bool,
}

impl Path {

    pub fn new(
        num_way_points: i32,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
        looped: bool
    ) -> Self {
        let mut path = Path {
            m_WayPoints: vec![],
            curWaypoint: 0,
            m_bLooped: looped,
        };
        path.CreateRandomPath(num_way_points, min_x, min_y, max_x, max_y);
        path
    }

    pub fn LoopOn(&mut self) {

    }

    fn CreateRandomPath(&mut self, num_way_points: i32, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        self.m_WayPoints.clear();

        let midX = (max_x+min_x)/2.0;
        let midY = (max_y+min_y)/2.0;

        let smaller = min(midX, midY);

        let spacing = TAU/num_way_points as f32;

        for i in 0..num_way_points {
            let RadialDist = RandInRange(thread_rng(), smaller*0.2, smaller);

            let mut temp = vec2(RadialDist, 0.0);

            temp = Vec2DRotateAroundOrigin(temp, i as f32 *spacing);

            temp.x += midX;
            temp.y += midY;

            self.m_WayPoints.push(temp);
        }

        self.curWaypoint = 0; // m_WayPoints.begin();
    }
}
