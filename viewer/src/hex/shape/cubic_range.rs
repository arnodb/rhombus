use rhombus_core::hex::coordinates::{axial::AxialVector, cubic::CubicVector};
use std::ops::RangeInclusive;

pub struct CubicRangeShape {
    range_x: RangeInclusive<isize>,
    range_y: RangeInclusive<isize>,
    range_z: RangeInclusive<isize>,
}

impl CubicRangeShape {
    pub fn new(
        range_x: RangeInclusive<isize>,
        range_y: RangeInclusive<isize>,
        range_z: RangeInclusive<isize>,
    ) -> Self {
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

    pub fn range_x(&self) -> &RangeInclusive<isize> {
        &self.range_x
    }

    pub fn range_y(&self) -> &RangeInclusive<isize> {
        &self.range_y
    }

    pub fn range_z(&self) -> &RangeInclusive<isize> {
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

    pub fn are_ranges_valid(
        range_x: &RangeInclusive<isize>,
        range_y: &RangeInclusive<isize>,
        range_z: &RangeInclusive<isize>,
    ) -> bool {
        let edges_lengths = Self::signed_edges_lengths(range_x, range_y, range_z);
        for edge_length in &edges_lengths {
            if *edge_length < 0 {
                return false;
            }
        }
        return true;
    }

    fn signed_edges_lengths(
        range_x: &RangeInclusive<isize>,
        range_y: &RangeInclusive<isize>,
        range_z: &RangeInclusive<isize>,
    ) -> [isize; 6] {
        [
            *range_x.start() + *range_y.end() + *range_z.end(),
            -*range_x.start() - *range_y.start() - *range_z.end(),
            *range_x.end() + *range_y.start() + *range_z.end(),
            -*range_x.end() - *range_y.start() - *range_z.start(),
            *range_x.end() + *range_y.end() + *range_z.start(),
            -*range_x.start() - *range_y.end() - *range_z.start(),
        ]
    }

    pub fn vertices(&self) -> [AxialVector; 6] {
        [
            CubicVector::new(
                *self.range_x.start(),
                *self.range_y.end(),
                -*self.range_x.start() - *self.range_y.end(),
            )
            .into(),
            CubicVector::new(
                *self.range_x.start(),
                -*self.range_x.start() - *self.range_z.end(),
                *self.range_z.end(),
            )
            .into(),
            CubicVector::new(
                -*self.range_y.start() - *self.range_z.end(),
                *self.range_y.start(),
                *self.range_z.end(),
            )
            .into(),
            CubicVector::new(
                *self.range_x.end(),
                *self.range_y.start(),
                -*self.range_x.end() - *self.range_y.start(),
            )
            .into(),
            CubicVector::new(
                *self.range_x.end(),
                -*self.range_x.end() - *self.range_z.start(),
                *self.range_z.start(),
            )
            .into(),
            CubicVector::new(
                -*self.range_y.end() - *self.range_z.start(),
                *self.range_y.end(),
                *self.range_z.start(),
            )
            .into(),
        ]
    }

    pub fn contains(&self, position: AxialVector) -> bool {
        let cubic = CubicVector::from(position);
        self.range_x.contains(&cubic.x())
            && self.range_y.contains(&cubic.y())
            && self.range_z.contains(&cubic.z())
    }

    #[allow(dead_code)]
    fn intersects(&self, other: &Self) -> bool {
        for v in self.vertices().iter() {
            if other.contains(*v) {
                return true;
            }
        }
        for v in other.vertices().iter() {
            if self.contains(*v) {
                return true;
            }
        }
        return false;
    }

    pub fn center(&self) -> AxialVector {
        AxialVector::new(
            (*self.range_x.start() + *self.range_x.end()
                - (*self.range_y.start()
                    + *self.range_y.end()
                    + *self.range_z.start()
                    + *self.range_z.end())
                    / 2)
                / 3,
            (*self.range_z.start() + *self.range_z.end()
                - (*self.range_x.start()
                    + *self.range_x.end()
                    + *self.range_y.start()
                    + *self.range_y.end())
                    / 2)
                / 3,
        )
    }

    pub fn stretch_x_start(&mut self) -> bool {
        Self::stretch_axis_start(&mut self.range_x, &mut self.range_y, &mut self.range_z)
    }

    pub fn stretch_y_start(&mut self) -> bool {
        Self::stretch_axis_start(&mut self.range_y, &mut self.range_z, &mut self.range_x)
    }

    pub fn stretch_z_start(&mut self) -> bool {
        Self::stretch_axis_start(&mut self.range_z, &mut self.range_x, &mut self.range_y)
    }

    fn stretch_axis_start(
        a: &mut RangeInclusive<isize>,
        b: &mut RangeInclusive<isize>,
        c: &mut RangeInclusive<isize>,
    ) -> bool {
        *a = a.start() - 1..=*a.end();
        if a.start() + b.end() + c.end() < 0 {
            *b = *b.start()..=b.end() + 1;
            *c = *c.start()..=c.end() + 1;
        }
        true
    }

    pub fn stretch_x_end(&mut self) -> bool {
        Self::stretch_axis_end(&mut self.range_x, &mut self.range_y, &mut self.range_z)
    }

    pub fn stretch_y_end(&mut self) -> bool {
        Self::stretch_axis_end(&mut self.range_y, &mut self.range_z, &mut self.range_x)
    }

    pub fn stretch_z_end(&mut self) -> bool {
        Self::stretch_axis_end(&mut self.range_z, &mut self.range_x, &mut self.range_y)
    }

    fn stretch_axis_end(
        a: &mut RangeInclusive<isize>,
        b: &mut RangeInclusive<isize>,
        c: &mut RangeInclusive<isize>,
    ) -> bool {
        *a = *a.start()..=a.end() + 1;
        if -a.end() - b.start() - c.start() < 0 {
            *b = b.start() - 1..=*b.end();
            *c = c.start() - 1..=*c.end();
        }
        true
    }

    pub fn shrink_x_start(&mut self) -> bool {
        Self::shrink_axis_start(&mut self.range_x, &mut self.range_y, &mut self.range_z)
    }

    pub fn shrink_y_start(&mut self) -> bool {
        Self::shrink_axis_start(&mut self.range_y, &mut self.range_z, &mut self.range_x)
    }

    pub fn shrink_z_start(&mut self) -> bool {
        Self::shrink_axis_start(&mut self.range_z, &mut self.range_x, &mut self.range_y)
    }

    fn shrink_axis_start(
        a: &mut RangeInclusive<isize>,
        b: &mut RangeInclusive<isize>,
        c: &mut RangeInclusive<isize>,
    ) -> bool {
        if a.start() < a.end() {
            *a = a.start() + 1..=*a.end();
            if -a.start() - b.end() - c.start() < 0 {
                *b = *b.start()..=b.end() - 1;
            }
            if -a.start() - b.start() - c.end() < 0 {
                *c = *c.start()..=c.end() - 1;
            }
            true
        } else {
            false
        }
    }

    pub fn shrink_x_end(&mut self) -> bool {
        Self::shrink_axis_end(&mut self.range_x, &mut self.range_y, &mut self.range_z)
    }

    pub fn shrink_y_end(&mut self) -> bool {
        Self::shrink_axis_end(&mut self.range_y, &mut self.range_z, &mut self.range_x)
    }

    pub fn shrink_z_end(&mut self) -> bool {
        Self::shrink_axis_end(&mut self.range_z, &mut self.range_x, &mut self.range_y)
    }

    fn shrink_axis_end(
        a: &mut RangeInclusive<isize>,
        b: &mut RangeInclusive<isize>,
        c: &mut RangeInclusive<isize>,
    ) -> bool {
        if a.start() < a.end() {
            *a = *a.start()..=*a.end() - 1;
            if a.end() + b.start() + c.end() < 0 {
                *b = b.start() + 1..=*b.end();
            }
            if a.end() + b.end() + c.start() < 0 {
                *c = c.start() + 1..=*c.end();
            }
            true
        } else {
            false
        }
    }
}

impl Default for CubicRangeShape {
    fn default() -> Self {
        CubicRangeShape::new(-1..=1, -1..=1, -1..=1)
    }
}
