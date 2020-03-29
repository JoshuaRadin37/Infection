use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Index, IndexMut, Range};

use crate::game::graph::GraphError::{EdgeAlreadyExists, IdDoesNotExist, IdExists};

pub struct Node<ID = usize, T = ()> where ID : PartialEq + Copy {
    id: ID,
    value: T
}

impl <ID : PartialEq + Copy, T> Node<ID, T> {

    pub fn new(id: ID, val: T) -> Self {
        Node { id, value: val }
    }

    pub fn is_id(&self, k: &ID) -> bool {
        &self.id == k
    }

    pub fn get_id(&self) -> &ID {
        &self.id
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

    pub fn get_value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}



pub struct Graph<ID = usize, W = f64, T = ()>
    where
        ID : Eq + Hash + Copy  {
    adjacency: HashMap<ID, HashMap<ID, W>>,
    nodes: HashMap<ID, Node<ID, T>>,
    edges: Vec<(ID, ID, W)>,
    num_nodes: usize,
    num_edges: usize,
}


#[derive(Debug)]
enum GraphError<ID> {
    IdExists(ID),
    IdDoesNotExist(ID),
    EdgeAlreadyExists
}


type GraphResult<ID> = Result<(), GraphError<ID>>;

impl <ID, W, T> Graph<ID, W, T>
    where
        ID : Eq + Hash + Copy {

    fn new() -> Self {
        Graph {
            adjacency: HashMap::new(),
            nodes: HashMap::new(),
            edges: Vec::new(),
            num_nodes: 0,
            num_edges: 0
        }
    }

    fn add_node(&mut self, id: ID, value: T) -> GraphResult<ID> {
        let n = Node::new(id.clone(), value);
        if self.nodes.contains_key(n.get_id()) {
            return Err(IdExists(id));
        }

        self.nodes.insert(id, n);
        self.num_nodes += 1;
        Ok(())
    }

    fn contains_node(&self, id: ID) -> bool {
        self.nodes.contains_key(&id)
    }

    fn add_edge(&mut self, u: ID, v: ID, weight: W) -> GraphResult<ID> {
        if !self.contains_node(u) {
            return Err(IdDoesNotExist(u));
        } else if  !self.contains_node(v) {
            return Err(IdDoesNotExist(v));
        }
        let map = self.adjacency.entry(u).or_insert(HashMap::new());
        if map.contains_key(&v) {
            return Err(EdgeAlreadyExists)
        }
        map.insert(v, weight);
        Ok(())
    }

    fn contains_edge(&self, u: ID, v: ID) -> bool {
        if !self.contains_node(u) || !self.contains_node(v) {
            return false;
        }
        match self.adjacency.get(&u) {
            None => {
                false
            },
            Some(map) => {
                map.contains_key(&v)
            },
        }
    }

    fn get_weight(&self, u: ID, v: ID) -> Option<&W> {
        if !self.contains_edge(u, v) {
            None
        } else {
            self.adjacency.get(&u).unwrap().get(&v)
        }
    }

    fn get_adjacent(&self, node: ID) -> Vec<&ID> {
        match self.adjacency.get(&node) {
            None => {
                Vec::new()
            },
            Some(map) => {
                map.keys().collect()
            },
        }
    }

}

impl <ID, W, T> Graph<ID, W, T>
    where
        ID : Eq + Hash + Copy,
        T : Copy {
    fn add_nodes<I>(&mut self, id: I, value: T) -> GraphResult<ID>
        where I : Iterator<Item=ID>{
        for n in id {
            if let Err(e) = self.add_node(n , value) {
                return Err(e);
            }
        }
        Ok(())
    }
}

impl <ID, W, T> Graph<ID, W, T>
    where
        ID : Eq + Hash + Copy,
        W : Default {
    fn add_edge_default(&mut self, u: ID, v: ID) -> GraphResult<ID> {
        self.add_edge(u, v, Default::default())
    }
}

impl <ID, W, T> Index<ID> for Graph<ID, W, T>
    where
        ID : Eq + Hash + Copy,
        T : Copy {
    type Output = T;

    fn index(&self, index: ID) -> &Self::Output {
        self.nodes.get(&index).unwrap().get_value()
    }
}

impl<ID, W, T> IndexMut<ID> for Graph<ID, W, T>
    where
        ID : Eq + Hash + Copy,
        T : Copy {
    fn index_mut(&mut self, index: ID) -> &mut Self::Output {
        self.nodes.get_mut(&index).unwrap().get_value_mut()
    }
}

impl <ID, W, T> Index<(ID, ID)> for Graph<ID, W, T>
    where
        ID : Eq + Hash + Copy,
        T : Copy {
    type Output = W;

    fn index(&self, index: (ID, ID)) -> &Self::Output {
        &self.adjacency[&index.0][&index.1]
    }
}

#[cfg(test)]
mod test {
    use crate::game::graph::{Graph, Node};

    #[test]
    fn is_key_works() {
        let n: Node = Node::new(1, ());

        assert!(n.is_id(&1))
    }

    #[test]
    fn create_graph() {
        let mut g: Graph<u32> = Graph::new();
        assert_eq!(g.num_edges, 0);
        assert_eq!(g.num_nodes, 0);

        g.add_node(0, ()).unwrap();
        assert_eq!(g.num_nodes, 1);
    }

    #[test]
    fn add_range_of_ids() {
        let mut g: Graph = Graph::new();

        g.add_nodes(0..10, ()).unwrap();
        assert_eq!(g.num_nodes, 10);
    }

    #[test]
    fn set_weight() {
        let mut g: Graph = Graph::new();

        g.add_nodes(0..10, ()).unwrap();
        assert!(!g.contains_edge(1, 2));
        assert!(g.add_edge(1, 2, 10.0).is_ok());
        assert!(g.contains_edge(1, 2));
        assert_eq!(g.get_weight(1, 2).unwrap(), &10.0);
        assert!(g.get_weight(4, 5).is_none());
        assert_eq!(g[(1, 2)], 10.0)
    }

    #[test]
    fn change_value() {
        let mut g: Graph<i32, f64, i32> = Graph::new();
        g.add_nodes(0..10, 10).unwrap();
        assert_eq!(g[3], 10);
        g[3] = 15;
        assert_eq!(g[3], 15);
    }

    #[test]
    fn get_adjacent() {
        let mut g: Graph = Graph::new();

        g.add_nodes(0..10, ()).unwrap();
        g.add_edge_default(0, 1).unwrap();
        g.add_edge_default(0, 3).unwrap();
        g.add_edge_default(0, 7).unwrap();
        let mut v = g.get_adjacent(0);
        v.sort();

        assert_eq!(v, vec![&1, &3, &7]);
    }
}