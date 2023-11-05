use crate::base_entity::EntityBase;
use crate::inverted_aab_box_2d::InvertedAABBox2D;
use glam::{vec2, Vec2};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Partition<Entity: EntityBase> {
    pub members: Vec<Rc<RefCell<Entity>>>,
    pub bounding_box: InvertedAABBox2D,
}

impl<Entity: EntityBase> Partition<Entity> {
    pub fn new(top_left: Vec2, bottom_right: Vec2) -> Partition<Entity> {
        Partition {
            members: vec![],
            bounding_box: InvertedAABBox2D::new(top_left, bottom_right),
        }
    }
}

#[derive(Debug)]
pub struct CellSpacePartition<Entity: EntityBase> {
    // the required amount of cells in the space
    pub m_Cells: Vec<Partition<Entity>>,

    // this is used to store any valid neighbors when an agent searches its neighboring space
    pub m_Neighbors: Vec<Rc<RefCell<Entity>>>,

    // the width and height of the world space the entities inhabit
    m_dSpaceWidth: f32,
    m_dSpaceHeight: f32,

    // the number of cell partitions the space is going to be divided up into
    m_iNumCellsX: i32,
    m_iNumCellsY: i32,

    m_dCellSizeX: f32,
    m_dCellSizeY: f32,
}

impl<Entity: EntityBase> CellSpacePartition<Entity> {
    pub fn new(width: f32, height: f32, num_cells_x: i32, num_cells_y: i32, max_entities: i32) -> Self {
        let mut cell_space = CellSpacePartition {
            m_Cells: vec![],
            m_Neighbors: Vec::with_capacity(max_entities as usize),
            m_dSpaceWidth: width,
            m_dSpaceHeight: height,
            m_iNumCellsX: num_cells_x,
            m_iNumCellsY: num_cells_y,
            m_dCellSizeX: width / num_cells_x as f32,
            m_dCellSizeY: height / num_cells_y as f32,
        };

        for y in 0..cell_space.m_iNumCellsY {
            for x in 0..cell_space.m_iNumCellsX {
                let left = x as f32 * cell_space.m_dCellSizeX;
                let right = left * cell_space.m_dCellSizeX;
                let top = y as f32 * cell_space.m_dCellSizeY;
                let bottom = top * cell_space.m_dCellSizeY;

                cell_space.m_Cells.push(Partition::<Entity>::new(vec2(left, top), vec2(right, bottom)));
            }
        }

        cell_space
    }

    pub fn EmptyCells(&mut self) {
        for cell in &mut self.m_Cells {
            cell.members.clear();
        }
    }

    //--------------------- PositionToIndex ----------------------------------
    //
    //  Given a 2D vector representing a position within the game world, this
    //  method calculates an index into its appropriate cell
    //------------------------------------------------------------------------
    pub fn position_to_index(&self, position: &Vec2) -> usize {
        let idx = (self.m_iNumCellsX as f32 * position.x / self.m_dSpaceWidth)
            + ((self.m_iNumCellsY as f32 * position.y / self.m_dSpaceHeight) * self.m_iNumCellsX as f32);

        let mut idx = idx as usize;

        // if the entity's position is equal to Vector2D(m_dSpaceWidth, m_dSpaceHeight)
        // then the index will overshoot. We need to check for this and adjust
        if idx > self.m_Cells.len() - 1 {
            idx = self.m_Cells.len() - 1;
        }

        idx
    }

    pub fn add_entity(&mut self, entity: Rc<RefCell<Entity>>) {
        let sz = self.m_Cells.len();
        let idx = self.position_to_index(&entity.borrow().position()) as usize;
        assert!(idx < sz);
        self.m_Cells[idx].members.push(entity);
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
        self.m_Neighbors.clear();

        // create the query box that is the bounding box of the target's query area
        let query_box = InvertedAABBox2D::new(
            target_pos - vec2(query_radius, query_radius),
            target_pos + vec2(query_radius, query_radius),
        );

        let query_radius_squared = query_radius * query_radius;

        for cur_cell in &self.m_Cells {
            if cur_cell.bounding_box.isOverlappedWith(&query_box) && !cur_cell.members.is_empty() {
                for entity in &cur_cell.members {
                    if entity.borrow().position().distance_squared(target_pos) < query_radius_squared {
                        self.m_Neighbors.push(entity.clone());
                        current_neighbor += 1;
                    }
                }
            }
        }
    }

    //----------------------- UpdateEntity -----------------------------------
    //
    //  Checks to see if an entity has moved cells. If so the data structure
    //  is updated accordingly
    //------------------------------------------------------------------------
    pub fn UpdateEntity(&mut self, entity: &Rc<RefCell<Entity>>, old_position: &Vec2) {
        // if the index for the old pos and the new pos are not equal then
        // the entity has moved to another cell.
        let old_idx = self.position_to_index(old_position);
        let new_idx = self.position_to_index(&entity.borrow().position());

        if new_idx == old_idx {
            return;
        }

        // the entity has moved into another cell so remove it from current cell and add to new one
        if let Some(member_index) = self.m_Cells[old_idx]
            .members
            .iter()
            .position(|member| member.borrow().id() == entity.borrow().id())
        {
            self.m_Cells[old_idx].members.remove(member_index);
        }

        self.m_Cells[new_idx].members.push(entity.clone());
    }

    pub fn render_cells(&self) {
        for cell in &self.m_Cells {
            cell.bounding_box.render();
        }
    }
}
