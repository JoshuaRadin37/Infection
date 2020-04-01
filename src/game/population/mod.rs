use std::borrow::{Borrow, BorrowMut};
use std::cell::{Ref, RefCell};
use std::cmp::{min, Ordering};
use std::fmt::{Debug, Display, Error, Formatter, Result};
use std::mem;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

use rand::{random, Rng};

use structure::time::Time;

use crate::game::{Age, ParallelUpdate, roll, tick_to_game_time_conversion, Update};
use crate::game::pathogen::infection::Infection;
use crate::game::pathogen::Pathogen;
use crate::game::pathogen::symptoms::Symp;
use crate::game::population::Condition::Normal;
use crate::game::population::Sex::{Female, Male};

pub mod person_behavior;

#[derive(Debug, Eq, PartialEq)]
pub enum Condition {
    Normal,
    NeedsHospital,
    Hospitalized,
}

#[derive(Debug)]
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
            Sex::Male => 0.95,
            Sex::Female => 1.0,
        }
    }
}

///
/// The most basic component of the simulation
///
pub struct Person {
    id: usize,
    age: Mutex<Age>,
    sex: Sex,
    pre_existing_condition: f64,
    health_points: RwLock<u32>,
    condition: Mutex<Condition>,
    modifiers: Mutex<Vec<Box<dyn HealthModifier + Sync + Send>>>,
    infection: Mutex<Option<Infection>>,
    recovered_status: RwLock<bool>,
}

impl Display for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Person {}", self.id)
    }
}

impl Debug for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "Person {} {{ age: {:?}, sex: {:?}, prex: {:?}, hp: {:?}, infected: {:?}}}",
            self.id,
            self.age.lock().unwrap().0.format("{:y}y {:m(12m)}m"),
            self.sex,
            self.pre_existing_condition,
            self.health_points,
            self.infected()
        )
    }
}

impl Person {
    pub(crate) fn new(id: usize, age: Age, sex: Sex, pre_existing_condition: f64) -> Self {
        let health = Self::max_health(
            usize::from(age.time_unit().as_years()) as u8,
            &sex,
            pre_existing_condition,
        );

        Person {
            id,
            age: Mutex::new(age),
            sex,
            pre_existing_condition,
            health_points: RwLock::new(health),
            condition: Mutex::new(Normal),
            modifiers: Mutex::new(Vec::new()),
            infection: Mutex::new(None),
            recovered_status: RwLock::new(false),
        }
    }

    /// Determines the maximum health for a person depending on a few conditions
    fn max_health(age: u8, sex: &Sex, pre_existing_condition: f64) -> u32 {
        ((match age {
            0..=3 => 30.0,
            4..=9 => 70.0,
            10..=19 => 100.0,
            age => 10.0 * (-(age as i16) as f64 + 120.0).sqrt(),
        }) * 10.0
            * sex.get_health_modification_factor()
            * pre_existing_condition) as u32
    }

    pub fn condition(&self) -> f64 {
        (*self.health_points.read().unwrap() as f64 / 1000.0) * self.pre_existing_condition
    }

    pub fn health_points(&self) -> &RwLock<u32> {
        &self.health_points
    }

    pub fn alive(&self) -> bool {
        *self.health_points.read().unwrap() > 0
    }

    pub fn dead(&self) -> bool {
        !self.alive()
    }

    pub fn never_infected(&self) -> bool {
        self.infection.lock().unwrap().is_none()
    }

    pub fn infected(&self) -> bool {
        if self.dead() {
            return false;
        }
        match &*self.infection.lock().unwrap() {
            None => false,
            Some(i) => !i.recovered(),
        }
    }

    pub fn recovered(&self) -> bool {
        if self.dead() {
            return false;
        }
        *self.recovered_status.read().unwrap()
    }

    /// Removes the immunity from someone
    pub fn remove_immunity(&mut self) {
        if self.recovered() && self.infection.lock().unwrap().is_some() {
            *self.infection.lock().unwrap() = None;
            *self.recovered_status.write().unwrap() = false;
        }
    }

