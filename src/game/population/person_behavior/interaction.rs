use std::io::{stdout, Write};
use std::sync::{Arc, Mutex, RwLockReadGuard, RwLockWriteGuard, TryLockError};

use rand::{Rng, thread_rng};
use rand::seq::IteratorRandom;
use rayon::prelude::*;

use crate::game::pathogen::infection::Infection;
use crate::game::population::{Person, Population};
use crate::game::population::person_behavior::Controller;
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

        let mut new_add = Arc::new(Mutex::new(vec![]));

        population.get_infected().clone().iter().par_bridge().for_each(
            |person | {
                let infected = &*person.read().expect("Should be able to get a read");

                let severity = {
                    let guard = infected.infection.lock().unwrap();
                    match &*guard {
                        None => { panic!("There should be an infection") },

                        Some(ref i) => {
                            i.get_pathogen().severity()
                        },
                    }
                };
                let severity_effect = 1.0 - severity;
                let count = thread_rng().gen_range(0, 7);

                for _ in 0..count {

                    if roll(INTERACTION_CHANCE * severity_effect) { // Whether the person actually interacts with a person

                        let (arc, mut other) =
                            'outer: loop {
                            let arc = population.get_everyone().iter().choose(&mut thread_rng()); // randomly choose a person

                            if arc.is_none() { continue; } // if it doesn't even get a person, try again

                            let mut arc = arc.unwrap(); // we know we have some value

                            match arc.try_write() { // if we can get write access, we can infect it
                                Ok(write_guard) => {
                                    break 'outer (arc, write_guard);
                                },
                                Err(_) => {},
                            }
                        };

                        if infected.interact_with(&mut *other) {// performs an interaction with the other person
                            // person was infected

                            new_add.lock().unwrap().push(arc.clone());
                        }
                        /*
                        let other = loop {
                            let other = population.get_everyone().iter().choose(&mut thread_rng());


                            if let Some(arc) = other {
                                let mut okay = false;
                                 match arc.try_read() {
                                    Ok(p) => {
                                        let other = &* p;
                                            if infected != other {
                                            okay = true;
                                        }
                                    },
                                    Err(_) => {
                                        // panic!("Tried to access person already being read")
                                    },
                                }

                                if okay {
                                    break arc;
                                }
                            }

                        };


                        if infected.interact_with(&mut *other.write().expect("Should be able to get the person")) {// performs an interaction with the other person
                            // person was infected

                            new_add.lock().unwrap().push(other.clone());
                        }
                        */
                    }
                }



            }
        );




        for person in & *new_add.lock().unwrap() {
            population.infected.push(person.clone());
        }
    }
}

#[cfg(test)]
mod test {

}


