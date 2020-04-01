mod community {
    use std::sync::{Arc, Mutex};
    use std::thread;

    use infection::game::pathogen::Pathogen;
    use infection::game::pathogen::symptoms::base::cheat::{
        CustomCatchChance, CustomDuration, CustomFatality, CustomSeverity, CustomSpread,
    };
    use infection::game::pathogen::symptoms::Symp;
    use infection::game::pathogen::types::{PathogenType, Virus};
    use infection::game::population::{PersonBuilder, Population, UniformDistribution};
    use infection::game::population::person_behavior::Controller;
    use infection::game::population::person_behavior::interaction::InteractionController;
    use infection::game::Update;

    #[test]
    fn community_transfer() {
        let mut pop = Population::new(
            &PersonBuilder::new(),
            0.0,
            10000,
            UniformDistribution::new(0, 120),
        );
        let mut pathogen = Arc::new(Virus.create_pathogen("Test", 100));

        // start with 10 infected
        for _ in 0..10 {
            assert!(pop.infect_one(&pathogen));
        }

        let pop_arc = Arc::new(Mutex::new(pop));

        let mut controller = InteractionController::new(&pop_arc);

        let mut loops = 0;
        let spread = loop {
            {
                let mut mutex_guard = pop_arc
                    .lock()
                    .expect("Should be able to get the mutex occasionally");
                let infected_count = mutex_guard.get_all_ever_infected();
                if infected_count >= 5000 {
                    break true;
                } else if mutex_guard.get_infected().len() == 0 {
                    break false;
                }

                mutex_guard.update(20);
            }
            controller.run();
            println!(
                "Infected/Recovered Count = {}",
                pop_arc.lock().unwrap().get_all_ever_infected()
            );
            println!(
                "Infected Count = {}",
                pop_arc.lock().unwrap().get_infected().len()
            );
            loops += 1;
        };
        println!("Took {} loops to complete", loops);
        assert!(
            spread,
            "Pathogen failed to spread to half the population and instead died"
        );
        assert!(!pop_arc.is_poisoned());
    }

    #[test]
    fn community_recover() {
        let mut pop = Population::new(
            &PersonBuilder::new(),
            0.0,
            1000,
            UniformDistribution::new(0, 120),
        );
        let mut pathogen = Arc::new(Virus.create_pathogen("Test", 100));

        for _ in 0..pop.get_total_population() {
            assert!(pop.infect_one(&pathogen));
        }

        let pop_arc = Arc::new(Mutex::new(pop));

        let mut controller = InteractionController::new(&pop_arc);

        let mut loops = 0;
        let spread = loop {
            {
                let mut mutex_guard = pop_arc
                    .lock()
                    .expect("Should be able to get the mutex occasionally");
                let infected_count = mutex_guard.get_infected().len();
                if infected_count == 0 {
                    break true;
                }

                mutex_guard.update(20);
            }
            controller.run();
            println!(
                "Infected Count = {}",
                pop_arc.lock().unwrap().get_infected().len()
            );
            loops += 1;
        };
        assert!(spread, "Pathogen failed to die out");
        assert!(!pop_arc.is_poisoned());
        println!("Took {} loops to complete", loops);
    }

    #[test]
    #[ignore]
    fn community_recover_big_test() {
        for i in 0..100 {
            let mut pop = Population::new(
                &PersonBuilder::new(),
                0.0,
                100,
                UniformDistribution::new(0, 120),
            );
            let mut pathogen = Arc::new(Virus.create_pathogen("Test", 100));

            for _ in 0..pop.get_total_population() {
                assert!(pop.infect_one(&pathogen));
            }

            let pop_arc = Arc::new(Mutex::new(pop));

            let mut controller = InteractionController::new(&pop_arc);

            let mut loops = 0;
            let spread = loop {
                {
                    let mut mutex_guard = pop_arc
                        .lock()
                        .expect("Should be able to get the mutex occasionally");
                    let infected_count = mutex_guard.get_infected().len();
                    if infected_count == 0 {
                        break true;
                    }

                    mutex_guard.update(20);
                }
                controller.run();
            };
            assert!(spread, "Pathogen failed to die out");
            assert!(!pop_arc.is_poisoned());
            println!("Completed Run {}", i);
        }
    }

    #[test]
    fn full_single_community_run() {
        let mut pop = Population::new(
            &PersonBuilder::new(),
            0.0,
            10000,
            UniformDistribution::new(0, 120),
        );
        let mut pathogen = {
            loop {
                let output = Arc::new(Virus.create_pathogen("Test", 100));

                if output.catch_chance() >= 0.01 && output.catch_chance() <= 0.1 {
                    break output;
                }
            }
        };

        // start with 50 infected
        for _ in 0..50 {
            assert!(pop.infect_one(&pathogen));
        }

        let pop_arc = Arc::new(Mutex::new(pop));

        let mut controller = InteractionController::new(&pop_arc);

        let mut loops = 0;
        let spread = loop {
            {
                let mut mutex_guard = pop_arc
                    .lock()
                    .expect("Should be able to get the mutex occasionally");
                let infected_count = mutex_guard.get_all_ever_infected();
                if infected_count >= mutex_guard.get_everyone().len() / 2 {
                    break true;
                } else if mutex_guard.get_infected().len() == 0 {
                    break false;
                }

                mutex_guard.update(20 * 15);
            }
            controller.run();
            println!("Loop {}:", loops);
            println!(
                "Infected/Recovered Count = {}",
                pop_arc.lock().unwrap().get_all_ever_infected()
            );
            println!(
                "Infected Count = {}",
                pop_arc.lock().unwrap().get_infected().len()
            );
            loops += 1;
        };
        println!("Took {} loops to reach 5000 infections", loops);
        assert!(
            spread,
            "Pathogen failed to spread to half the population and instead died"
        );
        assert!(!pop_arc.is_poisoned());

        let spread = loop {
            {
                let mut mutex_guard = pop_arc
                    .lock()
                    .expect("Should be able to get the mutex occasionally");
                let infected_count = mutex_guard.get_infected().len();
                if infected_count == 0 {
                    break true;
                }

                mutex_guard.update(20 * 15);
            }
            controller.run();
            println!("Loop {}:", loops);
            println!(
                "Infected/Recovered Count = {}",
                pop_arc.lock().unwrap().get_all_ever_infected()
            );
            println!(
                "Infected Count = {}",
                pop_arc.lock().unwrap().get_infected().len()
            );
            loops += 1;
        };
        assert!(spread, "Pathogen failed to die out");
        assert!(!pop_arc.is_poisoned());
        println!(
            "Infected/Recovered Count = {}",
            pop_arc.lock().unwrap().get_all_ever_infected()
        );
        println!("Took {} loops to complete", loops);
    }

    #[test]
    fn full_single_community_run_with_deadly() {
        let mut pop = Population::new(
            &PersonBuilder::new(),
            0.0,
            10000,
            UniformDistribution::new(0, 60),
        );
        let mut pathogen = Pathogen::default();
        pathogen.acquire_symptom(&CustomFatality(5.0).get_symptom(), None);
        pathogen.acquire_symptom(&CustomCatchChance(10.0).get_symptom(), None);

        let pathogen = Arc::new(pathogen);

        run_pop(pop, &pathogen);
    }

    #[test]
    fn full_single_community_run_with_severity() {
        let mut pop = Population::new(
            &PersonBuilder::new(),
            0.0,
            10000,
            UniformDistribution::new(0, 60),
        );
        let mut pathogen = Pathogen::default();
        pathogen.acquire_symptom(&CustomSeverity(90.0).get_symptom(), None);
        pathogen.acquire_symptom(&CustomDuration(0.3).get_symptom(), None);
        pathogen.acquire_symptom(&CustomSpread(0.3).get_symptom(), None);
        pathogen.acquire_symptom(&CustomCatchChance(10.0).get_symptom(), None);

        let pathogen = Arc::new(pathogen);

        run_pop(pop, &pathogen);
    }

    #[test]
    fn full_single_community_run_with_severity_and_deadly() {
        let mut pop = Population::new(
            &PersonBuilder::new(),
            0.0,
            10000,
            UniformDistribution::new(0, 60),
        );
        let mut pathogen = Pathogen::default();
        pathogen.acquire_symptom(&CustomSeverity(83.0).get_symptom(), None);
        // pathogen.acquire_symptom(&CustomSeverity(75.0).get_symptom(), None);
        pathogen.acquire_symptom(&CustomDuration(0.3).get_symptom(), None);
        pathogen.acquire_symptom(&CustomSpread(0.3).get_symptom(), None);
        pathogen.acquire_symptom(&CustomFatality(0.4).get_symptom(), None);
        pathogen.acquire_symptom(&CustomCatchChance(0.7).get_symptom(), None);

        let pathogen = Arc::new(pathogen);

        run_pop(pop, &pathogen);
    }

    #[test]
    #[ignore]
    fn full_big_community_run_with_severity_and_deadly() {
        let mut pop = Population::new(
            &PersonBuilder::new(),
            0.0,
            100_000,
            UniformDistribution::new(0, 60),
        );
        let mut pathogen = Pathogen::default();
        pathogen.acquire_symptom(&CustomSeverity(87.0).get_symptom(), None);
        // pathogen.acquire_symptom(&CustomSeverity(75.0).get_symptom(), None);
        pathogen.acquire_symptom(&CustomDuration(0.3).get_symptom(), None);
        pathogen.acquire_symptom(&CustomSpread(0.3).get_symptom(), None);
        pathogen.acquire_symptom(&CustomFatality(1.7).get_symptom(), None);
        pathogen.acquire_symptom(&CustomCatchChance(1.0).get_symptom(), None);

        let pathogen = Arc::new(pathogen);

        run_pop(pop, &pathogen);
    }

    fn run_pop(mut pop: Population, pathogen: &Arc<Pathogen>) {
        // start with 50 infected
        for _ in 0..10 {
            assert!(pop.infect_one(&pathogen));
        }
        let pop_arc = Arc::new(Mutex::new(pop));
        {
            let pop = pop_arc.lock().unwrap();
            println!("Infected/Recovered Count = {}", pop.get_all_ever_infected());
            println!(
                "Death Count = {}",
                pop.get_original_population() - pop.get_total_population()
            );
            println!("Infected Count = {}", pop.get_infected().len());
        }
        let mut controller = InteractionController::new(&pop_arc);
        let mut loops = 0;
        let spread = loop {
            {
                let mut mutex_guard = pop_arc
                    .lock()
                    .expect("Should be able to get the mutex occasionally");
                let infected_count = mutex_guard.get_all_ever_infected();
                if infected_count >= mutex_guard.get_everyone().len() / 2 {
                    break true;
                } else if mutex_guard.get_infected().len() == 0 {
                    break false;
                }

                mutex_guard.update(20);
            }
            controller.run();
            println!("Loop {}:", loops);
            let pop = pop_arc.lock().unwrap();
            println!("Infected/Recovered Count = {}", pop.get_all_ever_infected());
            println!(
                "Death Count = {}",
                pop.get_original_population() - pop.get_total_population()
            );
            println!("Infected Count = {}", pop.get_infected().len());
            loops += 1;
        };
        // println!("Took {} loops to reach 5000 infections", loops);

        assert!(!pop_arc.is_poisoned());
        let spread = loop {
            {
                let mut mutex_guard = pop_arc
                    .lock()
                    .expect("Should be able to get the mutex occasionally");
                let infected_count = mutex_guard.get_infected().len();
                if infected_count == 0 {
                    break true;
                }

                mutex_guard.update(20);
            }
            controller.run();
            println!("Loop {}:", loops);
            let pop = pop_arc.lock().unwrap();
            println!("Infected/Recovered Count = {}", pop.get_all_ever_infected());
            println!(
                "Death Count = {}",
                pop.get_original_population() - pop.get_total_population()
            );
            println!("Infected Count = {}", pop.get_infected().len());
            loops += 1;
        };
        assert!(spread, "Pathogen failed to die out");
        assert!(!pop_arc.is_poisoned());
        println!(
            "Infected/Recovered Count = {}",
            pop_arc.lock().unwrap().get_all_ever_infected()
        );
        let pop = pop_arc.lock().unwrap();
        println!(
            "Death Count = {}",
            pop.get_original_population() - pop.get_total_population()
        );
        println!(
            "% Infected = {}%",
            (pop.get_all_ever_infected() + pop.get_original_population()
                - pop.get_total_population()) as f64
                / pop.get_original_population() as f64
                * 100.0
        );
        println!(
            "Mortality Rate = {}%",
            (pop.get_original_population() - pop.get_total_population()) as f64
                / (pop.get_all_ever_infected() + pop.get_original_population()
                    - pop.get_total_population()) as f64
                * 100.0
        );
        println!("Took {} loops to complete", loops);
    }
}
