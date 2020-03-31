use std::rc::Rc;
use std::sync::Arc;

use rand::distributions::Distribution;
use rand::Rng;

use crate::game::{roll, tick_to_game_time_conversion, Update};
use crate::game::pathogen::Pathogen;
use crate::game::time::{Age, TimeUnit};

#[derive(Clone)]
pub struct Infection {
    pathogen: Arc<Pathogen>, // pathogen
    infection_age: Age, // age of the infection
    pathogen_count: usize,
    recovered: bool // if the person has recovered
}

impl Infection {

    pub fn new(pathogen: Arc<Pathogen>) -> Self {
        Infection {
            pathogen,
            infection_age: Age::new(0, 0 ,0),
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
        let ceiling = self.pathogen.recover_chance(self.infection_age.time_unit().clone());

        self.recovered = roll(ceiling)
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

    fn get_update_children(&mut self) -> Vec<&mut dyn Update> {
        Vec::new()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::rc::Rc;
    use std::sync::Arc;

    use crate::game::graph::Graph;
    use crate::game::pathogen::infection::Infection;
    use crate::game::pathogen::Pathogen;
    use crate::game::Update;

    /// Checks if an infection will eventually become mature
    #[test]
    fn infection_starts() {
        let pathogen = Arc::new(Pathogen::new("Testogen".to_string(),
                                             1000,
                                             0.0005,
                                             0.03,
                                             1.0,
                                             Graph::new(),
                                             HashSet::new()
        ));

        let mut infection = Infection::new(pathogen.clone());
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