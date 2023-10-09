use std::cell::RefCell;
use crate::inverted_aab_box_2d::InvertedAABBox2D;
use glam::{vec2, Vec2};
use std::rc::Rc;
use crate::base_entity::EntityBase;

pub struct Cell<Entity: EntityBase> {
    Members: Vec<Rc<RefCell<Entity>>>,
    BBox: InvertedAABBox2D,
}

impl<Entity: EntityBase> Cell<Entity> {
    pub fn new(top_left: Vec2, bottom_right: Vec2) -> Cell<Entity> {
        Cell {
            Members: vec![],
            BBox: InvertedAABBox2D::new(top_left, bottom_right),
        }
    }
}

pub struct CellSpacePartition<Entity: EntityBase> {
    //the required amount of cells in the space
    m_Cells: Vec<Cell<Entity>>,

    //this is used to store any valid neighbors when an agent searches
    //its neighboring space
    m_Neighbors: Vec<Rc<RefCell<Entity>>>,

    //this iterator will be used by the methods next and begin to traverse
    //through the above vector of neighbors
    // -- just iter() on m_Neighbor
    // typename std::vector<entity>::iterator m_curNeighbor;

    //the width and height of the world space the entities inhabit
    m_dSpaceWidth: f32,
    m_dSpaceHeight: f32,

    //the number of cells the space is going to be divided up into
    m_iNumCellsX: i32,
    m_iNumCellsY: i32,

    m_dCellSizeX: f32,
    m_dCellSizeY: f32,
}

impl<Entity: EntityBase> CellSpacePartition<Entity> {
    pub fn new(width: f32, height: f32, cellsX: i32, cellsY: i32, MaxEntities: i32) -> Self {
        let mut cell_space = CellSpacePartition {
            m_Cells: vec![],
            m_Neighbors: vec![],
            m_dSpaceWidth: width,
            m_dSpaceHeight: height,
            m_iNumCellsX: cellsX,
            m_iNumCellsY: cellsY,
            m_dCellSizeX: width / cellsX as f32,
            m_dCellSizeY: height / cellsY as f32,
        };

        for y in 0..cell_space.m_iNumCellsY {
            for x in 0..cell_space.m_iNumCellsX {
                let left = x as f32 * cell_space.m_dCellSizeX;
                let right = left * cell_space.m_dCellSizeX;
                let top = y as f32 * cell_space.m_dCellSizeY;
                let bottom = top * cell_space.m_dCellSizeY;

                cell_space.m_Cells.push(Cell::<Entity>::new(vec2(left, top), vec2(right, bottom)));
            }
        }

        cell_space
    }

    //--------------------- PositionToIndex ----------------------------------
    //
    //  Given a 2D vector representing a position within the game world, this
    //  method calculates an index into its appropriate cell
    //------------------------------------------------------------------------
    pub fn PositionToIndex(&self, pos: &Vec2) -> i32 {
        let idx = (self.m_iNumCellsX as f32 * pos.x / self.m_dSpaceWidth) +
            ((self.m_iNumCellsY as f32 * pos.y / self.m_dSpaceHeight) * self.m_iNumCellsX as f32);

        let mut idx = idx as i32;

        //if the entity's position is equal to Vector2D(m_dSpaceWidth, m_dSpaceHeight)
        //then the index will overshoot. We need to check for this and adjust
        if idx > self.m_Cells.len() as i32 - 1 {
            idx = self.m_Cells.len() as i32 - 1;
        }

        return idx;
    }

    pub fn AddEntity(&mut self, entity: Rc<RefCell<Entity>>) {
        let sz = self.m_Cells.len();
        let idx = self.PositionToIndex(&entity.borrow().Pos());
    }

    pub fn CalculateNeighbors(target_pos: Vec2, query_radius: f32) {}


}
