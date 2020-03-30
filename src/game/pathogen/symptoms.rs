use std::fmt::{Debug, Error, Formatter, Result};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::usize;

use crate::game::graph::{Graph, GraphResult, Node};
use crate::game::population::Person;

pub struct Symptom {
    name: String,
    description: String,
    catch_chance_increase: f64, // percentage increase
    severity_increase: f64, // percentage increase
    fatality_increase: f64, // percentage increase
    internal_spread_rate_increase: f64, // percentage increase
    recovery_chance_base: Option<f64>,
    additional_effect: Option<fn()>,
    recovery_function: Option<Arc<dyn Fn(&mut Person) + Send + Sync>>
}

impl Symptom {

    pub fn new(name: String,
               description: String,
               catch_chance_increase: f64,
               severity_increase: f64,
               fatality_increase: f64,
               internal_spread_rate_increase: f64,
               recovery_chance_base: Option<f64>,
               additional_effect: Option<fn()>,
               recovery_function: Option<&Arc<dyn Fn(&mut Person) + Send + Sync>>)
               -> Self {
        Symptom {
            name,
            description,
            catch_chance_increase,
            severity_increase,
            fatality_increase,
            internal_spread_rate_increase,
            recovery_chance_base,
            additional_effect: match additional_effect {
                None => { None },
                Some(f) => { Some(f)},
            },
            recovery_function: recovery_function.map(|f| f.clone())
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_description(&self) -> &String {
        &self.description
    }

    pub fn get_catch_chance_increase(&self) -> f64 {
        self.catch_chance_increase
    }

    pub fn get_severity_increase(&self) -> f64 {
        self.severity_increase
    }

    pub fn get_fatality_increase(&self) -> f64 {
        self.fatality_increase
    }

    pub fn get_internal_spread_rate_increase(&self) -> f64 {
        self.internal_spread_rate_increase
    }

    pub fn get_recovery_chance_base(&self) -> &Option<f64> {
        &self.recovery_chance_base
    }

    pub fn can_reverse(&self) -> bool {
        self.additional_effect.is_none() && self.recovery_chance_base.is_none()
    }

    pub fn additional_effect(&self) {
        match &self.additional_effect {
            None => {},
            Some(b) => { b() },
        }
    }


    pub fn get_recovery_effect(&self) -> &Option<Arc<dyn Fn(&mut Person) + Send + Sync>> {
        &self.recovery_function
    }

}

pub trait Symp {
    fn get_symptom(&self) -> Symptom;
}



pub trait SymptomMap {
    fn get_map(self) -> Graph<usize, f64, Arc<Symptom>>;

    fn new() -> Graph<usize, f64, Arc<Symptom>> {
        Graph::new()
    }
}

impl SymptomMap for Graph<usize, f64, Arc<Symptom>> {
    fn get_map(self) -> Graph<usize, f64, Arc<Symptom>> {
        self
    }
}

impl Debug for Symptom {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.name)
    }
}

/// Enables easy creation of Symptoms and a Symptom Map
/// > Handles the creation of RC pointers and ids
pub struct SymptomMapBuilder {
    symptoms_map: Graph<usize, f64, Arc<Symptom>>,
    symptoms: Vec<Arc<Symptom>>,
    next_id: usize
}



impl SymptomMapBuilder {
    pub fn new() -> Self {
        Self {
            symptoms_map: Graph::new(),
            symptoms: vec![],
            next_id: 0
        }
    }

    fn get_next_id(&mut self) -> usize {
        if self.next_id == std::usize::MAX {
            panic!("Reached Maximum Symptoms")
        }
        let output = self.next_id;
        self.next_id += 1;
        output
    }

    pub fn add(&mut self, symptom: Symptom) -> SymptomMapBuilderEntry {
        let id = self.get_next_id();
        let rc_ptr = Arc::new(symptom);
        self.symptoms.push(rc_ptr.clone());
        self.symptoms_map.add_node(id, rc_ptr).unwrap();
        SymptomMapBuilderEntry::new(id, self)
    }

    pub fn push(&mut self, symptom: Symptom) -> usize {
        let id = self.get_next_id();
        let rc_ptr = Arc::new(symptom);
        self.symptoms.push(rc_ptr.clone());
        self.symptoms_map.add_node(id, rc_ptr).unwrap();
        id
    }

