use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Error, Formatter, Result};
use std::io::Read;
use std::rc::Rc;
use std::sync::Arc;

use rand::Rng;

use structure::graph::Graph;
use structure::time::{Time, TimeUnit};
use structure::time::TimeUnit::{Days, Hours};

use crate::game::pathogen::symptoms::{Symptom, SymptomMap};
use crate::game::population::Person;
use crate::game::roll;

pub mod infection;
pub mod symptoms;
pub mod types;

#[derive(Clone)]
pub struct Pathogen {
    name: String,                                            // name of the pathogen
    catch_chance: f64,                                       // chance spreads per interaction
    severity: f64,                                           // chance will go to doctor
    fatality: f64,                                           // chance hp reduction
    internal_spread_rate: f64,                               // chance amount of pathogen increases
    min_count_for_symptoms: usize, // minimum amount of pathogens for spread, be discovered, be fatal, and to recover
    mutation: f64,                 // chance on new infection the pathogen mutates
    average_recovery_time: usize,  // in minutes
    base_recovery_distance: usize, // in minutes, represents the base range for recovery
    symptoms_map: Graph<usize, f64, Arc<Symptom>>, // map of possible symptoms that a pathogen can have
    acquired_map: HashSet<usize>,                  // the set of acquired symptoms
    on_recover: Vec<Arc<dyn Fn(&mut Person) + Send + Sync>>, // a vector of functions that affect a person after recovery
    recover_function_position: HashMap<usize, usize>, // map of a symptoms ID to it's recovery function
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
        average_recovery_time: usize, // in minutes
        base_recovery_distance: usize,
        symptoms_map: R,
        acquired: HashSet<usize>,
    ) -> Self
    where
        R: SymptomMap,
    {
        let mut pathogen = Pathogen {
            name,
            catch_chance: 0.999999,
            severity: 0.9999,
            fatality: 0.9999999999999,
            internal_spread_rate: 0.99,
            min_count_for_symptoms,
            mutation: 1.0 - mutation,
            average_recovery_time, // in minutes
            base_recovery_distance,
            symptoms_map: symptoms_map.get_map(),
            acquired_map: acquired.clone(),
            on_recover: Vec::new(),
            recover_function_position: Default::default(),
        };

        for ref node in acquired {
            let symptom = &*pathogen.symptoms_map.get(node).unwrap().clone();
            pathogen.acquire_symptom(symptom, Some(*node));
        }
        pathogen
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
            let acquired_leaf = self
                .symptoms_map
                .get_adjacent(**id)
                .into_iter()
                .map(|id| !acquired.contains(&id))
                .fold(true, |b, item| b && item);

            if acquired_leaf && self.symptoms_map.get(*id).unwrap().can_reverse() {
                output.push((*id, self.sum_weights_onto_node(*id)));
            }
        }

        output
    }

    pub fn acquire_symptom(&mut self, symptom: &Symptom, symptom_id: Option<usize>) {
        self.catch_chance *= 1.0 - symptom.get_catch_chance_increase() / 100.0;
        self.severity *= 1.0 - symptom.get_severity_increase() / 100.0;
        self.fatality *= 1.0 - symptom.get_fatality_increase() / 100.0;
        self.internal_spread_rate *= 1.0 - symptom.get_internal_spread_rate_increase() / 100.0;
        if let Some(base) = symptom.get_duration_change() {
            if base.is_infinite() {
                self.average_recovery_time = std::usize::MAX;
            } else {
                self.average_recovery_time = (self.average_recovery_time as f64 * *base) as usize;
            }
        }
        if let Some(spread) = symptom.get_spread_change() {
            self.base_recovery_distance = (self.base_recovery_distance as f64 * *spread) as usize;
        }
        if let Some(function) = symptom.get_recovery_effect() {
            let index = self.on_recover.len();
            self.on_recover.push((*function).clone());
            if let Some(id) = symptom_id {
                self.recover_function_position.insert(id, index);
            }
        }
        symptom.additional_effect()
    }

    pub fn remove_symptom(&mut self, symptom: &Symptom, symptom_id: Option<usize>) {
        self.catch_chance /= 1.0 - symptom.get_catch_chance_increase() / 100.0;
        self.severity /= 1.0 - symptom.get_severity_increase() / 100.0;
        self.fatality /= 1.0 - symptom.get_fatality_increase() / 100.0;
        self.internal_spread_rate /= 1.0 - symptom.get_internal_spread_rate_increase() / 100.0;

        if let Some(base) = symptom.get_duration_change() {
            self.average_recovery_time = (self.average_recovery_time as f64 / *base) as usize;
        }
        if let Some(spread) = symptom.get_spread_change() {
            self.base_recovery_distance = (self.base_recovery_distance as f64 / *spread) as usize;
        }

        if let Some(id) = symptom_id {
            if self.recover_function_position.contains_key(&id) {
                self.on_recover.remove(id);
                self.recover_function_position.remove(&id);
            }
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn catch_chance(&self) -> f64 {
        1.0 - self.catch_chance
    }

    pub fn severity(&self) -> f64 {
        1.0 - self.severity
    }

    pub fn fatality(&self) -> f64 {
        1.0 - self.fatality
    }

    pub fn average_recovery_time(&self) -> usize {
        self.average_recovery_time
    }

    pub fn base_recovery_distance(&self) -> usize {
        self.base_recovery_distance
    }

    pub fn internal_spread_rate(&self) -> f64 {
        1.0 - self.internal_spread_rate
    }

    fn add_recovery_symptom<F>(&mut self, function: F)
    where
        F: 'static + Fn(&mut Person) + Send + Sync,
    {
        self.on_recover.push(Arc::new(function))
    }

    pub fn perform_recovery(&self, person: &mut Person) {
        for funcs in &self.on_recover {
            funcs(person)
        }
    }

    pub fn mutate(&self) -> Self {
        let mut next_pathogen = self.clone();

        let potential_gains = self.get_potential_gains();

        for (id, chance) in potential_gains {
            if roll(chance) && !next_pathogen.acquired_map.contains(id) {
                next_pathogen.acquire_symptom(
                    self.symptoms_map.get(id).unwrap().clone().borrow_mut(),
                    Some(*id),
                );
                next_pathogen.acquired_map.insert(*id);
            }
        }

        let potential_losses = self.get_potential_losses();

        for (id, chance) in potential_losses {
            if roll(chance) && next_pathogen.acquired_map.contains(id) {
                next_pathogen.remove_symptom(
                    self.symptoms_map.get(id).unwrap().clone().borrow_mut(),
                    Some(*id),
                );
                next_pathogen.acquired_map.remove(id);
            }
        }

        next_pathogen
    }
}

impl Default for Pathogen {
    fn default() -> Self {
        Pathogen::new(
            "Testogen".to_string(),
            100000000,
            0.0005,
            usize::from((Days(4) + Hours(12)).into_minutes()),
            usize::from((Days(1) + Hours(12)).into_minutes()),
            Graph::new(),
            HashSet::new(),
        )
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use crate::game::Age;
    use crate::game::pathogen::Pathogen;
    use crate::game::pathogen::symptoms::Symptom;
    use crate::game::pathogen::types::{PathogenType, Virus};
    use crate::game::population::Person;
    use crate::game::population::Sex::Male;

    #[test]
    fn add_symptom_increases_catch_chance() {
        let mut p = Pathogen::default();
        let catch = p.catch_chance();

        let s = Symptom::new(
            "Test".to_string(),
            "Test".to_string(),
            99.0,
            1.0001,
            1.0,
            1.0,
            None,
            None,
            None,
            None,
        );

        p.acquire_symptom(&s, None);

        assert!(p.catch_chance() > catch);
    }

    #[test]
    fn add_and_remove_symptom_maintains_consistency() {
        let mut p = Pathogen::default();
        let catch = p.catch_chance();

        let s = Symptom::new(
            "Test".to_string(),
            "Test".to_string(),
            99.0,
            1.0001,
            1.0,
            1.0,
            None,
            None,
            None,
            None,
        );

        p.acquire_symptom(&s, None);

        assert!(p.catch_chance() > catch);

        p.remove_symptom(&s, None);

        assert_eq!(p.catch_chance(), catch);
    }

    #[test]
    fn add_and_remove_on_recover_function() {
        let mut p = Pathogen::default();
        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        let function: Arc<dyn Fn(&mut Person) + Send + Sync> = Arc::new(move |person| {
            *count_clone.lock().unwrap() = 1;
        });

        let s = Symptom::new(
            "Test".to_string(),
            "Test".to_string(),
            99.0,
            1.0001,
            1.0,
            1.0,
            None,
            None,
            None,
            Some(&function),
        );

        p.acquire_symptom(&s, Some(0));
        assert_eq!(
            p.on_recover.len(),
            1,
            "Although symptom had recover function, wasn't added to list"
        );
        let mut person_a = Person::new(0, Age::new(17, 0, 0), Male, 1.00);
        let mut arc = Arc::new(p);
        person_a.infect(&arc);

        arc.perform_recovery(&mut person_a);
        assert_eq!(
            *count.lock().unwrap(),
            1,
            "Problem with recovery functions acting on objects"
        );
    }
}
