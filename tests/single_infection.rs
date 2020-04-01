use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use infection::game::pathogen::infection::Infection;
use infection::game::pathogen::Pathogen;
use infection::game::pathogen::symptoms::{Symptom, SymptomMap};
use infection::game::Update;
use structure::graph::Graph;
use structure::time::Time;
use structure::time::TimeUnit::Minutes;

const ATTEMPTS: usize = 100;

#[test]
fn infection_recovery_test() {
    let pathogen = Arc::new(Pathogen::default());

    let mut sum_time = Minutes(0);
    let mut times = Vec::new();

    for attempt in 0..ATTEMPTS {
        let mut infection = Infection::new(pathogen.clone(), 1.0);

        while !&infection.recovered() {
            infection.update(20);
        }

        let recover_time = infection.infection_age().time_unit().clone();
        println!("Attempt {} Recover Time: {}", attempt, recover_time.format("{:d} ({:m} minutes)"));
        sum_time = sum_time + &recover_time;
        times.push(recover_time);
    }
    let avg_time = sum_time / ATTEMPTS;
    assert!(avg_time.as_days() >= 3 && avg_time.as_days() < 6, "Aiming for default recover time to be between 3 and 6 days, instead {} ({} minutes)", avg_time.format("{:d}"), avg_time);
    println!("Average recovery time = {}", avg_time.format("{:d}d:{:h(24h)}h:{:m(60m)}m"));
    let mut variance: f64 = times.into_iter().map(|time|
        ((usize::from(time) as isize - usize::from(&avg_time) as isize) as f64).powi(2)
    ).sum();
    variance = variance / ATTEMPTS as f64;
    let std_dev = variance.sqrt() as usize;
    let std_dev_time = Minutes(std_dev);
    // println!("Recovery time standard deviation = {}", std_dev_time.format("{:d}d:{:h(24h)}h:{:m(60m)}m"));

}