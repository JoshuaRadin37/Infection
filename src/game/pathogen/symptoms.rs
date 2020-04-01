use std::fmt::{Debug, Error, Formatter, Result};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::usize;

use structure::graph::{Graph, GraphResult};

use crate::game::population::Person;

///
/// A symptom are the building blocks of pathogens, and effect the way they behave while in a person
pub struct Symptom {
    name: String,
    description: String,
    catch_chance_increase: f64,         // percentage increase
    severity_increase: f64,             // percentage increase
    fatality_increase: f64,             // percentage increase
    internal_spread_rate_increase: f64, // percentage increase
    duration_change: Option<f64>,
    spread_change: Option<f64>,
    additional_effect: Option<fn()>,
    recovery_function: Option<Arc<dyn Fn(&mut Person) + Send + Sync>>,
}

impl Symptom {
    /// Creates a new symptom that affects the way a [Pathogen] behaves
    ///
    /// # Inputs
    /// * `name` - The name of the symptom
    /// * `description` - description
    /// * `catch_chance_increase` - A number in the range of (-100, 100) representing the percent change of the catch chance when an
    /// infected person interacts with another person
    /// * `severity_increase` - A number in the range of (-100, 100) representing the percent change of the severity of an infection,
    /// which impacts the likelyhood of a person going to the doctor, and not interacting with people or traveling
    /// * `fatality_increase` - A number in the range of (-100, 100) representing the percent change of the fatality of an infection,
    /// * `internal_spread_rate_increase` - A number in the range of (-100, 100) representing the percent change of the spread rate within an
    /// infected person, where the greater the value, the faster a person's case becomes active
    /// where the higher the fatality the more likely an infected person is to lose a hp per tick
    /// * `recovery_chance_base` - If a `Some(...)` value, set the base recovery chance to that value
    /// * `additonal_effect` - If a `Some(...)` value, when a person gets infected with a pathogen with this symptom, this function is run
    /// (Note: a symptom with such a function can not be reversed)
    /// * `recovery_function` - If a `Some(...)` value, this is a function that is run on a person who just recovered from a pathogen with
    /// this symptom
    ///
    /// # Example
    ///
    /// ```
    ///use infection::game::pathogen::symptoms::Symptom;
    ///Symptom::new(
    ///                 "A Runny Nose".to_string(),
    ///                 "Some serious leakage problems".to_string(),
    ///                 10.0,
    ///                 1.0001,
    ///                 1.0,
    ///                 1.0,
    ///                 None,
    ///                 None,
    ///                 None,
    ///                 None
    ///             );
    ///
    /// ```
    ///
    /// # Panics
    ///
    /// The function will panic if any of the `*_increase` parameters are not within the range of (100, -100)\
    ///
    /// ```rust,should_panic
    ///use infection::game::pathogen::symptoms::Symptom;
    /// Symptom::new("Panic attacks".to_string(), "This panics".to_string(), 25.0, 35.0, 120.0, 0.0, None, None, None, None);
    /// ```
    pub fn new(
        name: String,
        description: String,
        mut catch_chance_increase: f64,
        mut severity_increase: f64,
        mut fatality_increase: f64,
        mut internal_spread_rate_increase: f64,
        duration_change: Option<f64>,
        spread_change: Option<f64>,
        additional_effect: Option<fn()>,
        recovery_function: Option<&Arc<dyn Fn(&mut Person) + Send + Sync>>,
    ) -> Self {
        if catch_chance_increase.abs() >= 100.0 {
            panic!(
                "Catch chance increase must be in range (-100, 100), but was given {}",
                catch_chance_increase
            )
        }
        if severity_increase.abs() >= 100.0 {
            panic!(
                "Severity increase must be in range (-100, 100), but was given {}",
                severity_increase
            )
        }
        if fatality_increase.abs() >= 100.0 {
            panic!(
                "Fatality increase must be in range (-100, 100), but was given {}",
                fatality_increase
            )
        }
        if internal_spread_rate_increase.abs() >= 100.0 {
            panic!(
                "Catch chance increase must be in range (-100, 100), but was given {}",
                internal_spread_rate_increase
            )
        }

        if catch_chance_increase < 0.0 {
            catch_chance_increase = 1.0 + catch_chance_increase / 100.0
        }

        if severity_increase < 0.0 {
            severity_increase = 1.0 + severity_increase / 100.0
        }

        if fatality_increase < 0.0 {
            fatality_increase = 1.0 + fatality_increase / 100.0
        }

        if internal_spread_rate_increase < 0.0 {
            internal_spread_rate_increase = 1.0 + internal_spread_rate_increase / 100.0
        }

        Symptom {
            name,
            description,
            catch_chance_increase,
            severity_increase,
            fatality_increase,
            internal_spread_rate_increase,
            duration_change,
            spread_change,
            additional_effect: match additional_effect {
                None => None,
                Some(f) => Some(f),
            },
            recovery_function: recovery_function.map(|f| f.clone()),
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

    pub fn get_duration_change(&self) -> &Option<f64> {
        &self.duration_change
    }

    pub fn get_spread_change(&self) -> &Option<f64> {
        &self.spread_change
    }

    pub fn can_reverse(&self) -> bool {
        self.additional_effect.is_none() && self.duration_change.map_or(true, |f| f.is_finite())
    }

    pub fn additional_effect(&self) {
        match &self.additional_effect {
            None => {}
            Some(b) => b(),
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
    next_id: usize,
}

impl SymptomMapBuilder {
    pub fn new() -> Self {
        Self {
            symptoms_map: Graph::new(),
            symptoms: vec![],
            next_id: 0,
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

    pub fn add_next_symptom(
        &mut self,
        id1: usize,
        id2: usize,
        mutation_chance: f64,
    ) -> GraphResult<usize> {
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
    map_builder: &'a mut SymptomMapBuilder,
}

impl<'a> SymptomMapBuilderEntry<'a> {
    fn new(node: usize, map_builder: &'a mut SymptomMapBuilder) -> Self {
        SymptomMapBuilderEntry { node, map_builder }
    }

    pub fn node(&self) -> usize {
        self.node
    }

    pub fn next_symptom(
        &mut self,
        symptom: Symptom,
        mutation_chance: f64,
    ) -> SymptomMapBuilderEntry {
        let output = self.map_builder.add(symptom);
        let id1 = self.node;
        let id2 = output.node;
        output
            .map_builder
            .add_next_symptom(id1, id2, mutation_chance)
            .expect("Should not fail");
        output
    }

    pub fn add_next_symptoms(&mut self, symptoms: Vec<(Symptom, f64)>) -> Vec<usize> {
        let mut output = Vec::new();
        for (symptom, mutation_chance) in symptoms {
            let next = self.map_builder.push(symptom);
            self.map_builder
                .add_next_symptom(self.node, next, mutation_chance);
            output.push(next);
        }
        output
    }
}

pub mod base {
    use std::cell::RefCell;
    use std::sync::{Arc, Mutex};

    use crate::game::pathogen::symptoms::{Symp, Symptom};
    use crate::game::population::Person;

    /// Cheat symptoms, way too powerful or weak for standard viruses
    pub mod cheat {
        use std::f64::INFINITY;

        use super::*;

        /// Person can never recover
        pub struct Undying;

        impl Symp for Undying {
            fn get_symptom(&self) -> Symptom {
                Symptom::new(
                    "Immunity Immunity".to_string(),
                    "The immune system can never beat the pathogen, and the person will never recover".to_string(),
                    99.9,
                    1.0001,
                    1.0,
                    1.0,
                    Some(INFINITY),
                    Some(0.0),
                    None,
                    None
                )
            }
        }

        pub fn create_recovery_function<'a, F>(
            function: F,
        ) -> Arc<dyn Fn(&'a mut Person) + Send + Sync + 'a>
        where
            F: Fn(&'a mut Person) + Send + Sync + 'a,
        {
            let output: Arc<dyn Fn(&'a mut Person) + Send + Sync + 'a> = Arc::new(function);
            output
        }

        // Person are never immune to the Pathogen by forcing the Person to remove their infection
        pub struct NeverImmune;

        impl Symp for NeverImmune {
            fn get_symptom(&self) -> Symptom {
                let function: Arc<dyn Fn(&mut Person) + Send + Sync> =
                    Arc::new(|person| person.remove_immunity());

                Symptom::new(
                    "Viral Amnesia".to_string(),
                    "What Virus? ".to_string(),
                    1.0,
                    1.0,
                    1.0,
                    99.0,
                    None,
                    None,
                    None,
                    Some(&function),
                )
            }
        }

        pub struct NoSpread;

        impl Symp for NoSpread {
            fn get_symptom(&self) -> Symptom {
                Symptom::new(
                    "Catch me if you can!".to_string(),
                    "Which is pretty unlikely. -100% infection rate".to_string(),
                    0.0,
                    0.0,
                    0.0,
                    99.0,
                    None,
                    None,
                    None,
                    None,
                )
            }
        }

        pub struct CustomCatchChance(pub f64);
        impl Symp for CustomCatchChance {
            fn get_symptom(&self) -> Symptom {
                Symptom::new(
                    format!("Custom Catch Chance {}", self.0),
                    "Genetics are wild".to_string(),
                    self.0,
                    0.0,
                    0.0,
                    0.0,
                    None,
                    None,
                    None,
                    None,
                )
            }
        }

        pub struct CustomInternalSpreadRate(pub f64);
        impl Symp for CustomInternalSpreadRate {
            fn get_symptom(&self) -> Symptom {
                Symptom::new(
                    format!("Custom Internal Spread rate {}", self.0),
                    "Genetics are wild".to_string(),
                    0.0,
                    0.0,
                    0.0,
                    self.0,
                    None,
                    None,
                    None,
                    None,
                )
            }
        }

        pub struct CustomSeverity(pub f64);
        impl Symp for CustomSeverity {
            fn get_symptom(&self) -> Symptom {
                Symptom::new(
                    format!("Custom Severity {}", self.0),
                    "Genetics are wild".to_string(),
                    0.0,
                    self.0,
                    0.0,
                    0.0,
                    None,
                    None,
                    None,
                    None,
                )
            }
        }

        pub struct CustomFatality(pub f64);
        impl Symp for CustomFatality {
            fn get_symptom(&self) -> Symptom {
                Symptom::new(
                    format!("Custom Fatality {}", self.0),
                    "Genetics are wild".to_string(),
                    0.0,
                    0.0,
                    self.0,
                    0.0,
                    None,
                    None,
                    None,
                    None,
                )
            }
        }

        pub struct CustomDuration(pub f64);
        impl Symp for CustomDuration {
            fn get_symptom(&self) -> Symptom {
                Symptom::new(
                    format!("Custom Duration {}", self.0),
                    "Genetics are wild".to_string(),
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    Some(self.0),
                    None,
                    None,
                    None,
                )
            }
        }

        pub struct CustomSpread(pub f64);
        impl Symp for CustomSpread {
            fn get_symptom(&self) -> Symptom {
                Symptom::new(
                    format!("Custom Spread {}", self.0),
                    "Genetics are wild".to_string(),
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    None,
                    Some(self.0),
                    None,
                    None,
                )
            }
        }
    }

    pub struct RunnyNose;
    impl Symp for RunnyNose {
        fn get_symptom(&self) -> Symptom {
            Symptom::new(
                "A Runny Nose".to_string(),
                "Some serious leakage problems".to_string(),
                5.0,
                1.0001,
                1.0,
                20.0,
                None,
                None,
                None,
                None,
            )
        }
    }

    pub struct Cough(pub u8);
    impl Symp for Cough {
        fn get_symptom(&self) -> Symptom {
            Symptom::new(
                format!("Cough {}", self.0),
                "A upper respiratory cough".to_string(),
                9.5,
                1.5,
                1.0,
                1.0,
                None,
                None,
                None,
                None,
            )
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::sync::{Arc, mpsc, Mutex};
    use std::sync::mpsc::TryRecvError;
    use std::thread;
    use std::thread::spawn;
    use std::time::Duration;

    use rand::thread_rng;

    use crate::game::{Age, Update};
    use crate::game::pathogen::symptoms::base::cheat::NeverImmune;
    use crate::game::pathogen::symptoms::Symp;
    use crate::game::pathogen::types::{PathogenType, Virus};
    use crate::game::population::Person;
    use crate::game::population::Sex::Male;

    #[test]
    fn never_immune_removes_immunity() {
        let mut p = Virus.create_pathogen("Test", 0);
        let activations = Arc::new(Mutex::new(0));
        p.acquire_symptom(&NeverImmune.get_symptom(), None);

        let mut person = Person::new(0, Age::new(17, 0, 0), Male, 1.00);
        let arc = Arc::new(p);
        person.infect(&arc);

        assert!(person.infected(), "Person must be infected");
        let (tx, rx) = mpsc::channel();

        let handle = spawn(move || {
            while !person.recovered() {
                match rx.try_recv() {
                    Ok(_) => {
                        return Ok(true);
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(_) => {
                        return Err(());
                    }
                }

                person.update(20);
            }

            Ok(false)
        });

        thread::sleep(Duration::from_secs(1));
        tx.send(());
        if let Ok(Ok(not_recovered)) = handle.join() {
            assert!(not_recovered, "The person should have never recovered")
        } else {
            panic!("Thread errored out when it should not have")
        }
    }
}
