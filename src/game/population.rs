use std::rc::Rc;

use crate::game::pathogen::infection::Infection;
use crate::game::pathogen::Pathogen;
use crate::game::population::Condition::{Normal, Sick};
use crate::game::time::{Age, Time};
use crate::game::Update;

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
    age: Age,
    sex: Sex,
    pre_existing_condition: f64,
    health_points: u32,
    condition: Condition,
    modifiers: Vec<Box<dyn HealthModifier>>,
    infection: Option<Infection>,
}


impl Person {
    fn new(age: Age,
           sex: Sex,
           pre_existing_condition: f64) -> Self {
        let health = Self::max_health(usize::from(age.time_unit().as_years()) as u8, &sex, pre_existing_condition);
        Person {
            age,
            sex,
            pre_existing_condition,
            health_points: health,
            condition: Normal,
            modifiers: Vec::new(),
            infection: None,
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
                i.recovered()
            }
        }
    }

    pub fn infect(&mut self, pathogen: &Rc<Pathogen>) -> bool {
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
                        let pathogen = Rc::new(infection.get_pathogen().mutate());

                        other.infect(&pathogen);
                    }
                }
            }

        }
        other
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


pub struct Population<'a> {
    people: Vec<Person>,
    infected: Vec<&'a Person>,
    growth_rate: f64,
}

/// Represents the distribution of ages in a population
pub trait PopulationDistribution {
    /// Gets the percent of the population of an age
    /// The lower bounds of this function is 0 and the upperbounds is 120
    /// The area under the curve of the function is 1
    fn get_percent_of_pop(self, age: usize) -> f64;
}

impl <F> PopulationDistribution for F where F : Fn(usize) -> f64 {
    fn get_percent_of_pop(self, age: usize) -> f64 {
        self(age)
    }
}



impl<'a> Population<'a> {
    pub fn new<T : PopulationDistribution>(growth_rate: f64, population: usize, population_distribution: T) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::rc::Rc;

    use crate::game::graph::Graph;
    use crate::game::pathogen::Pathogen;
    use crate::game::pathogen::symptoms::base::Undying;
    use crate::game::pathogen::symptoms::Symp;
    use crate::game::pathogen::types::{PathogenType, Virus};
    use crate::game::population::{Person, Population, PopulationDistribution};
    use crate::game::population::Sex::Male;
    use crate::game::time::Age;
    use crate::game::Update;

    struct UniformDistribution;

    impl PopulationDistribution for UniformDistribution {
        fn get_percent_of_pop(self, age: usize) -> f64 {
            1.0 / 120.0
        }
    }

    #[test]
    fn can_transfer() {
        let mut person_a = Person::new(Age::new(17, 0, 0), Male, 1.00);
        let mut person_b = Person::new(Age::new(17, 0, 0), Male, 1.00);
        let mut p = Virus.create_pathogen("Test", 100);
        p.acquire_symptom(
            &Undying.get_symptom()
        );
        let pathogen = Rc::new(p);

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

    #[test]
    fn community_transfer() {
        let mut pop = Population::new(0.0, 1000, UniformDistribution);

    }
}