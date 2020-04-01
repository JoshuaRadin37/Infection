use std::rc::Rc;
use std::sync::Arc;

use rand::distributions::Distribution;
use rand::Rng;

use structure::time::TimeUnit;
use structure::time::TimeUnit::Minutes;

use crate::game::{Age, roll, tick_to_game_time_conversion, Update};
use crate::game::pathogen::Pathogen;

#[derive(Clone)]
pub struct Infection {
    pathogen: Arc<Pathogen>, // pathogen
    infection_age: Age, // age of the infection
    predetermined_duration: TimeUnit,
    pathogen_count: usize,
    recovered: bool // if the person has recovered
}

impl Infection {

    pub fn new(pathogen: Arc<Pathogen>, condition: f64) -> Self {
        if pathogen.average_recovery_time() <= pathogen.base_recovery_distance() {
            panic!("Pathogen recovery range {} is greater than the average recovery time {}", pathogen.base_recovery_distance(),  pathogen.average_recovery_time());
        }
        let min_duration = usize::max(0, pathogen.average_recovery_time() - (pathogen.base_recovery_distance() as f64 * condition.powi(2)) as usize);
        let max_duration = pathogen.average_recovery_time() + (pathogen.base_recovery_distance() as f64 / condition) as usize;


        let duration = if min_duration == max_duration {
            Minutes(min_duration)
        } else {
            Minutes(rand::thread_rng().gen_range(min_duration, max_duration))
        };
        Infection {
            pathogen,
            infection_age: Age::new(0, 0 ,0),
            predetermined_duration: duration,
            pathogen_count: 100,
            recovered: false
        }
    }

    pub fn get_pathogen(&self) -> &Arc<Pathogen> {
        &self.pathogen
    }

    pub fn active_case(&self) -> bool {
        !self.recovered && self.pathogen_count > self.pathogen.min_count_for_symptoms
    }


    pub fn recovered(&self) -> bool {
        self.recovered
    }

    pub fn attempt_recover(&mut self) {
        if self.predetermined_duration <= self.infection_age.time_unit() {
            self.recovered = true;
        }
    }

    pub fn infection_age(&self) -> &Age {
        &self.infection_age
    }
}

impl Update for Infection {
    fn update_self(&mut self, delta_time: usize) {
        let time_passed = tick_to_game_time_conversion(delta_time);
        self.infection_age += time_passed;
        if self.pathogen_count < self.pathogen.min_count_for_symptoms {
            if roll(self.pathogen.internal_spread_rate) {
                self.pathogen_count += (rand::thread_rng().gen_range::<f64, f64, f64>(0.2, 1.02) * self.pathogen_count as f64) as usize;
            }
        } else {
            self.attempt_recover();
        }
    }

}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::rc::Rc;
    use std::sync::Arc;

    use structure::graph::Graph;

    use crate::game::pathogen::infection::Infection;
    use crate::game::pathogen::Pathogen;
    use crate::game::Update;

    /// Checks if an infection will eventually become mature
    #[test]
    fn infection_starts() {
        let pathogen = Arc::new(Pathogen::default());

        let mut infection = Infection::new(pathogen.clone(), 1.0);
        let mut time = std::time::SystemTime::now();

        while infection.pathogen_count < pathogen.min_count_for_symptoms {
            if let Ok(elapsed) = time.elapsed() {
                if elapsed.as_secs() > 30 {
                    panic!("Infections can't progress")
                }
            } else {
                panic!()
            }
            infection.update(20);
        }
    }
}