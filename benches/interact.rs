#[macro_use]
extern crate criterion;

use std::ops::Range;
use std::sync::{Arc, Mutex};

use criterion::{BenchmarkId, Criterion, Throughput};

use infection::game::pathogen::symptoms::base::cheat::NoSpread;
use infection::game::pathogen::symptoms::Symp;
use infection::game::pathogen::types::{PathogenType, Virus};
use infection::game::population::{PersonBuilder, Population, UniformDistribution};
use infection::game::population::person_behavior::Controller;
use infection::game::population::person_behavior::interaction::InteractionController;
use infection::game::Update;

fn interact(pop: usize, infected: usize) {
    let mut pop = Population::new(&PersonBuilder::new(), 0.0, pop, UniformDistribution::new(0, 120));
    let mut pathogen = Virus.create_pathogen("Test", 100);
    pathogen.acquire_symptom(&NoSpread.get_symptom(), None); // Disable spread

    let pathogen = Arc::new(pathogen);
    for _ in 0..infected {
        pop.infect_one(&pathogen);
    }

    let mut controller = InteractionController::new(&Arc::new(Mutex::new(pop)));

    controller.run();

}



fn interaction_100(c: &mut Criterion) {


    let pop = 100;

    let mut group = c.benchmark_group(format!("infected {}", pop));
    let mut sizes = vec![];
    for i in 0..11 {
        sizes.push(pop * i / 10);
    }


    for size in sizes.as_slice() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            format!("p:{} i:{}", pop, size),
            size,
            |b, &size| {
                b.iter(|| interact(pop, size))
            }
        );
    }



}

fn interaction_1000(c: &mut Criterion) {


    let pop = 1000;

    let mut group = c.benchmark_group(format!("infected {}", pop));
    let mut sizes = vec![];
    for i in 0..11 {
        sizes.push(pop * i / 10);
    }


    for size in sizes.as_slice() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            format!("p:{} i:{}", pop, size),
            size,
            |b, &size| {
                b.iter(|| interact(pop, size))
            }
        );
    }



}

criterion_group!(interact_benches, interaction_100, interaction_1000);
criterion_main!(interact_benches);