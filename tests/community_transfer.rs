
mod community {
    use std::sync::{Arc, Mutex};
    use std::thread;

    use infection::game::pathogen::types::{PathogenType, Virus};
    use infection::game::population::{PersonBuilder, Population, UniformDistribution};
    use infection::game::population::person_behavior::Controller;
    use infection::game::population::person_behavior::interaction::InteractionController;
    use infection::game::Update;

    #[test]
    #[ignore]
    fn community_transfer() {
        let mut pop = Population::new(&PersonBuilder::new(), 0.0, 10000, UniformDistribution::new(0, 120));
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
                let mut mutex_guard = pop_arc.lock().expect("Should be able to get the mutex occasionally");
                let infected_count = mutex_guard.get_all_ever_infected();
                if infected_count >= 5000 {
                    break true;
                } else if mutex_guard.get_infected().len() == 0 {
                    break false;
                }

                mutex_guard.update(20);
            }
            controller.run();
            println!("Infected/Recovered Count = {}", pop_arc.lock().unwrap().get_all_ever_infected());
            println!("Infected Count = {}", pop_arc.lock().unwrap().get_infected().len());
            loops += 1;
        };
        println!("Took {} loops to complete", loops);
        assert!(spread, "Pathogen failed to spread to half the population and instead died");
        assert!(!pop_arc.is_poisoned());

    }


    #[test]
    #[ignore]
    fn community_recover() {
        let mut pop = Population::new(&PersonBuilder::new(), 0.0, 100, UniformDistribution::new(0, 120));
        let mut pathogen = Arc::new(Virus.create_pathogen("Test", 100));

        for _ in 0..100 {
            assert!(pop.infect_one(&pathogen));
        }

        let pop_arc = Arc::new(Mutex::new(pop));

        let mut controller = InteractionController::new(&pop_arc);

        let mut loops = 0;
        let spread = loop {
            {
                let mut mutex_guard = pop_arc.lock().expect("Should be able to get the mutex occasionally");
                let infected_count = mutex_guard.get_infected().len();
                if infected_count == 0 {
                    break true;
                }

                mutex_guard.update(20);
            }
            controller.run();
            println!("Infected Count = {}", pop_arc.lock().unwrap().get_infected().len());
            loops += 1;
        };
        assert!(spread, "Pathogen failed to die out");
        assert!(!pop_arc.is_poisoned());
        println!("Took {} loops to complete", loops);
    }

    #[test]
    #[ignore]
    fn full_single_community_run() {
        let mut pop = Population::new(&PersonBuilder::new(), 0.0, 1000, UniformDistribution::new(0, 120));
        let mut pathogen = {
            loop {
                let output = Arc::new(Virus.create_pathogen("Test", 100));

                if output.catch_chance() >= &0.01 && output.catch_chance() <= &0.1  {
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
                let mut mutex_guard = pop_arc.lock().expect("Should be able to get the mutex occasionally");
                let infected_count = mutex_guard.get_all_ever_infected();
                if infected_count >= mutex_guard.get_everyone().len() / 2{
                    break true;
                } else if mutex_guard.get_infected().len() == 0 {
                    break false;
                }

                mutex_guard.update(20*15);
            }
            controller.run();
            println!("Loop {}:", loops);
            println!("Infected/Recovered Count = {}", pop_arc.lock().unwrap().get_all_ever_infected());
            println!("Infected Count = {}", pop_arc.lock().unwrap().get_infected().len());
            loops += 1;
        };
        println!("Took {} loops to reach 5000 infections", loops);
        assert!(spread, "Pathogen failed to spread to half the population and instead died");
        assert!(!pop_arc.is_poisoned());


        let spread = loop {
            {
                let mut mutex_guard = pop_arc.lock().expect("Should be able to get the mutex occasionally");
                let infected_count = mutex_guard.get_infected().len();
                if infected_count == 0 {
                    break true;
                }

                mutex_guard.update(20*15);
            }
            controller.run();
            println!("Loop {}:", loops);
            println!("Infected/Recovered Count = {}", pop_arc.lock().unwrap().get_all_ever_infected());
            println!("Infected Count = {}", pop_arc.lock().unwrap().get_infected().len());
            loops += 1;
        };
        assert!(spread, "Pathogen failed to die out");
        assert!(!pop_arc.is_poisoned());
        println!("Infected/Recovered Count = {}", pop_arc.lock().unwrap().get_all_ever_infected());
        println!("Took {} loops to complete", loops);

    }
}