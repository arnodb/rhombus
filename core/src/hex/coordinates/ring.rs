use crate::hex::coordinates::{
    direction::{HexagonalDirection, NUM_DIRECTIONS},
    HexagonalVector,
};

pub struct RingIter<V: HexagonalVector + HexagonalDirection> {
    edge_length: usize,
    direction: usize,
    next: V,
    edge_index: usize,
}

impl<V: HexagonalVector + HexagonalDirection> RingIter<V> {
    pub fn new(radius: usize, center: V) -> Self {
        Self {
            edge_length: radius,
            direction: 0,
            next: center + V::direction(4) * radius as isize,
            edge_index: 1,
        }
    }

    pub fn peek(&mut self) -> Option<&V> {
        if self.direction < NUM_DIRECTIONS {
            Some(&self.next)
        } else {
            None
        }
    }
}

impl<V: HexagonalDirection> Iterator for RingIter<V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        let edge_length = self.edge_length;
        let direction = self.direction;
        if direction < NUM_DIRECTIONS {
            let next = self.next;
            self.next = next.neighbor(direction);
            let ei = self.edge_index;
            if ei < edge_length {
                self.edge_index = ei + 1;
            } else {
                self.edge_index = 1;
                self.direction = direction + 1;
                while self.direction < NUM_DIRECTIONS && edge_length == 0 {
                    self.direction += 1;
                }
            }
            Some(next)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let el = self.edge_length;
        if el > 0 {
            let length = el * 6;
            (length, Some(length))
        } else {
            (1, Some(1))
        }
    }
}

pub struct BigRingIter<V: HexagonalDirection> {
    edge_length: usize,
    direction: usize,
    direction_vector: V,
    next: V,
    edge_index: usize,
    cell_radius: usize,
}

impl<V: HexagonalDirection> BigRingIter<V> {
    pub fn new(cell_radius: usize, radius: usize, center: V) -> Self {
        let direction_vector =
            V::direction(0) * (cell_radius as isize + 1) + V::direction(1) * cell_radius as isize;
        let next = center
            + (V::direction(4) * (cell_radius as isize + 1)
                + V::direction(5) * cell_radius as isize)
                * radius as isize;
        Self {
            edge_length: radius,
            direction: 0,
            direction_vector,
            next,
            edge_index: 1,
            cell_radius,
        }
    }

    pub fn peek(&mut self) -> Option<&V> {
        if self.direction < 6 {
            Some(&self.next)
        } else {
            None
        }
    }
}

impl<V: HexagonalDirection> Iterator for BigRingIter<V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        let edge_length = self.edge_length;
        let direction = self.direction;
        if direction < 6 {
            let next = self.next;
            self.next = next + self.direction_vector;
            let ei = self.edge_index;
            if ei < edge_length {
                self.edge_index = ei + 1;
            } else {
                self.edge_index = 1;
                self.direction = direction + 1;
                while self.direction < NUM_DIRECTIONS && edge_length == 0 {
                    self.direction += 1;
                }
                if self.direction < 6 {
                    self.direction_vector = V::direction(self.direction)
                        * (self.cell_radius as isize + 1)
                        + V::direction((self.direction + 1) % NUM_DIRECTIONS)
                            * self.cell_radius as isize;
                }
            }
            Some(next)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let el = self.edge_length;
        if el > 0 {
            let length = el * 6;
            (length, Some(length))
        } else {
            (1, Some(1))
        }
    }
}
