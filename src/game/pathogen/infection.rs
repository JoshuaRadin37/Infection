use std::rc::Rc;

use rand::distributions::Distribution;
use rand::Rng;

use crate::game::{tick_to_game_time_conversion, Update};
use crate::game::pathogen::Pathogen;
use crate::game::time::{Age, TimeUnit};

pub struct Infection {
    pathogen: Rc<Pathogen>, // pathogen
    infection_age: Age, // age of the infection
    pathogen_count: usize,
    recovered: bool // if the person has recovered
}

impl Infection {

    pub fn new(pathogen: Rc<Pathogen>) -> Self {
        Infection {
            pathogen,
            infection_age: Age::new(0, 0 ,0),
            pathogen_count: 5,
            recovered: false
        }
    }

    pub fn get_pathogen(&self) -> &Pathogen {
        &self.pathogen
    }

    pub fn recovered(&self) -> bool {
        self.recovered
    }

    pub fn attempt_recover(&mut self) {
        let ceiling = self.pathogen.recovery_chance.recover(self.infection_age.time_unit().clone());
        let roll: f64 = rand::random();

        self.recovered = roll < ceiling;
    }
}

impl Update for Infection {
    fn update_self(&mut self, delta_time: usize) {
        let time_passed = tick_to_game_time_conversion(delta_time);
        self.infection_age += time_passed;
        if self.pathogen_count < self.pathogen.min_count_for_symptoms {
            if Pathogen::roll(self.pathogen.internal_spread_rate) {
                self.pathogen_count += (rand::thread_rng().gen_range::<f64, f64, f64>(0.33, 0.66) * self.pathogen_count as f64) as usize;
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
    use std::rc::Rc;

    use crate::game::{min_wait, Update};
    use crate::game::pathogen::{DefaultRecoveryChance, Pathogen};
    use crate::game::pathogen::infection::Infection;
    use crate::game::time::Time;
    use crate::game::time::TimeUnit::Minutes;

    const ATTEMPTS: usize = 100;

    #[test]
    fn infection_recovery_test() {
        let pathogen = Rc::new(Pathogen::new("Testogen", 1000, DefaultRecoveryChance));

        let mut sum_time = Minutes(0);

        for attempt in 0..ATTEMPTS {
            let mut infection = Infection::new(pathogen.clone());

            while !&infection.recovered {
                infection.update(20);
            }

            let recover_time = infection.infection_age.time_unit();
            println!("Attempt {} Recover Time: {} days", attempt, recover_time.format("{:d}"));
            sum_time = sum_time + recover_time;

        }
        let avg_time = sum_time / ATTEMPTS;
        assert!(avg_time.as_days() >= 3, "Aiming for default recover time to be 3 days, instead {} ({} minutes)", avg_time.format("{:d}"), avg_time);
        println!("Average recovery time = {}", avg_time.format("{:d}d:{:h(24h)}h:{:m(60m)}m"))
    }
}

