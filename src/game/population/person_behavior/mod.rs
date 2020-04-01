use std::time::Duration;

pub mod interaction;
pub mod travel;

pub trait Controller {
    fn run(&mut self);
}
