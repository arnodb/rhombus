use crate::hex::coordinates::axial::AxialVector;
use std::{
    collections::{btree_map::Entry, BTreeMap},
    ops::RangeInclusive,
};

#[derive(Default)]
pub struct LargestAreaIterator {
    data: BTreeMap<AxialVector, CellData>,
}

#[derive(Clone, Copy)]
struct CellData {
    h: usize,
    w: usize,
}

impl LargestAreaIterator {
    pub fn initialize<I>(&mut self, positions: I)
    where
        I: Iterator<Item = AxialVector>,
    {
        self.data.clear();
        let mut previous_q = BTreeMap::<isize, (isize, CellData)>::new();
        let mut previous_r = BTreeMap::<isize, (isize, CellData)>::new();
        for pos in positions {
            let mut cell_data = CellData { h: 0, w: 0 };
            if let Some((prev_q, prev_cell_data)) = previous_r.get(&pos.r()) {
                if prev_q + 1 == pos.q() {
                    cell_data.h = prev_cell_data.h + 1;
                } else {
                    cell_data.h = 1;
                }
            } else {
                cell_data.h = 1;
            }
            if let Some((prev_r, prev_cell_data)) = previous_q.get(&pos.q()) {
                if prev_r + 1 == pos.r() {
                    cell_data.w = prev_cell_data.w + 1;
                } else {
                    cell_data.w = 1;
                }
            } else {
                cell_data.w = 1;
            }
            self.data.insert(pos, cell_data);
            previous_q.insert(pos.q(), (pos.r(), cell_data));
            previous_r.insert(pos.r(), (pos.q(), cell_data));
        }
    }

    pub fn next_largest_area(
        &mut self,
    ) -> (
        usize,
        Option<(RangeInclusive<isize>, RangeInclusive<isize>)>,
    ) {
        let mut largest_area = (0, None);
        for (pos, cell_data) in &self.data {
            let mut min_w = cell_data.w;
            if min_w > largest_area.0 {
                largest_area = (
                    min_w,
                    Some((pos.q()..=pos.q(), pos.r() - min_w as isize + 1..=pos.r())),
                );
            }
            for dh in 1..cell_data.h {
                min_w = min_w.min(
                    self.data
                        .get(&AxialVector::new(pos.q() - dh as isize, pos.r()))
                        .unwrap()
                        .w,
                );
                let area = (dh + 1) * min_w;
                if area > largest_area.0 {
                    largest_area = (
                        area,
                        Some((
                            pos.q() - dh as isize..=pos.q(),
                            pos.r() - min_w as isize + 1..=pos.r(),
                        )),
                    );
                }
            }
        }
        if let Some((range_q, range_r)) = &largest_area.1 {
            for q in range_q.clone() {
                for r in range_r.clone() {
                    self.data.remove(&AxialVector::new(q, r));
                }
                let mut r = range_r.end() + 1;
                let mut w = 1;
                loop {
                    match self.data.entry(AxialVector::new(q, r)) {
                        Entry::Occupied(mut cell_data) => {
                            cell_data.get_mut().w = w;
                            r += 1;
                            w += 1
                        }
                        Entry::Vacant(..) => {
                            break;
                        }
                    }
                }
            }
            for r in range_r.clone() {
                let mut q = range_q.end() + 1;
                let mut h = 1;
                loop {
                    match self.data.entry(AxialVector::new(q, r)) {
                        Entry::Occupied(mut cell_data) => {
                            cell_data.get_mut().h = h;
                            q += 1;
                            h += 1
                        }
                        Entry::Vacant(..) => {
                            break;
                        }
                    }
                }
            }
        }
        debug_assert!(largest_area.1.is_some() || self.data.is_empty());
        largest_area
    }
}
