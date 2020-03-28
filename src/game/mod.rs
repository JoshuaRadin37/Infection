pub mod board;
pub mod graph;
pub mod population;
pub mod time;


pub static LAND_TRAVEL_TIME: f64 = 45.0;
pub static SEA_TRAVEL_TIME: f64 = 100.0;
pub static AIR_TRAVEL_TIME: f64 = 500.0;

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

#[cfg(test)]
mod test {
    use crate::game::Update;
    use std::borrow::BorrowMut;

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

