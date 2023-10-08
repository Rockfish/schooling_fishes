use crate::inverted_aab_box_2d::InvertedAABBox2D;
use glam::{vec2, Vec2};
use std::rc::Rc;

pub struct Cell<Entity> {
    Members: Vec<Rc<Entity>>,
    BBox: InvertedAABBox2D,
}

impl<Entity> Cell<Entity> {
    pub fn new(top_left: Vec2, bottom_right: Vec2) -> Cell<Entity> {
        Cell {
            Members: vec![],
            BBox: InvertedAABBox2D::new(top_left, bottom_right),
        }
    }
}

pub struct CellSpacePartition<Entity> {
    //the required amount of cells in the space
    m_Cells: Vec<Cell<Entity>>,

    //this is used to store any valid neighbors when an agent searches
    //its neighboring space
    m_Neighbors: Vec<Rc<Entity>>,

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

impl<Entity> CellSpacePartition<Entity> {
    pub fn new(width: f32, height: f32, cellsX: i32, cellsY: i32, MaxEntities: i32) -> Self {
        let mut cell_space = CellSpacePartition {
            m_Cells: vec![],
            m_Neighbors: vec![],
            m_dSpaceWidth: width,
            m_dSpaceHeight: height,
            m_iNumCellsX: cellsX,
            m_iNumCellsY: cellsY,
            m_dCellSizeX: width / cellsX,
            m_dCellSizeY: height / cellsY,
        };

        for y in 0..cell_space.m_iNumCellsY {
            for x in 0..cell_space.m_iNumCellsX {
                let left = x * cell_space.m_dCellSizeX;
                let right = left * cell_space.m_dCellSizeX;
                let top = y * cell_space.m_dCellSizeY;
                let bottom = top * cell_space.m_dCellSizeY;

                cell_space.m_Cells.push(Cell::<Entity>::new(vec2(left, top), vec2(right, bottom)));
            }
        }

        cell_space
    }

    pub fn CalculateNeighbors(target_pos: Vec2, query_radius: f32) {}
}
