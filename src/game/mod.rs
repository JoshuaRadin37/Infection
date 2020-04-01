use std::borrow::Borrow;
use std::cmp::Ordering;
use std::ops::{AddAssign, Deref};
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;

use rand::Rng;
use rayon::prelude::*;

use structure::time::{FineGrainTimeType, Time, TimeUnit, YearsType};
use structure::time::TimeUnit::{Days, Months, Years};

use crate::game;

pub mod board;
pub mod population;
pub mod pathogen;
pub mod playable;
pub mod doctors;


pub static LAND_TRAVEL_TIME: f64 = 45.0;
pub static SEA_TRAVEL_TIME: f64 = 100.0;
pub static AIR_TRAVEL_TIME: f64 = 500.0;

const TICKS_TO_GAME_MIN: usize = 20;

pub trait Update<T=Self> where T : Update<T> {


    fn update_self(&mut self, delta_time: usize);
    fn get_update_children(&mut self) -> Vec<&mut T> {
        Vec::new()
    }

    fn update(&mut self, delta_time: usize) {
        self.update_self(delta_time);
        for child in self.get_update_children() {
            child.update(delta_time);
        }
    }



}

impl <T> Update for Arc<RwLock<T>> where T : Update<T> {
    fn update_self(&mut self, delta_time: usize) {
        self.write().unwrap().update_self(delta_time)
    }
}


impl <T> Update for RwLock<T> where T : Update<T> {
    fn update_self(&mut self, delta_time: usize) {
        self.write().unwrap().update_self(delta_time)
    }
}

impl <T, R> Update<R> for T where R : Send + Update<R>,
T : ParallelUpdate<R> {
    fn update_self(&mut self, delta_time: usize) {
        ParallelUpdate::parallel_update_self(self, delta_time)
    }

    fn get_update_children(&mut self) -> Vec<&mut R> {
        self.parallel_get_update_children()
    }


    fn update(&mut self, delta_time: usize) {
        ParallelUpdate::parallel_update(self, delta_time)
    }
}

const USE_PARALLEL: bool = true;

pub trait ParallelUpdate<T=Self>
    where T : Send + Update<T> {

    fn parallel_update_self(&mut self, delta_time: usize);
    fn parallel_get_update_children(&mut self) -> Vec<&mut T> {
        Vec::new()
    }

    fn parallel_update(&mut self, delta_time: usize) {
        self.parallel_update_self(delta_time);
        self.parallel_get_update_children().par_iter_mut().for_each(
            |child| child.update(delta_time)
        )
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

pub fn roll(chance: f64) -> bool {
    if chance < 0.0 || chance > 1.0 {
        panic!("Invalid chance: {}", chance);
    }
    rand::thread_rng().gen_bool(chance)
}

#[derive(Debug, Clone)]
pub struct Age(TimeUnit); // in minutes

impl Age {

    pub fn new(years: YearsType, months: FineGrainTimeType, days: FineGrainTimeType) -> Age {
        let years = Years(years).into_minutes();
        let months = Months(months).into_minutes();
        let days = Days(days).into_minutes();

        Age(years + months + days)
    }

    pub fn time_unit(&self) -> &TimeUnit {
        &self.0
    }

    pub fn time_unit_mut(&mut self) -> &mut TimeUnit {
        &mut self.0
    }
}

impl From<TimeUnit> for Age {
    fn from(t: TimeUnit) -> Self {
        Age(t.into_minutes())
    }
}

impl AddAssign<TimeUnit> for Age {
    fn add_assign(&mut self, rhs: TimeUnit) {
        self.0 = &self.0 + rhs;
    }
}

impl AddAssign<&TimeUnit> for Age {
    fn add_assign(&mut self, rhs: &TimeUnit) {
        self.0 = &self.0 + rhs;
    }
}

impl AddAssign<usize> for Age {
    fn add_assign(&mut self, rhs: usize) {
        self.0 = &self.0 + rhs;
    }
}

impl PartialEq<TimeUnit> for Age {
    fn eq(&self, other: &TimeUnit) -> bool {
        self.time_unit().eq(other)
    }
}

impl PartialOrd<TimeUnit> for Age {
    fn partial_cmp(&self, other: &TimeUnit) -> Option<Ordering> {
        self.time_unit().partial_cmp(other)
    }
}

impl Update for Age {
    fn update_self(&mut self, delta_time: usize) {
        *self += game::tick_to_game_time_conversion(delta_time);
        //self.add_assign();
    }


}


#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use structure::time::TimeUnit::{Days, Minutes, Years};

    use crate::game::{Age, Update};

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

        fn get_update_children(&mut self) -> Vec<&mut Self> {
            let mut output: Vec<&mut Self> = Vec::new();
            if let Some((ref mut left, ref mut right)) = *self.1 {
                output.push(left);
                output.push(right);
            }
            output
        }
    }

    #[test]
    fn age_modification() {
        let mut age: Age = (Years(21) + Days(21)).into();
        assert_eq!(age, Years(21) + Days(21));
        age += Minutes(1);
        assert_eq!(age, Years(21) + Days(21) + Minutes(1));
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

