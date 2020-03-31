#[macro_use]
extern crate criterion;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use criterion::Criterion;

use infection::game::pathogen::types::{PathogenType, Virus};
use infection::game::population::{PersonBuilder, Population, UniformDistribution};
use infection::game::population::person_behavior::Controller;
use infection::game::population::person_behavior::interaction::InteractionController;
use infection::game::Update;

fn community(c: &mut Criterion) {
    let mut pop = Population::new(&PersonBuilder::new(), 0.0, 100000, UniformDistribution::new(0, 120));
    let mut pathogen = Arc::new(Virus.create_pathogen("Test", 100));

    // start with 10 infected
    for _ in 0..10 {
        assert!(pop.infect_one(&pathogen));
    }


    let pop_arc = Arc::new(Mutex::new(pop));
    let loops = Arc::new(Mutex::new(0));

    let mut controller = InteractionController::new(&pop_arc);

    c.bench_function("Community spread cycle", |b| b.iter(
        || {
            {
                let mut mutex_guard = pop_arc.lock().expect("Should be able to get the mutex occasionally");
                mutex_guard.update(20);
            }
            controller.run();
            let guard = pop_arc.lock().unwrap();
            {
                let mut loops_guard = loops.lock().unwrap();
                println!("Loop {}: Infected% = {:1.3}%", *loops_guard, guard.get_infected().len() as f64 / guard.get_everyone().len() as f64 * 100.0);
                *loops_guard += 1;
            }
        }
    ));


}


criterion_group!(community_benches, community);
criterion_main!(community_benches);
