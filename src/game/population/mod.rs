use std::borrow::{Borrow, BorrowMut};
use std::cell::{Ref, RefCell};
use std::cmp::Ordering;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

use rand::{random, Rng};

use crate::game::pathogen::infection::Infection;
use crate::game::pathogen::Pathogen;
use crate::game::pathogen::symptoms::Symp;
use crate::game::population::Condition::{Normal, Sick};
use crate::game::population::Sex::{Female, Male};
use crate::game::time::{Age, Time};
use crate::game::Update;

pub mod person_behavior;


pub enum Condition {
    Normal,
    Sick,
    Hospitalized,
    Critical,
    Dead,
}

pub enum Sex {
    Male,
    Female,
}

trait HealthModifier {
    fn get_health_modification_factor(&self) -> f64;
}

impl HealthModifier for Sex {
    fn get_health_modification_factor(&self) -> f64 {
        match self {
            Sex::Male => { 0.95 }
            Sex::Female => { 1.0 }
        }
    }
}




///
/// The most basic component of the simulation
///
pub struct Person {
    id: usize,
    age: Age,
    sex: Sex,
    pre_existing_condition: f64,
    health_points: u32,
    condition: Condition,
    modifiers: Vec<Box<dyn HealthModifier + Sync + Send>>,
    infection: Option<Infection>,
    recovered_status: RwLock<bool>,
}


impl Person {
    fn new(
        id: usize,
        age: Age,
        sex: Sex,
        pre_existing_condition: f64) -> Self {
        let health = Self::max_health(usize::from(age.time_unit().as_years()) as u8, &sex, pre_existing_condition);

        Person {
            id,
            age,
            sex,
            pre_existing_condition,
            health_points: health,
            condition: Normal,
            modifiers: Vec::new(),
            infection: None,
            recovered_status: RwLock::new(false)
        }
    }

    /// Determines the maximum health for a person depending on a few conditions
    fn max_health(age: u8, sex: &Sex, pre_existing_condition: f64) -> u32 {
        ((match age {
            0..=3 => {
                30.0
            }
            4..=9 => {
                70.0
            }
            10..=19 => {
                100.0
            }
            age => {
                10.0 * (-(age as i16) as f64 + 120.0).sqrt()
            }
        }) * sex.get_health_modification_factor() * pre_existing_condition) as u32
    }


    pub fn health_points(&self) -> &u32 {
        &self.health_points
    }

    pub fn alive(&self) -> bool {
        self.health_points > 0
    }

    pub fn dead(&self) -> bool {
        !self.alive()
    }

    pub fn uninfected(&self) -> bool {
        self.infection.is_none()
    }

    pub fn infected(&self) -> bool {
        match &self.infection {
            None => { false }
            Some(i) => {
                !i.recovered()
            }
        }
    }

    pub fn recovered(&self) -> bool {
        match &self.infection {
            None => { false }
            Some(i) => {
                if i.recovered() && !*self.recovered_status.read().unwrap(){
                    *self.recovered_status.write().unwrap() = true

                }
                *self.recovered_status.read().unwrap()
            }
        }
    }

    /// Removes the immunity from someone
    pub fn remove_immunity(&mut self) {
        if self.recovered() && self.infection.is_some(){
            self.infection = None;
        }
    }

    pub fn infect(&mut self, pathogen: &Arc<Pathogen>) -> bool {
        if self.infection.is_none() {
            self.infection = Some(Infection::new(pathogen.clone()));
            self.condition = Sick;
            true
        } else {
            false
        }
    }


    pub fn interact_with<'a>(&self, other: &'a mut Person) -> &'a Person {
        if self.infected() {
            if let Some(ref infection) = self.infection {
                if infection.active_case() {
                    if Pathogen::roll(*infection.get_pathogen().catch_chance()) {
                        let pathogen = Arc::new(infection.get_pathogen().mutate());

                        other.infect(&pathogen);
                    }
                }
            }

        }
        other
    }

}

impl PartialEq for Person {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Update for Person {
    fn update_self(&mut self, delta_time: usize) {

    }

    fn get_update_children(&mut self) -> Vec<&mut dyn Update> {
        match &mut self.infection {
            None => { vec![] },
            Some(i) => {
                vec![i]
            },
        }
    }
}


pub struct Population {
    factory: Arc<Mutex<PersonBuilder>>,
    people: Vec<Arc<RwLock<Person>>>,
    infected: Vec<Arc<RwLock<Person>>>,
    growth_rate: f64,
}

/// Represents the distribution of ages in a population
pub trait PopulationDistribution {
    /// Gets the percent of the population of an age
    /// The lower bounds of this function is 0 and the upperbounds is 120
    /// The area under the curve of the function is 1
    fn get_percent_of_pop(&self, age: usize) -> f64;
}

impl <F> PopulationDistribution for F where F : Fn(usize) -> f64 {
    fn get_percent_of_pop(&self, age: usize) -> f64 {
        self(age)
    }
}



pub struct PersonBuilder {
    count: usize
}

impl PersonBuilder {
    fn new() -> Arc<Mutex<PersonBuilder>> {
        Arc::new(Mutex::new(Self {
            count: 0
        }))
    }

