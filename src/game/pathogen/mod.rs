use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Error, Formatter, Result};
use std::io::Read;
use std::rc::Rc;
use std::sync::Arc;

use rand::Rng;

use crate::game::graph::Graph;
use crate::game::pathogen::symptoms::{Symptom, SymptomMap};
use crate::game::population::Person;
use crate::game::time::{Time, TimeUnit};

pub mod infection;
pub mod symptoms;
pub mod types;




pub struct Pathogen {
    name: String, // name of the pathogen
    catch_chance: f64, // chance spreads per interaction
    severity: f64, // chance will go to doctor
    fatality: f64, // chance hp reduction
    internal_spread_rate: f64, // chance amount of pathogen increases
    min_count_for_symptoms: usize, // minimum amount of pathogens for spread, be discovered, be fatal, and to recover
    mutation: f64, // chance on new infection the pathogen mutates
    recovery_chance_base: f64, // chance of recovery after TimeUnit (converted to Minutes)
    recovery_chance_increase: f64, // change of recovery over time
    symptoms_map: Graph<usize, f64, Arc<Symptom>>, // map of possible symptoms that a pathogen can have
    acquired_map: HashSet<usize>, // the set of acquired symptoms
    on_recover: Vec<Arc<dyn Fn(&mut Person) + Send + Sync>>, // a vector of functions that affect a person after recovery
    recover_function_position: HashMap<usize, usize> // map of a symptoms ID to it's recovery function
}

impl Debug for Pathogen {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Pathogen {}", self.name)
    }
}


impl Pathogen {


    pub fn new<R>(
        name: String,
        min_count_for_symptoms: usize,
        mutation: f64,
        recovery_chance_base: f64,
        recovery_chance_increase: f64,
        symptoms_map: R,
        acquired: HashSet<usize>
    ) -> Self
        where
            R : SymptomMap
    {


        let mut pathogen = Pathogen {
            name,
            catch_chance: 0.000001,
            severity: 0.000001,
            fatality: 0.000001,
            internal_spread_rate: 0.01,
            min_count_for_symptoms,
            mutation,
            recovery_chance_base,
            recovery_chance_increase,
            symptoms_map: symptoms_map.get_map(),
            acquired_map: acquired.clone(),
            on_recover: Vec::new(),
            recover_function_position: Default::default()
        };

        for ref node in acquired {
            let symptom = &*pathogen.symptoms_map.get(node).unwrap().clone();
            pathogen.acquire_symptom(symptom);
        }
        pathogen
    }

    pub fn roll(chance: f64) -> bool {
        rand::thread_rng().gen_bool(chance)
    }

    pub fn get_acquired(&self) -> Vec<&usize> {
        self.acquired_map.iter().map(|i| i).collect()
    }

    /// Gets a list of the id of non acquired node ids and the weight for a mutation to get them
    pub fn get_potential_gains(&self) -> Vec<(&usize, f64)> {
        let acquired = self.get_acquired();
        let mut output = Vec::new();

        for id in &acquired {
            for to_id in self.symptoms_map.get_adjacent(**id) {
                if !acquired.contains(&to_id) {
                    let weight = *self.symptoms_map.get_weight(**id, *to_id).unwrap();
                    output.push((to_id, weight));
                }
            }
        }

        output
    }

    fn sum_weights_onto_node(&self, id: &usize) -> f64 {
        let mut output = 0.0;

        for (u, v) in self.symptoms_map.edges() {
            if id == v {
                output += *self.symptoms_map.get_weight(*u, *v).unwrap();
            }
        }

        output
    }

    pub fn get_potential_losses(&self) -> Vec<(&usize, f64)> {
        let acquired = self.get_acquired();
        let mut output = Vec::new();

        for id in &acquired {
            let acquired_leaf = self.symptoms_map.get_adjacent(**id).into_iter().map(|id| {
                !acquired.contains(&id)
            }).fold(true, |b, item| b && item);

            if acquired_leaf && self.symptoms_map.get(*id).unwrap().can_reverse() {
                output.push((*id, self.sum_weights_onto_node(*id)));
            }
        }

        output
    }

    pub fn acquire_symptom(&mut self, symptom: &Symptom) {
        self.catch_chance *= symptom.get_catch_chance_increase();
        self.severity *= symptom.get_severity_increase();
        self.fatality *= symptom.get_fatality_increase();
        self.internal_spread_rate *= symptom.get_severity_increase();
        if let Some(base) = symptom.get_recovery_chance_base() {
            self.recovery_chance_base = *base;
        }
        symptom.additional_effect()
    }

    pub fn remove_symptom(&mut self, symptom: &Symptom) {
        self.catch_chance /= symptom.get_catch_chance_increase();
        self.severity /= symptom.get_severity_increase();
        self.fatality /= symptom.get_fatality_increase();
        self.internal_spread_rate /= symptom.get_severity_increase();
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn catch_chance(&self) -> &f64 {
        &self.catch_chance
    }

    pub fn severity(&self) -> &f64 {
        &self.severity
    }

    pub fn fatality(&self) -> &f64 {
        &self.fatality
    }

    fn recover_chance(&self, passed: TimeUnit) -> f64 {
        let days = usize::from(passed.into_days()) as f64;
        days * days * self.recovery_chance_increase * self.recovery_chance_base / (24.0 * 60.0)
    }

    fn add_recovery_symptom<F>(&mut self, function: F)
    where F : 'static + Fn(&mut Person) + Send + Sync {
        self.on_recover.push(Arc::new(function))
    }

    fn perform_recovery(&self, person: &mut Person) {
        for funcs in &self.on_recover {
            funcs(person)
        }
    }

    pub fn mutate(&self) -> Self {

        let mut mutated_graph = self.symptoms_map.clone();
        let mut next_pathogen = Pathogen {
            name: self.name.clone(),
            catch_chance: self.catch_chance,
            severity: self.severity,
            fatality: self.fatality,
            internal_spread_rate: self.internal_spread_rate,
            min_count_for_symptoms: self.min_count_for_symptoms,
            mutation: self.mutation,
            recovery_chance_base: self.recovery_chance_base,
            recovery_chance_increase: self.recovery_chance_increase,
            symptoms_map: mutated_graph,
            acquired_map: self.acquired_map.clone(),
            on_recover: self.on_recover.clone(),
            recover_function_position: self.recover_function_position.clone()
        };


        let potential_gains = self.get_potential_gains();

        for (id, chance) in potential_gains {
            if Self::roll(chance) && !next_pathogen.acquired_map.contains(id) {
                next_pathogen.acquire_symptom(self.symptoms_map.get(id).unwrap().clone().borrow_mut());
                next_pathogen.acquired_map.insert(*id);
            }
        }

        let potential_losses = self.get_potential_losses();

        for (id, chance) in potential_losses {
            if Self::roll(chance) && next_pathogen.acquired_map.contains(id) {
                next_pathogen.remove_symptom(self.symptoms_map.get(id).unwrap().clone().borrow_mut());
                next_pathogen.acquired_map.remove(id);
            }
        }

        next_pathogen
    }
}
