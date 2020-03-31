use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};

use rand::{Rng, thread_rng};
use rand::seq::IteratorRandom;

use crate::game::pathogen::infection::Infection;
use crate::game::population::person_behavior::Controller;
use crate::game::population::Population;
use crate::game::roll;

pub struct InteractionController {
    population: Arc<Mutex<Population>>
}

impl InteractionController {

    pub fn new(population: &Arc<Mutex<Population>>) -> Self {
        Self {
            population: population.clone()
        }
    }

}

const INTERACTION_CHANCE: f64 = 1.0;

impl Controller for InteractionController {
    fn run(&mut self) {

        let mut _population = self.population.lock().expect("Should have been able to receive population");
        let population = &mut *_population;

        let mut new_add = vec![];

        for person in population.get_infected() {
            let infected = &*person.read().expect("Should be able to get a read");

            let severity = {
                let guard = infected.infection.lock().unwrap();
                match &*guard {
                    None => { panic!("There should be an infection") },

                    Some(ref i) => {
                        *i.get_pathogen().severity()
                    },
                }
            };
            let severity_effect = 1.0 - severity;
            let count = thread_rng().gen_range(0, 7);
            /*
            for _ in 0..count {

                if roll(INTERACTION_CHANCE * severity_effect) {
                    let other = loop {
                        let other = population.get_everyone().iter().choose(&mut thread_rng());

                        if let Some(arc) = other {
                            let other = &*arc.read().unwrap();

                            if infected != other {
                                break arc;
                            }
                        }
                    };


                    if infected.interact_with(&mut *other.write().expect("Should be able to get the person")) {// performs an interaction with the other person
                        // person was infected

                        new_add.push(other.clone());
                    }
                }
            }
            */
        }


        for person in new_add {
            population.infected.push(person);
        }
    }
}

#[cfg(test)]
mod test {

}


