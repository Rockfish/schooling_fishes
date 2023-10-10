use std::cell::RefCell;
use crate::inverted_aab_box_2d::InvertedAABBox2D;
use glam::{vec2, Vec2};
use std::rc::Rc;
use crate::base_entity::EntityBase;

pub struct Cell<Entity: EntityBase> {
    pub Members: Vec<Rc<RefCell<Entity>>>,
    pub BBox: InvertedAABBox2D,
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
            m_Neighbors: Vec::with_capacity(MaxEntities as usize),
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

    pub fn EmptyCells(&mut self) {
        for mut cell in &mut self.m_Cells {
            cell.Members.clear();
        }
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

    //----------------------- CalculateNeighbors ----------------------------
    //
    //  This must be called to create the vector of neighbors.This method
    //  examines each cell within range of the target, If the
    //  cells contain entities then they are tested to see if they are situated
    //  within the target's neighborhood region. If they are they are added to
    //  neighbor list
    //------------------------------------------------------------------------
    pub fn CalculateNeighbors(&mut self, target_pos: Vec2, query_radius: f32) {

        // index into the neighbor vector
        let mut current_neighbor: usize = 0;

        //create the query box that is the bounding box of the target's query area
        let query_box = InvertedAABBox2D::new(
            target_pos - vec2(query_radius, query_radius),
            target_pos + vec2(query_radius, query_radius),
        );

        let query_radius_squared = query_radius * query_radius;

        for cur_cell in &self.m_Cells {

            if cur_cell.BBox.isOverlappedWith(&query_box) && !cur_cell.Members.is_empty() {

                for entity in &cur_cell.Members {

                    if entity.borrow().Pos().distance_squared(target_pos) < query_radius_squared {
                        // todo: revisit probably should just clear then push. Rust vecs probably don't work like the cpp vectors
                        self.m_Neighbors[current_neighbor] = entity.clone();
                        current_neighbor += 1;
                    }
                }
            }
        }

        // TODO: clear and push instead of this
        self.m_Neighbors.truncate(current_neighbor);
    }

    //----------------------- UpdateEntity -----------------------------------
//
//  Checks to see if an entity has moved cells. If so the data structure
//  is updated accordingly
//------------------------------------------------------------------------
    pub fn UpdateEntity(&mut self, entity: &Rc<RefCell<Entity>>, OldPos: &Vec2) {
        //if the index for the old pos and the new pos are not equal then
        //the entity has moved to another cell.
        let OldIdx = self.PositionToIndex(OldPos);
        let NewIdx = self.PositionToIndex(&entity.borrow().Pos());

        if NewIdx == OldIdx {return; }

        //the entity has moved into another cell so delete from current cell
        //and add to new one
        let _ = self.m_Cells[OldIdx as usize].Members.extract_if(|e| e.borrow().ID() == entity.borrow().ID());
        self.m_Cells[NewIdx as usize].Members.push(entity.clone());
    }


}
