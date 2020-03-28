use crate::game::time::{Time, Age};

pub enum Condition {
    Normal,
    Sick,
    Hospitalized,
    Critical,
    Dead
}

pub enum Sex {
    Male,
    Female
}

trait HealthModifier {
    fn get_health_modification_factor(&self) -> f64;
}

impl HealthModifier for Sex {
    fn get_health_modification_factor(&self) -> f64 {
        match self {
            Sex::Male => { 0.95 },
            Sex::Female => { 1.0 },
        }
    }
}



///
///
///
pub struct Person {
    age: Age,
    sex: Sex,
    pre_existing_condition: f64,
    health_points: u32,
    condition: Condition
}



impl Person {

    fn new(age: Age,
               sex: Sex,
               pre_existing_condition: f64,
               health_points: u32,
               condition: Condition) -> Self {
        Person {
            age,
            sex,
            pre_existing_condition,
            health_points,
            condition
        }
    }

    fn max_health(age: u8, sex: &Sex, pre_existing_condition: f64) -> u32 {
        ((match age {
            0..=3 => {
                30.0
            },
            4..=9 => {
                70.0
            },
            10..=19 => {
                100.0
            }
            age => {
                (10.0 * (-(age as i16) as f64 + 120.0).sqrt())
            }
        }) * sex.get_health_modification_factor() * pre_existing_condition) as u32
    }

    fn health_points(&self) -> &u32 {
        &self.health_points
    }
}



pub struct Population {
    people: Vec<Person>,
    growth_rate: f64
}

impl Population {



    pub fn new(growth_rate: f64, population: usize, population_distribution: fn(usize) -> f64) {

    }
}