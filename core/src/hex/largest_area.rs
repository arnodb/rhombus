use crate::hex::{
    coordinates::axial::AxialVector,
    storage::hash::{RectHashEntry, RectHashStorage},
};
use std::{collections::BTreeMap, ops::RangeInclusive};

#[derive(Default)]
pub struct LargestAreaIterator {
    data: RectHashStorage<CellData>,
}

#[derive(Clone, Copy)]
struct CellData {
    h: usize,
    w: usize,
}

impl LargestAreaIterator {
    pub fn start_accumulation<'a>(&'a mut self) -> Accumulator<'a> {
        self.data.clear();
        Accumulator::new(self)
    }

    pub fn next_largest_area(
        &mut self,
    ) -> (
        usize,
        Option<(RangeInclusive<isize>, RangeInclusive<isize>)>,
    ) {
        let mut largest_area = (0, None);
        for (pos, cell_data) in self.data.iter() {
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
                        .get(AxialVector::new(pos.q() - dh as isize, pos.r()))
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
                    self.data.remove(AxialVector::new(q, r));
                }
                let mut r = range_r.end() + 1;
                let mut w = 1;
                loop {
                    match self.data.entry(AxialVector::new(q, r)) {
                        RectHashEntry::Occupied(mut cell_data) => {
                            cell_data.get_mut().w = w;
                            r += 1;
                            w += 1
                        }
                        RectHashEntry::Vacant(..) => {
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
                        RectHashEntry::Occupied(mut cell_data) => {
                            cell_data.get_mut().h = h;
                            q += 1;
                            h += 1
                        }
                        RectHashEntry::Vacant(..) => {
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

pub struct Accumulator<'a> {
    lai: &'a mut LargestAreaIterator,
    previous_q: BTreeMap<isize, (isize, CellData)>,
    previous_r: BTreeMap<isize, (isize, CellData)>,
}

impl<'a> Accumulator<'a> {
    fn new(lai: &'a mut LargestAreaIterator) -> Self {
        Self {
            lai,
            previous_q: BTreeMap::new(),
            previous_r: BTreeMap::new(),
        }
    }

    pub fn push(&mut self, position: AxialVector) {
        let mut cell_data = CellData { h: 0, w: 0 };
        if let Some((prev_q, prev_cell_data)) = self.previous_r.get(&position.r()) {
            if prev_q + 1 == position.q() {
                cell_data.h = prev_cell_data.h + 1;
            } else {
                cell_data.h = 1;
            }
        } else {
            cell_data.h = 1;
        }
        if let Some((prev_r, prev_cell_data)) = self.previous_q.get(&position.q()) {
            if prev_r + 1 == position.r() {
                cell_data.w = prev_cell_data.w + 1;
            } else {
                cell_data.w = 1;
            }
        } else {
            cell_data.w = 1;
        }
        self.lai.data.insert(position, cell_data);
        self.previous_q
            .insert(position.q(), (position.r(), cell_data));
        self.previous_r
            .insert(position.r(), (position.q(), cell_data));
    }
}
