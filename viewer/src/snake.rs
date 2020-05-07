use std::collections::VecDeque;

pub struct Snake<V, I> {
    pub radius: usize,
    pub state: VecDeque<V>,
    pub iter: I,
}