    pub fn add_next_symptom(&mut self, id1: usize, id2: usize, mutation_chance: f64) -> GraphResult<usize> {
        self.symptoms_map.add_edge(id1, id2, mutation_chance)
    }
}

impl SymptomMap for SymptomMapBuilder {
    fn get_map(self) -> Graph<usize, f64, Arc<Symptom>> {
        self.symptoms_map
    }
}

pub struct SymptomMapBuilderEntry<'a> {
    node: usize,
    map_builder: &'a mut SymptomMapBuilder
}

impl <'a> SymptomMapBuilderEntry<'a> {
    fn new(node: usize, map_builder: &'a mut SymptomMapBuilder) -> Self {
        SymptomMapBuilderEntry{ node, map_builder }
    }

    pub fn node(&self) -> usize {
        self.node
    }

    pub fn next_symptom(&mut self, symptom: Symptom, mutation_chance: f64) -> SymptomMapBuilderEntry {
        let output = self.map_builder.add(symptom);
        let id1 = self.node;
        let id2 = output.node;
        output.map_builder.add_next_symptom(id1, id2, mutation_chance).expect("Should not fail");
        output
    }

    pub fn add_next_symptoms(&mut self, symptoms: Vec<(Symptom, f64)>) -> Vec<usize> {
        let mut output = Vec::new();
        for (symptom, mutation_chance) in symptoms {
            let next = self.map_builder.push(symptom);
            self.map_builder.add_next_symptom(self.node, next, mutation_chance);
            output.push(next);
        }
        output
    }

}

pub mod base {
    use std::sync::Arc;

    use crate::game::pathogen::symptoms::{Symp, Symptom};
    use crate::game::population::Person;

    /// Person can never recover
    pub struct Undying;
    impl Symp for Undying {
        fn get_symptom(&self) -> Symptom {
            Symptom::new(
                "Immunity Immunity".to_string(),
                "The immune system can never beat the pathogen, and the person will never recover".to_string(),
                100.0,
                1.0001,
                1.0,
                1.0,
                Some(0.0),
                None,
                None
            )
        }
    }

    pub fn create_recovery_function<'a, F>(function: F) -> Arc<dyn Fn(&'a mut Person) + Send + Sync + 'a>
        where F : Fn(&'a mut Person) + Send + Sync + 'a {
        let output: Arc<dyn Fn(&'a mut Person) + Send + Sync + 'a> = Arc::new(function);
        output
    }

    // Person are never immune to the Pathogen by forcing the Person to remove their infection
    pub struct NeverImmune;
    impl Symp for NeverImmune {
        fn get_symptom(&self) -> Symptom {

            let function: Arc<dyn Fn(&mut Person) + Send + Sync> = Arc::new(
                |person| {
                    person.remove_immunity();
                }
            );


            Symptom::new(
                "Immunity Immunity".to_string(),
                "The immune system can never beat the pathogen, and the person will never recover".to_string(),
                100.0,
                1.0001,
                1.0,
                1.0,
                Some(0.0),
                None,
                Some(
                    &function
                )
            )

        }
    }

    pub struct RunnyNose;
    impl Symp for RunnyNose {
        fn get_symptom(&self) -> Symptom {
            Symptom::new(
                "A Runny Nose".to_string(),
                "Some serious leakage problems".to_string(),
                100.0,
                1.0001,
                1.0,
                1.0,
                None,
                None,
                None
            )
        }
    }

    pub struct Cough;
    impl Symp for Cough {
        fn get_symptom(&self) -> Symptom {
            Symptom::new(
                "Cough".to_string(),
                "A upper respiratory cough".to_string(),
                100.0,
                1.1,
                1.0,
                1.0,
                None,
                None,
                None
            )
        }
    }
}

#[cfg(test)]
mod test {
    use crate::game::pathogen::symptoms::base::NeverImmune;
    use crate::game::pathogen::symptoms::Symp;
    use crate::game::pathogen::types::{PathogenType, Virus};

    #[test]
    fn never_immune_removes_immunity() {
        let mut p = Virus.create_pathogen("Test", 0);
        p.acquire_symptom(&NeverImmune.get_symptom(), None);


    }
}

