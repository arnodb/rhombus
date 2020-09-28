use rhombus_core::hex::coordinates::{
    axial::AxialVector,
    cubic::CubicVector,
    direction::{HexagonalDirection, NUM_DIRECTIONS},
};

#[derive(Clone, Copy, Debug)]
pub struct Range {
    start: isize,
    end: isize,
}

impl Range {
    pub fn start(&self) -> isize {
        self.start
    }

    pub fn end(&self) -> isize {
        self.end
    }

    pub fn start_mut(&mut self) -> &mut isize {
        &mut self.start
    }

    pub fn end_mut(&mut self) -> &mut isize {
        &mut self.end
    }

    pub fn contains(&self, value: isize) -> bool {
        self.start <= value && value <= self.end
    }
}

impl From<(isize, isize)> for Range {
    fn from(tuple: (isize, isize)) -> Self {
        Range {
            start: tuple.0,
            end: tuple.1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CubicRangeShape {
    range_x: Range,
    range_y: Range,
    range_z: Range,
}

impl CubicRangeShape {
    pub fn new<R: Into<Range>>(range_x: R, range_y: R, range_z: R) -> Self {
        let range_x = range_x.into();
        let range_y = range_y.into();
        let range_z = range_z.into();
        if !Self::are_ranges_valid(&range_x, &range_y, &range_z) {
            panic!(
                "Invalid CubicRangeShape [{}, {}], [{}, {}], [{}, {}]",
                range_x.start(),
                range_x.end(),
                range_y.start(),
                range_y.end(),
                range_z.start(),
                range_z.end()
            );
        }
        Self {
            range_x,
            range_y,
            range_z,
        }
    }

    pub fn range_x(&self) -> &Range {
        &self.range_x
    }

    pub fn range_y(&self) -> &Range {
        &self.range_y
    }

    pub fn range_z(&self) -> &Range {
        &self.range_z
    }

    #[allow(dead_code)]
    fn edges_length(&self) -> [usize; 6] {
        let signed = Self::signed_edges_lengths(&self.range_x, &self.range_y, &self.range_z);
        [
            signed[0] as usize,
            signed[1] as usize,
            signed[2] as usize,
            signed[3] as usize,
            signed[4] as usize,
            signed[5] as usize,
        ]
    }

    pub fn are_ranges_valid(range_x: &Range, range_y: &Range, range_z: &Range) -> bool {
        let edges_lengths = Self::signed_edges_lengths(range_x, range_y, range_z);
        for edge_length in &edges_lengths {
            if *edge_length < 0 {
                return false;
            }
        }
        return true;
    }

    fn signed_edges_lengths(range_x: &Range, range_y: &Range, range_z: &Range) -> [isize; 6] {
        [
            -range_x.start() - range_y.start() - range_z.end(),
            range_x.end() + range_y.start() + range_z.end(),
            -range_x.end() - range_y.start() - range_z.start(),
            range_x.end() + range_y.end() + range_z.start(),
            -range_x.start() - range_y.end() - range_z.start(),
            range_x.start() + range_y.end() + range_z.end(),
        ]
    }

    pub fn vertices(&self) -> [AxialVector; 6] {
        [
            CubicVector::new(
                self.range_x.start(),
                -self.range_x.start() - self.range_z.end(),
                self.range_z.end(),
            )
            .into(),
            CubicVector::new(
                -self.range_y.start() - self.range_z.end(),
                self.range_y.start(),
                self.range_z.end(),
            )
            .into(),
            CubicVector::new(
                self.range_x.end(),
                self.range_y.start(),
                -self.range_x.end() - self.range_y.start(),
            )
            .into(),
            CubicVector::new(
                self.range_x.end(),
                -self.range_x.end() - self.range_z.start(),
                self.range_z.start(),
            )
            .into(),
            CubicVector::new(
                -self.range_y.end() - self.range_z.start(),
                self.range_y.end(),
                self.range_z.start(),
            )
            .into(),
            CubicVector::new(
                self.range_x.start(),
                self.range_y.end(),
                -self.range_x.start() - self.range_y.end(),
            )
            .into(),
        ]
    }

    pub fn perimeter(&self) -> PerimeterIter {
        PerimeterIter::new(
            self.edges_length(),
            CubicVector::new(
                self.range_x.start(),
                -self.range_x.start() - self.range_z.end(),
                self.range_z.end(),
            )
            .into(),
        )
    }

    pub fn contains_position(&self, position: AxialVector) -> bool {
        let cubic = CubicVector::from(position);
        self.range_x.contains(cubic.x())
            && self.range_y.contains(cubic.y())
            && self.range_z.contains(cubic.z())
    }

    pub fn intersects(&self, other: &Self) -> bool {
        if self.contains_position(other.vertices()[0]) {
            return true;
        }
        for pos in self.perimeter() {
            if other.contains_position(pos) {
                return true;
            }
        }
        return false;
    }

    pub fn center(&self) -> AxialVector {
        AxialVector::new(
            (self.range_x.start() + self.range_x.end()
                - (self.range_y.start()
                    + self.range_y.end()
                    + self.range_z.start()
                    + self.range_z.end())
                    / 2)
                / 3,
            (self.range_z.start() + self.range_z.end()
                - (self.range_x.start()
                    + self.range_x.end()
                    + self.range_y.start()
                    + self.range_y.end())
                    / 2)
                / 3,
        )
    }

    pub fn stretch_x_start(&mut self, amount: usize) -> bool {
        Self::stretch_axis_start(
            &mut self.range_x,
            &mut self.range_y,
            &mut self.range_z,
            amount,
        )
    }

    pub fn stretch_y_start(&mut self, amount: usize) -> bool {
        Self::stretch_axis_start(
            &mut self.range_y,
            &mut self.range_z,
            &mut self.range_x,
            amount,
        )
    }

    pub fn stretch_z_start(&mut self, amount: usize) -> bool {
        Self::stretch_axis_start(
            &mut self.range_z,
            &mut self.range_x,
            &mut self.range_y,
            amount,
        )
    }

    fn stretch_axis_start(a: &mut Range, b: &mut Range, c: &mut Range, amount: usize) -> bool {
        *a.start_mut() -= amount as isize;
        if a.start() + b.end() + c.end() < 0 {
            *b.end_mut() += amount as isize;
            *c.end_mut() += amount as isize;
        }
        true
    }

    pub fn stretch_x_end(&mut self, amount: usize) -> bool {
        Self::stretch_axis_end(
            &mut self.range_x,
            &mut self.range_y,
            &mut self.range_z,
            amount,
        )
    }

    pub fn stretch_y_end(&mut self, amount: usize) -> bool {
        Self::stretch_axis_end(
            &mut self.range_y,
            &mut self.range_z,
            &mut self.range_x,
            amount,
        )
    }

    pub fn stretch_z_end(&mut self, amount: usize) -> bool {
        Self::stretch_axis_end(
            &mut self.range_z,
            &mut self.range_x,
            &mut self.range_y,
            amount,
        )
    }

    fn stretch_axis_end(a: &mut Range, b: &mut Range, c: &mut Range, amount: usize) -> bool {
        *a.end_mut() += amount as isize;
        if -a.end() - b.start() - c.start() < 0 {
            *b.start_mut() -= amount as isize;
            *c.start_mut() -= amount as isize;
        }
        true
    }

    pub fn shrink_x_start(&mut self, amount: usize) -> bool {
        Self::shrink_axis_start(
            &mut self.range_x,
            &mut self.range_y,
            &mut self.range_z,
            amount,
        )
    }

    pub fn shrink_y_start(&mut self, amount: usize) -> bool {
        Self::shrink_axis_start(
            &mut self.range_y,
            &mut self.range_z,
            &mut self.range_x,
            amount,
        )
    }

    pub fn shrink_z_start(&mut self, amount: usize) -> bool {
        Self::shrink_axis_start(
            &mut self.range_z,
            &mut self.range_x,
            &mut self.range_y,
            amount,
        )
    }

    fn shrink_axis_start(a: &mut Range, b: &mut Range, c: &mut Range, amount: usize) -> bool {
        if a.start() + amount as isize <= a.end() {
            *a.start_mut() += amount as isize;
            if -a.start() - b.end() - c.start() < 0 {
                *b.end_mut() -= amount as isize;
            }
            if -a.start() - b.start() - c.end() < 0 {
                *c.end_mut() -= amount as isize;
            }
            true
        } else {
            false
        }
    }

    pub fn shrink_x_end(&mut self, amount: usize) -> bool {
        Self::shrink_axis_end(
            &mut self.range_x,
            &mut self.range_y,
            &mut self.range_z,
            amount,
        )
    }

    pub fn shrink_y_end(&mut self, amount: usize) -> bool {
        Self::shrink_axis_end(
            &mut self.range_y,
            &mut self.range_z,
            &mut self.range_x,
            amount,
        )
    }

    pub fn shrink_z_end(&mut self, amount: usize) -> bool {
        Self::shrink_axis_end(
            &mut self.range_z,
            &mut self.range_x,
            &mut self.range_y,
            amount,
        )
    }

    fn shrink_axis_end(a: &mut Range, b: &mut Range, c: &mut Range, amount: usize) -> bool {
        if a.start() + amount as isize <= a.end() {
            *a.end_mut() -= amount as isize;
            if a.end() + b.start() + c.end() < 0 {
                *b.start_mut() += amount as isize;
            }
            if a.end() + b.end() + c.start() < 0 {
                *c.start_mut() += amount as isize;
            }
            true
        } else {
            false
        }
    }
}

impl Default for CubicRangeShape {
    fn default() -> Self {
        CubicRangeShape::new((-1, 1), (-1, 1), (-1, 1))
    }
}

pub struct PerimeterIter {
    edges_lengths: [usize; 6],
    direction: usize,
    next: AxialVector,
    edge_index: usize,
}

impl PerimeterIter {
    pub fn new(edges_lengths: [usize; 6], initial: AxialVector) -> Self {
        let mut direction = 0;
        // Drain all but last edge so that:
        //     - the state is ready for next iteration
        //     - the ring of size 0 case is handled correctly (it returns
        //       the first value, and no more then)
        while direction < 5 && edges_lengths[direction] == 0 {
            direction += 1;
        }
        Self {
            edges_lengths,
            direction,
            next: initial,
            edge_index: 1,
        }
    }

    pub fn peek(&mut self) -> Option<&AxialVector> {
        if self.direction < NUM_DIRECTIONS {
            Some(&self.next)
        } else {
            None
        }
    }
}

impl Iterator for PerimeterIter {
    type Item = AxialVector;

    fn next(&mut self) -> Option<Self::Item> {
        let edges_lengths = self.edges_lengths;
        let direction = self.direction;
        if direction < NUM_DIRECTIONS {
            let next = self.next;
            self.next = next.neighbor(direction);
            let ei = self.edge_index;
            if ei < edges_lengths[direction] {
                self.edge_index = ei + 1;
            } else {
                self.edge_index = 1;
                self.direction = direction + 1;
                while self.direction < NUM_DIRECTIONS && edges_lengths[self.direction] == 0 {
                    self.direction += 1;
                }
            }
            Some(next)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.edges_lengths.iter().sum();
        (length, Some(length))
    }
}
