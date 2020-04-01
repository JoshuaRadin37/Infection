use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use structure::graph::Graph;
use structure::time::{Time, TimeUnit};
use structure::time::TimeUnit::Days;

use crate::game::pathogen::Pathogen;
use crate::game::pathogen::symptoms::{Symp, Symptom, SymptomMap, SymptomMapBuilder};
use crate::game::pathogen::symptoms::base::{Cough, RunnyNose};

pub trait PathogenType {

    /// Gets the prefix of the Pathogen Type
    fn get_prefix(&self) -> &str;


    fn get_min_count(&self) -> usize;
    fn get_mutativity(&self) -> f64;
    fn get_average_duration(&self) -> TimeUnit;
    fn get_duration_spread(&self) -> TimeUnit;
    fn get_symptoms_map(&self) -> (Graph<usize, f64, Arc<Symptom>>, HashSet<usize>);

    fn create_pathogen(&self, name: &str, mutation_ticks: usize) -> Pathogen {
        let fixed_name = format!("{} {}", self.get_prefix(), name);
        let (graph, set) = self.get_symptoms_map();

        let mut pathogen = Pathogen::new(fixed_name,
                                         self.get_min_count(),
                                         self.get_mutativity(),
                                         usize::from(self.get_average_duration().into_minutes()),
                                         usize::from(self.get_duration_spread().into_minutes()),
                                         graph,
                                         set);


        for _ in 0..mutation_ticks {
            pathogen = pathogen.mutate()
        }

        pathogen
    }

    fn default(&self) -> Pathogen {
        self.create_pathogen("Default", 0)
    }
}


pub struct Virus;

impl PathogenType for Virus {
    fn get_prefix(&self) -> &str {
        "Virus"
    }

    fn get_min_count(&self) -> usize {
        1_000_000
    }

    fn get_mutativity(&self) -> f64 {
        0.05
    }

    fn get_average_duration(&self) -> TimeUnit {
        Days(8)
    }

    fn get_duration_spread(&self) -> TimeUnit {
        Days(3)
    }


    fn get_symptoms_map(&self) -> (Graph<usize, f64, Arc<Symptom>>, HashSet<usize>) {
        let mut builder = SymptomMapBuilder::new();
        let mut set = HashSet::new();

        let mut builder_entry = builder.add(RunnyNose.get_symptom());
        set.insert(builder_entry.node());
        builder_entry.next_symptom(Cough(1).get_symptom(), 0.5)
            .next_symptom(Cough(2).get_symptom(), 0.02)
            .next_symptom(Cough(3).get_symptom(), 0.01);

        (builder.get_map(), set)
    }
}

#[cfg(test)]
mod test {
    use structure::time::{Time, TimeUnit};
    use structure::time::TimeUnit::Minutes;

    use crate::game::pathogen::infection::Infection;
    use crate::game::pathogen::types::{PathogenType, Virus};
    use crate::game::Update;

    use super::*;

    const ATTEMPTS: usize = 100;

    fn avg_recovery_time(pathogen: Arc<Pathogen>, min: usize, max: usize) {
        let mut sum_time = Minutes(0);
        let mut times = Vec::new();
        for attempt in 0..ATTEMPTS {
            let mut infection = Infection::new(pathogen.clone(), 1.0);

            while !&infection.recovered() {
                infection.update(20);
            }

            let recover_time = infection.infection_age().time_unit().clone();
            // println!("Attempt {} Recover Time: {} days", attempt, recover_time.format("{:d}"));
            sum_time = sum_time + &recover_time;
            times.push(recover_time);
        }
        let avg_time = sum_time / ATTEMPTS;
        assert!(avg_time.as_days() >= min && avg_time.as_days() < max, "Aiming for default recover time to be between {} and {} days, instead {} ({} minutes)", min, max, avg_time.format("{:d}"), avg_time);
        println!("Average recovery time = {}", avg_time.format("{:d}d:{:h(24h)}h:{:m(60m)}m"));
    }

    #[test]
    fn virus_avg_recovery_time() {
        let pathogen = Arc::new(Virus.default());

        avg_recovery_time(pathogen, 5, 12);
    }

    #[test]
    fn mutation_works() {

        let pathogen = Virus.create_pathogen("Test", 100);


    }
}