    fn create_person(&mut self,
                     age: Age,
                     sex: Sex,
                     pre_existing_condition: f64) -> Person {
        let id = self.count;
        self.count += 1;
        Person::new(
            id,
            age,
            sex,
            pre_existing_condition
        )
    }
}



impl Population {
    pub fn new<T : PopulationDistribution>(builder: &Arc<Mutex<PersonBuilder>>, growth_rate: f64, population: usize, population_distribution: T) -> Self {
        let mut pop = Vec::new();
        let mut people_created = 0;
        let mut rng = rand::thread_rng();

        for age in 0..121 {
            let people_count  = (population as f64 * population_distribution.get_percent_of_pop(age)) as usize;
            people_created += people_count;
            for _ in 0..people_count {
                let mut builder_guard = builder.lock().unwrap();
                pop.push(
                    Arc::new(RwLock::new(
                        builder_guard.create_person(
                            Age::new(age as u16,
                                     rng.gen_range::<usize, usize, usize>(0, 12),
                                     rng.gen_range::<usize, usize, usize>(0, 28)),
                            if rng.gen_bool(0.5) { Male } else { Female },
                            match rng.gen_range::<f64, f64, f64>(30.0, 200.0) {
                                i if i < 100.0 => { i },
                                i => { 100.0 }
                            } / 100.0
                        )
                    ))
                );

            }
        }

        while people_created < population {
            let mut builder_guard = builder.lock().unwrap();
            pop.push(
                Arc::new(RwLock::new(
                    builder_guard.create_person(
                        Age::new(0, 0, 0),
                        if rng.gen_bool(0.5) { Male} else { Female},
                        1.0
                    )
                ))
            );
            people_created += 1;
        }

        Population {
            factory: builder.clone(),
            people: pop,
            infected: Vec::new(),
            growth_rate
        }
    }

    pub fn infect_one(&mut self, pathogen: &Arc<Pathogen>) -> bool {
        if self.people.is_empty() {
            panic!("Population is empty, can't infect anyone");
        }

        loop {
            let person_id = (random::<f64>() * self.people.len() as f64) as usize;

            let person = self.people.get(person_id).unwrap().clone();
            if person.write().unwrap().infect(pathogen) {
                self.infected.push(person);
                break true;
            }
        }
    }

    pub fn remove_infected(&mut self, person: &Arc<RwLock<Person>>) -> Option<Arc<RwLock<Person>>> {
        let position = self.infected.iter().position(|p| p.read().unwrap().id == person.read().unwrap().id);
        match position {
            None => { None },
            Some(index) => {
                Some(self.infected.remove(index))
            },
        }
    }

    pub fn get_infected(&self) -> Vec<&Person> {
        let mut output = Vec::new();

        output
    }
}


#[cfg(test)]
mod test {
    use std::borrow::{Borrow, BorrowMut};
    use std::collections::HashSet;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use std::thread;

    use crate::game::graph::Graph;
    use crate::game::pathogen::Pathogen;
    use crate::game::pathogen::symptoms::base::Undying;
    use crate::game::pathogen::symptoms::Symp;
    use crate::game::pathogen::types::{PathogenType, Virus};
    use crate::game::population::{Person, PersonBuilder, Population, PopulationDistribution};
    use crate::game::population::Sex::Male;
    use crate::game::time::Age;
    use crate::game::Update;

    struct UniformDistribution { min_age: usize, max_age: usize}

    impl UniformDistribution {
        fn new(min_age: usize, max_age: usize) -> Self {
            Self {
                min_age, max_age
            }
        }
    }

    impl PopulationDistribution for UniformDistribution {
        fn get_percent_of_pop(&self, age: usize) -> f64 {
            if age < self.min_age || age > self.max_age {
                0.0
            } else {
                1.0 / (self.max_age - self.min_age) as f64
            }

        }
    }

    #[test]
    fn can_transfer() {
        let mut person_a = Person::new(0, Age::new(17, 0, 0), Male, 1.00);
        let mut person_b = Person::new(1, Age::new(17, 0, 0), Male, 1.00);
        let mut p = Virus.create_pathogen("Test", 100);
        p.acquire_symptom(
            &Undying.get_symptom()
        );
        let pathogen = Arc::new(p);

        person_a.infect(&pathogen);
        if !person_a.infected() {
            panic!("Person A wasn't infected")
        }

        while !person_a.recovered() && !person_b.infected() {
            person_a.update(20);
            person_a.interact_with(&mut person_b);
        }

        if !person_b.infected() {
            panic!("Person B wasn't infected before Person A recovered")
        }
    }

    /// Tests to see if creating multiple populations at once works fine and all ids are unique
    #[test]
    fn concurrent_population_creation_id_check() {
        let builder = PersonBuilder::new();
        let vec = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();
        for _ in 0..10 {
            let vec_copy = vec.clone();
            let build_clone = builder.clone();
            handles.push(thread::spawn(move || {
                let pop = Population::new(&build_clone, 0.0, 100, UniformDistribution::new(20, 55));
                if let Ok(mut g) = vec_copy.lock() {
                    for ref people in pop.people {
                        g.push(people.clone())
                    }
                }
            }
            ));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let vector = vec.lock().expect("Should be able to access the Vector as all threads have been waited").to_owned();
        assert_eq!(vector.len(), 1000, "1000 people should have been created concurrently, but {} was instead", vector.len());
        let mut found = [false; 1000];
        let mut ids = 0;
        for person in &vector {
            let id = person.read().unwrap().id;
            if found[id] == false {
                found[id] = true;
                ids += 1;
            } else {
                // panic!("Duplicate ID found: {}", id);
            }
        }
        assert_eq!(ids, vector.len(), "There should be 1000 unique IDS, but {} were created", ids);
    }

    #[test]
    fn can_infect_a_population() {
        let mut pop = Population::new(&PersonBuilder::new(), 0.0, 1000, UniformDistribution::new(0, 120));
        let mut pathogen = Arc::new(Virus.create_pathogen("Test", 100));
        assert!(pop.infect_one(&pathogen));
    }

    #[test]
    fn community_transfer() {
        let mut pop = Population::new(&PersonBuilder::new(), 0.0, 1000, UniformDistribution::new(0, 120));
        let mut pathogen = Arc::new(Virus.create_pathogen("Test", 100));
        assert!(pop.infect_one(&pathogen));
    }
}