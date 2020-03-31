use std::time::Duration;

pub mod travel;
pub mod interaction;

pub trait Controller {
    fn run(&mut self);
}