use std::fmt::{Debug, Error, Formatter, Result};

use rand::Rng;

use crate::game::time::{Time, TimeUnit};

pub mod infection;

pub trait RecoveryChance {
    fn recover(&self, passed: TimeUnit) -> f64;
}

impl RecoveryChance for fn(TimeUnit) -> f64 {
    fn recover(&self, passed: TimeUnit) -> f64 {
        self(passed)
    }
}

struct DefaultRecoveryChance;

impl RecoveryChance for DefaultRecoveryChance {
    fn recover(&self, passed: TimeUnit) -> f64 {
        let days = usize::from(passed.into_days()) as f64;
        days * days * 0.02 / (24.0 * 60.0)
    }
}

impl Debug for dyn RecoveryChance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "recover(t)")
    }
}


#[derive(Debug)]
pub struct Pathogen
{
    name: String, // name of the pathogen
    catch_chance: f64, // chance spreads per interaction
    severity: f64, // chance will go to doctor
    fatality: f64, // chance hp reduction
    internal_spread_rate: f64, // chance amount of pathogen increases
    min_count_for_symptoms: usize, // minimum amount of pathogens for spread, be discovered, be fatal, and to recover
    recovery_chance: Box<dyn RecoveryChance> // chance of recovery after TimeUnit (converted to Minutes)
}

impl Pathogen {


    pub fn new<T : RecoveryChance + 'static>(name: &str, min_count_for_symptoms: usize, recovery_chance: T) -> Self {
        Pathogen {
            name: name.to_string(),
            catch_chance: 0.0,
            severity: 0.0,
            fatality: 0.0,
            internal_spread_rate: 0.01,
            min_count_for_symptoms,
            recovery_chance: Box::new(recovery_chance)
        }
    }

    pub fn roll(chance: f64) -> bool {
        rand::thread_rng().gen_bool(chance)
    }
}
