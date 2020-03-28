use crate::game::time::Time;

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
pub struct Person<'a> {
    birthday: usize,
    timer: &'a Time,
    sex: Sex,
    pre_existing_condition: f64,
    health_points: u32,
    condition: Condition
}

impl <'a> Person<'a> {
    fn new(birthday: usize,
               timer: &'a Time,
               sex: Sex,
               pre_existing_condition: f64,
               health_points: u32,
               condition: Condition) -> Self {
        Person {
            birthday,
            timer,
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
}



pub struct Population<'a> {
    timer: &'a Time,
    people: Vec<Person<'a>>,
    growth_rate: f64
}

impl<'a> Population<'a> {



    pub fn new(timer: &'a Time, growth_rate: f64, population: usize, population_distribution: fn(usize) -> f64) {

    }
}