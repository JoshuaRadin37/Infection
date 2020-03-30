use std::ops::AddAssign;
use std::thread::sleep;
use std::time::Duration;

use crate::game::time::TimeUnit;

pub mod board;
pub mod graph;
pub mod population;
pub mod time;
pub mod pathogen;
pub mod playable;
pub mod doctors;


pub static LAND_TRAVEL_TIME: f64 = 45.0;
pub static SEA_TRAVEL_TIME: f64 = 100.0;
pub static AIR_TRAVEL_TIME: f64 = 500.0;

const TICKS_TO_GAME_MIN: usize = 20;

pub trait Update {


    fn update_self(&mut self, delta_time: usize);
    fn get_update_children(&mut self) -> Vec<&mut dyn Update>;

    fn update(&mut self, delta_time: usize) {
        self.update_self(delta_time);
        for child in self.get_update_children() {
            child.update(delta_time);
        }
    }


}

/// forces time passed to be at minimum one game minute
pub fn min_wait(delta_time: &mut usize) {
    while delta_time < &mut TICKS_TO_GAME_MIN {
        tick();
        delta_time.add_assign(1);
    }
}


/// An in game tick
pub fn tick() {
    sleep(Duration::from_millis(1000 / 20));
}

pub fn tick_to_game_time_conversion(delta_time: usize) -> TimeUnit {
    TimeUnit::Minutes(delta_time / TICKS_TO_GAME_MIN)
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use crate::game::Update;

    struct UpdateObject(i32, Box<Option<(UpdateObject, UpdateObject)>>);

    impl UpdateObject {
        fn new(children: Option<(UpdateObject, UpdateObject)>) -> Self {
            UpdateObject(0, Box::new(children))
        }

        fn linearized(&self) -> Vec<&i32> {
            let mut output = vec![&self.0];
            if let Some((ref left, ref right)) = *self.1 {
                output.append(&mut left.linearized());
                output.append(&mut right.linearized());
            }
            output
        }
    }

    impl Update for UpdateObject {
        fn update_self(&mut self, _: usize) {
            self.0 += 1;
        }

        fn get_update_children(&mut self) -> Vec<&mut dyn Update> {
            let mut output: Vec<&mut dyn Update> = Vec::new();
            if let Some((ref mut left, ref mut right)) = *self.1 {
                output.push(left);
                output.push(right);
            }
            output
        }
    }

    #[test]
    fn update_tree() {
        let mut tree = UpdateObject::new(
            Some(
                (
                    UpdateObject::new(None),
                    UpdateObject::new(Some(
                        (
                            UpdateObject::new(None),
                            UpdateObject::new(None)
                        )
                    ))
                )
            )
        );
        let actual = vec![&0, &0, &0, &0, &0];
        assert_eq!(tree.linearized(), actual);
        tree.update(0);
        let actual = vec![&1, &1, &1, &1, &1];
        assert_eq!(tree.linearized(), actual);
    }
}

