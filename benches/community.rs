#[macro_use]
extern crate criterion;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput};

use infection::game::{ParallelUpdate, Update};
use infection::game::pathogen::Pathogen;
use infection::game::pathogen::symptoms::base::cheat::{
    CustomCatchChance, CustomInternalSpreadRate,
};
use infection::game::pathogen::symptoms::Symp;
use infection::game::pathogen::types::{PathogenType, Virus};
use infection::game::population::{PersonBuilder, Population, UniformDistribution};
use infection::game::population::person_behavior::Controller;
use infection::game::population::person_behavior::interaction::InteractionController;

fn infected_population_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("Population update stress");
    for size in &[10, 100, 1000, 10000] {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut pop = Population::new(
                &PersonBuilder::new(),
                0.0,
                size,
                UniformDistribution::new(0, 120),
            );
            b.iter(|| pop.update(20))
        });
    }
}

fn community(c: &mut Criterion) {
    let mut pop = Population::new(
        &PersonBuilder::new(),
        0.0,
        10000,
        UniformDistribution::new(0, 120),
    );
    let mut p = Pathogen::default();
    p.acquire_symptom(&CustomCatchChance(20.0).get_symptom(), None);
    p.acquire_symptom(&CustomInternalSpreadRate(-99.0).get_symptom(), None);
    let mut pathogen = Arc::new(p);

    // start with 1 infected
    for _ in 0..1 {
        assert!(pop.infect_one(&pathogen));
    }

    let pop_arc = Arc::new(Mutex::new(pop));
    let loops = Arc::new(Mutex::new(0));

    let mut controller = InteractionController::new(&pop_arc);

    c.bench_function("Community spread cycle", |b| {
        b.iter(|| {
            {
                let mut mutex_guard = pop_arc
                    .lock()
                    .expect("Should be able to get the mutex occasionally");
                mutex_guard.update(20);
            }
            controller.run();
            let guard = pop_arc.lock().unwrap();
            {
                let mut loops_guard = loops.lock().unwrap();
                // println!("Loop {}: Infected% = {:1.3}%", *loops_guard, guard.get_infected().len() as f64 / guard.get_everyone().len() as f64 * 100.0);
                *loops_guard += 1;
            }
        })
    });
}

criterion_group!(community_benches, community, infected_population_update);
criterion_main!(community_benches);
