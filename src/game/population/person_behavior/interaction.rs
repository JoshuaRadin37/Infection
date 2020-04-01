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
        let pop_size = population.get_total_population();

        population.get_infected().iter().par_bridge().for_each(
            |person | {
                let infected = &*match person.read() {
                    Ok(i) => { i },
                    Err(_) => { panic!("Poisoned") },
                };


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
                let count = 1;// thread_rng().gen_range(0, 7);

                'outer:
                for _ in 0..count {

                    if roll(INTERACTION_CHANCE * severity_effect) { // Whether the person actually interacts with a person

                        if let Some((arc, mut other)) = {
                            let output = {
                                let mut output = None;
                                'inner: for i in 0..pop_size {
                                    let everyone = population.get_everyone();
                                    let roll = thread_rng().gen_range(0, everyone.len());  // randomly choose a person
                                    let arc = everyone.get(roll);

                                    if arc.is_none() { continue; } // if it doesn't even get a person, try again

                                    let mut arc = arc.unwrap(); // we know we have some value

                                    match arc.try_write() { // if we can get write access, we can infect it
                                        Ok(write_guard) => {
                                            output = Some((arc, write_guard));
                                            break 'inner;
                                        },
                                        Err(_) => {},
                                    }
                                }
                                output
                            };


                            output
                        } {
                            if infected.interact_with(&mut *other) {// performs an interaction with the other person
                                // person was infected

                                new_add.lock().unwrap().push(arc.clone());
                            }
                        } else {
                            // didn't pick up anything
                            break 'outer;
                        }

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