    pub fn infect(&mut self, pathogen: &Arc<Pathogen>) -> bool {
        if self.infection.lock().unwrap().is_none() {
            *self.infection.lock().unwrap() =
                Some(Infection::new(pathogen.clone(), self.condition()));
            true
        } else {
            false
        }
    }

    /// Perform an interaction with another person
    ///
    /// ###Return
    /// Whether the other person just became infected
    pub fn interact_with(&self, other: &mut Person) -> bool {
        if other.infected() || other.recovered() {
            return false;
        }
        if self.infected() {
            if let Some(ref infection) = *self.infection.lock().unwrap() {
                if infection.active_case() {
                    if roll(infection.get_pathogen().catch_chance()) {
                        let pathogen = Arc::new(infection.get_pathogen().mutate());

                        return other.infect(&pathogen);
                    }
                }
            }
        }
        false
    }

    fn get_age_years(&self) -> u8 {
        usize::from(self.age.lock().unwrap().0.as_years()) as u8
    }
}

impl PartialEq for Person {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Update for Person {
    fn update_self(&mut self, delta_time: usize) {
        {
            match &mut *self.infection.lock().unwrap() {
                // update infection
                None => {}
                Some(i) => {
                    i.update(delta_time);
                }
            }
        }

        {
            // update age
            let mut age_guard = self.age.lock().unwrap();
            *age_guard += tick_to_game_time_conversion(delta_time);
        }

        if !self.recovered() {
            // update recover status
            let infection_recovered = {
                let guard1 = &*self.infection.lock().unwrap();
                if let Some(i) = guard1 {
                    i.recovered()
                } else {
                    false
                }
            };

            if infection_recovered {
                *self.recovered_status.write().unwrap() = true;
                *self.condition.lock().unwrap() = Normal;
                let mut lock = self.infection.lock();
                let guard = (&*lock.unwrap()).clone();
                {
                    match guard {
                        None => {}
                        Some(i) => {
                            i.get_pathogen().perform_recovery(self);
                        }
                    }
                }
            }
        }

        // update health points and condition
        {
            let max_health = {
                let mut hp_guard = self.health_points.write().unwrap();
                let max_health =
                    Self::max_health(self.get_age_years(), &self.sex, self.pre_existing_condition);

                if max_health < *hp_guard {
                    *hp_guard = max_health;
                }

                max_health
            };

            if self.infected() {
                let mut rate = 1.0;
                let get_hurt = {
                    // remove infection mutex as fast as possible
                    match &*self.infection.lock().unwrap() {
                        None => panic!("Infection must exist"),
                        Some(i) => {
                            if !i.active_case() {
                                false
                            } else {
                                rate = 1.0 / (1.0 - i.get_pathogen().severity());
                                roll(i.get_pathogen().fatality())
                            }
                        }
                    }
                };

                if get_hurt {
                    let change = &mut *self.condition.lock().unwrap();
                    let mut hp_guard = self.health_points.write().unwrap();
                    *hp_guard -= u32::min(
                        *hp_guard,
                        ((match change {
                            Condition::Normal => 1.0,
                            Condition::NeedsHospital => 3.0,
                            Condition::Hospitalized => 2.0,
                        }) * rate) as u32,
                    );

                    if *change == Condition::Normal {
                        match *hp_guard {
                            hp if hp < max_health / 4 => {
                                *change = Condition::NeedsHospital;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

pub struct PersonBuilder {
    count: usize,
}

impl PersonBuilder {
    pub fn new() -> Arc<Mutex<PersonBuilder>> {
        Arc::new(Mutex::new(Self { count: 0 }))
    }

    fn create_person(&mut self, age: Age, sex: Sex, pre_existing_condition: f64) -> Person {
        let id = self.count;
        self.count += 1;
        Person::new(id, age, sex, pre_existing_condition)
    }
}

pub struct Population {
    factory: Arc<Mutex<PersonBuilder>>,
    people: Vec<Arc<RwLock<Person>>>,
    original_pop: usize,
    current_pop: usize,
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

impl<F> PopulationDistribution for F
where
    F: Fn(usize) -> f64,
{
    fn get_percent_of_pop(&self, age: usize) -> f64 {
        self(age)
    }
}

impl Population {
    pub fn new<T: PopulationDistribution>(
        builder: &Arc<Mutex<PersonBuilder>>,
        growth_rate: f64,
        population: usize,
        population_distribution: T,
    ) -> Self {
        let mut pop = Vec::new();
        let mut people_created = 0;
        let mut rng = rand::thread_rng();

        for age in 0..121 {
            let people_count =
                (population as f64 * population_distribution.get_percent_of_pop(age)) as usize;
            for _ in 0..people_count {
                let mut builder_guard = builder.lock().unwrap();
                pop.push(Arc::new(RwLock::new(builder_guard.create_person(
                    Age::new(
                        age as u16,
                        rng.gen_range::<usize, usize, usize>(0, 12),
                        rng.gen_range::<usize, usize, usize>(0, 28),
                    ),
                    if rng.gen_bool(0.5) { Male } else { Female },
                    match rng.gen_range::<f64, f64, f64>(30.0, 200.0) {
                        i if i < 100.0 => i,
                        i => 100.0,
                    } / 100.0,
                ))));
                people_created += 1;
                if people_created == population {
                    break;
                }
            }
        }

        while people_created < population {
            let mut builder_guard = builder.lock().unwrap();
            pop.push(Arc::new(RwLock::new(builder_guard.create_person(
                Age::new(0, 0, 0),
                if rng.gen_bool(0.5) { Male } else { Female },
                1.0,
            ))));
            people_created += 1;
        }

        Population {
            factory: builder.clone(),
            people: pop,
            original_pop: population,
            current_pop: population,
            infected: Vec::new(),
            growth_rate,
        }
    }

    /// gets the count of people who are either infected or recovered
    pub fn get_all_ever_infected(&self) -> usize {
        self.get_everyone()
            .iter()
            .filter(|p| {
                let person = &*p.read().unwrap();
                person.recovered() || person.infected()
            })
            .count()
    }

    pub fn infect_one(&mut self, pathogen: &Arc<Pathogen>) -> bool {
        if self.people.is_empty() {
            panic!("Population is empty, can't infect anyone");
        }

        loop {
            let person_id = (random::<f64>() * self.people.len() as f64) as usize;

            let person = self.people.get(person_id).unwrap().clone();
            {
                let read = person.read().unwrap();
                if read.infected() || read.recovered() {
                    continue;
                }
            }
            if person.write().unwrap().infect(pathogen) {
                self.infected.push(person);
                break true;
            }
        }
    }

    pub fn remove_infected(&mut self, person: &Arc<RwLock<Person>>) -> Option<Arc<RwLock<Person>>> {
        let position = self
            .infected
            .iter()
            .position(|p| p.read().unwrap().id == person.read().unwrap().id);
        match position {
            None => None,
            Some(index) => Some(self.infected.remove(index)),
        }
    }

    pub fn get_everyone(&self) -> &Vec<Arc<RwLock<Person>>> {
        &self.people
    }

    pub fn get_infected(&self) -> &Vec<Arc<RwLock<Person>>> {
        &self.infected
    }

    pub fn get_total_population(&self) -> usize {
        self.current_pop
    }

    pub fn get_original_population(&self) -> usize {
        self.original_pop
    }

    pub fn age_a_year(&mut self) {
        for _ in 0..1200 {
            self.update(438);
        }
    }
}


impl ParallelUpdate<Arc<RwLock<Person>>> for Population {
    fn parallel_update_self(&mut self, delta_time: usize) {
        let mut infected_remove = Vec::new();

        for (pos, x) in self.get_infected().iter().enumerate() {
            let person = &*x.read().expect("Should be able to get person");
            if person.recovered() || person.dead() {
                infected_remove.push(pos)
            }
        }

        infected_remove.sort_by(|a, b| a.cmp(b).reverse());
        for r in infected_remove {
            self.infected.remove(r);
        }

        let mut full_remove = Vec::new();
        for (pos, x) in self.get_everyone().iter().enumerate() {
            let person = &*x.read().expect("Should be able to get person");
            if person.dead() {
                full_remove.push(pos)
            }
        }

        full_remove.sort_by(|a, b| a.cmp(b).reverse());
        for r in full_remove {
            self.people.remove(r);
            self.current_pop -= 1;
        }
    }

    fn parallel_get_update_children(&mut self) -> Vec<&mut Arc<RwLock<Person>>> {
        self.people.iter_mut().map(|arc| arc).collect()
    }
}

pub struct UniformDistribution {
    min_age: usize,
    max_age: usize,
}

impl UniformDistribution {
    pub fn new(min_age: usize, max_age: usize) -> Self {
        Self { min_age, max_age }
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

#[cfg(test)]
mod test {
    use std::borrow::{Borrow, BorrowMut};
    use std::collections::HashSet;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use std::thread;

    use crate::game::{Age, Update};
    use crate::game::pathogen::Pathogen;
    use crate::game::pathogen::symptoms::base::cheat::{CustomFatality, Undying};
    use crate::game::pathogen::symptoms::Symp;
    use crate::game::pathogen::types::{PathogenType, Virus};
    use crate::game::population::{
        Person, PersonBuilder, Population, PopulationDistribution, UniformDistribution,
    };
    use crate::game::population::Sex::Male;

    #[test]
    fn can_transfer() {
        let mut person_a = Person::new(0, Age::new(17, 0, 0), Male, 1.00);
        let mut person_b = Person::new(1, Age::new(17, 0, 0), Male, 1.00);
        let mut p = Virus.create_pathogen("Test", 100);
        p.acquire_symptom(&Undying.get_symptom(), None);
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
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let vector = vec
            .lock()
            .expect("Should be able to access the Vector as all threads have been waited")
            .to_owned();
        assert_eq!(
            vector.len(),
            1000,
            "1000 people should have been created concurrently, but {} was instead",
            vector.len()
        );
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
        assert_eq!(
            ids,
            vector.len(),
            "There should be 1000 unique IDS, but {} were created",
            ids
        );
    }

    #[test]
    fn can_infect_a_population() {
        let mut pop = Population::new(
            &PersonBuilder::new(),
            0.0,
            1000,
            UniformDistribution::new(0, 120),
        );
        let mut pathogen = Arc::new(Virus.create_pathogen("Test", 100));
        assert!(pop.infect_one(&pathogen));
    }

    #[test]
    fn healthy_population_doesnt_lose_health() {
        let mut pop = Population::new(
            &PersonBuilder::new(),
            0.0,
            1000,
            UniformDistribution::new(10, 60),
        );
        let healths = pop
            .get_everyone()
            .iter()
            .map(|p| *p.read().unwrap().health_points().read().unwrap())
            .collect::<Vec<u32>>();

        for _ in 0..1000 {
            pop.update(20);
        }

        for (pos, person) in pop.get_everyone().iter().enumerate() {
            let person = person.read().unwrap();
            assert_eq!(
                *person.health_points().read().unwrap(),
                *healths.get(pos).unwrap(),
                "{:?} lost health",
                &*person
            );
        }
    }

    #[test]
    fn can_kill_a_person() {
        let mut person_a = Person::new(0, Age::new(17, 0, 0), Male, 1.00);
        let mut p = Pathogen::default();
        p.acquire_symptom(&CustomFatality(99.0).get_symptom(), None);
        let mut pathogen = Arc::new(p);
        assert!(person_a.infect(&pathogen));

        while person_a.infected() {
            person_a.update(20);
        }

        assert!(
            !person_a.recovered(),
            "Person should have not been able to recover before they died"
        );
        assert!(person_a.dead())
    }
}
