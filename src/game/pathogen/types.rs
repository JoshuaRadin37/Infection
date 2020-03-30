use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::rc::Rc;

use crate::game::graph::Graph;
use crate::game::pathogen::Pathogen;
use crate::game::pathogen::symptoms::{Symp, Symptom, SymptomMap, SymptomMapBuilder};
use crate::game::pathogen::symptoms::base::{Cough, RunnyNose};

pub trait PathogenType{

    /// Gets the prefix of the Pathogen Type
    fn get_prefix(&self) -> &str;


    fn get_min_count(&self) -> usize;
    fn get_mutativity(&self) -> f64;
    fn get_recovery_base_chance(&self) -> f64;
    fn get_recovery_chance_increase(&self) -> f64;
    fn get_symptoms_map(&self) -> (Graph<usize, f64, Rc<Symptom>>, HashSet<usize>);


    fn create_pathogen(&self, name: &str, mutation_ticks: usize) -> Pathogen {
        let fixed_name = format!("{} {}", self.get_prefix(), name);
        let (graph, set) = self.get_symptoms_map();

        let mut pathogen = Pathogen::new(fixed_name,
                                         self.get_min_count(),
                                         self.get_mutativity(),
                                         self.get_recovery_base_chance(),
                                         self.get_recovery_chance_increase(),
                                         graph,
                                         set);


        for _ in 0..mutation_ticks {
            pathogen = pathogen.mutate()
        }

        pathogen
    }
}

pub struct Virus;

impl PathogenType for Virus {
    fn get_prefix(&self) -> &str {
        "Virus"
    }

    fn get_min_count(&self) -> usize {
        1_000_000
    }

    fn get_mutativity(&self) -> f64 {
        0.05
    }

    fn get_recovery_base_chance(&self) -> f64 {
        0.03
    }

    fn get_recovery_chance_increase(&self) -> f64 {
        1.0
    }


    fn get_symptoms_map(&self) -> (Graph<usize, f64, Rc<Symptom>>, HashSet<usize>) {
        let mut builder = SymptomMapBuilder::new();
        let mut set = HashSet::new();

        let mut builder_entry = builder.add(RunnyNose.get_symptom());
        set.insert(builder_entry.node());
        builder_entry.next_symptom(Cough.get_symptom(), 0.02);

        (builder.get_map(), set)
    }
}

#[cfg(test)]
mod test {
    use crate::game::pathogen::types::{PathogenType, Virus};

    #[test]
    fn mutation_works() {

        let pathogen = Virus.create_pathogen("Test", 100);
        println!("{:?}", pathogen);


    }
}