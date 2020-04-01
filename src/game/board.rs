use std::cmp::Ordering;

use structure::graph::Graph;

use crate::game::{AIR_TRAVEL_TIME, LAND_TRAVEL_TIME, SEA_TRAVEL_TIME};

pub struct Chunk {
    population: usize,
    size: f64,
}

pub enum Adjacency {
    Land(f64),
    Water(f64),
    Air(f64),
}

impl Adjacency {
    pub fn get_travel_time(&self) -> f64 {
        match self {
            Adjacency::Land(d) => d * LAND_TRAVEL_TIME,
            Adjacency::Water(d) => d * SEA_TRAVEL_TIME,
            Adjacency::Air(d) => d * AIR_TRAVEL_TIME,
        }
    }
}

impl PartialOrd for Adjacency {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_travel_time().partial_cmp(&other.get_travel_time())
    }
}

impl PartialEq for Adjacency {
    fn eq(&self, other: &Self) -> bool {
        self.get_travel_time() == other.get_travel_time()
    }
}

pub struct GameBoard {
    chunk_graph: Graph<usize, Adjacency, Chunk>,
}
