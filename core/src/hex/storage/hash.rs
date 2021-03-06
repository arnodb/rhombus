use crate::{
    hex::{
        coordinates::{axial::AxialVector, direction::HexagonalDirection},
        storage::{
            adjacent::{HexWithAdjacents, HexWithAdjacentsMut},
            rect::{
                RectEntry, RectOccupiedEntry, RectStorage, RectVacantEntry, RECT_X_LEN, RECT_Y_LEN,
            },
        },
    },
    vector::Vector2ISize,
};
use std::collections::{hash_map::Entry, HashMap};

pub struct RectHashStorage<H> {
    rects: HashMap<Vector2ISize, RectStorage<H>>,
    len: usize,
}

impl<H> RectHashStorage<H> {
    pub fn new() -> Self {
        Self {
            rects: HashMap::new(),
            len: 0,
        }
    }

    pub fn get(&self, position: AxialVector) -> Option<&H> {
        let x = position.q().div_euclid(RECT_X_LEN as isize);
        let y = position.r().div_euclid(RECT_Y_LEN as isize);
        self.rects.get(&Vector2ISize { x, y }).and_then(|rect| {
            rect.get(
                position.q().rem_euclid(RECT_X_LEN as isize) as usize,
                position.r().rem_euclid(RECT_Y_LEN as isize) as usize,
            )
        })
    }

    pub fn get_mut(&mut self, position: AxialVector) -> Option<&mut H> {
        let x = position.q().div_euclid(RECT_X_LEN as isize);
        let y = position.r().div_euclid(RECT_Y_LEN as isize);
        self.rects.get_mut(&Vector2ISize { x, y }).and_then(|rect| {
            rect.get_mut(
                position.q().rem_euclid(RECT_X_LEN as isize) as usize,
                position.r().rem_euclid(RECT_Y_LEN as isize) as usize,
            )
        })
    }

    pub fn contains_position(&self, position: AxialVector) -> bool {
        let x = position.q().div_euclid(RECT_X_LEN as isize);
        let y = position.r().div_euclid(RECT_Y_LEN as isize);
        self.rects
            .get(&Vector2ISize { x, y })
            .map_or(false, |rect| {
                rect.contains_position(
                    position.q().rem_euclid(RECT_X_LEN as isize) as usize,
                    position.r().rem_euclid(RECT_Y_LEN as isize) as usize,
                )
            })
    }

    pub fn iter(&self) -> impl Iterator<Item = (AxialVector, &H)> {
        self.rects.iter().flat_map(|(rect_origin, rect)| {
            rect.iter().map(move |(x, y, hex)| {
                (
                    AxialVector::new(
                        rect_origin.x * RECT_X_LEN as isize + x as isize,
                        rect_origin.y * RECT_Y_LEN as isize + y as isize,
                    ),
                    hex,
                )
            })
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (AxialVector, &mut H)> {
        self.rects.iter_mut().flat_map(|(rect_origin, rect)| {
            rect.iter_mut().map(move |(x, y, hex)| {
                (
                    AxialVector::new(
                        rect_origin.x * RECT_X_LEN as isize + x as isize,
                        rect_origin.y * RECT_Y_LEN as isize + y as isize,
                    ),
                    hex,
                )
            })
        })
    }

    pub fn positions<'a>(&'a self) -> impl 'a + Iterator<Item = AxialVector> {
        self.rects.iter().flat_map(|(rect_origin, rect)| {
            rect.positions().map(move |(x, y)| {
                AxialVector::new(
                    rect_origin.x * RECT_X_LEN as isize + x as isize,
                    rect_origin.y * RECT_Y_LEN as isize + y as isize,
                )
            })
        })
    }

    pub fn hexes(&self) -> impl Iterator<Item = &H> {
        self.rects.values().flat_map(|rect| rect.hexes())
    }

    pub fn hexes_mut(&mut self) -> impl Iterator<Item = &mut H> {
        self.rects.values_mut().flat_map(|rect| rect.hexes_mut())
    }

    pub fn positions_and_hexes_with_adjacents<'a>(
        &'a self,
    ) -> impl Iterator<Item = (AxialVector, HexWithAdjacents<'a, &'a H, H>)> {
        self.rects.iter().flat_map(move |(rect_origin, rect)| {
            rect.positions().map(move |(x, y)| {
                let position = AxialVector::new(
                    rect_origin.x * RECT_X_LEN as isize + x as isize,
                    rect_origin.y * RECT_Y_LEN as isize + y as isize,
                );
                (position, self.hex_with_adjacents(position).unwrap())
            })
        })
    }

    pub fn positions_and_hexes_with_adjacents_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = (AxialVector, HexWithAdjacentsMut<'a, &'a mut H, H>)> {
        let self_ptr = self as *mut Self;
        self.rects.iter().flat_map(move |(rect_origin, rect)| {
            rect.positions().map(move |(x, y)| {
                let position = AxialVector::new(
                    rect_origin.x * RECT_X_LEN as isize + x as isize,
                    rect_origin.y * RECT_Y_LEN as isize + y as isize,
                );
                (
                    position,
                    unsafe { &mut *self_ptr }
                        .hex_with_adjacents_mut(position)
                        .unwrap(),
                )
            })
        })
    }

    pub fn hex_with_adjacents(&self, position: AxialVector) -> HexWithAdjacents<Option<&H>, H> {
        let mut rects_len = 0;
        let mut rects: [(Vector2ISize, Option<&RectStorage<H>>); 4] = Default::default();
        let mut get = |pos: AxialVector| -> Option<&H> {
            let x = pos.q().div_euclid(RECT_X_LEN as isize);
            let y = pos.r().div_euclid(RECT_Y_LEN as isize);
            let rect_pos = Vector2ISize { x, y };
            let mut index = 0;
            while index < rects_len && rects[index].0 != rect_pos {
                index += 1;
            }
            if index >= rects_len {
                if index < rects.len() {
                    rects[index] = (rect_pos, self.rects.get(&rect_pos));
                    rects_len += 1;
                } else {
                    unreachable!();
                }
            }
            rects[index].1.and_then(|rect| {
                rect.get(
                    pos.q().rem_euclid(RECT_X_LEN as isize) as usize,
                    pos.r().rem_euclid(RECT_Y_LEN as isize) as usize,
                )
            })
        };
        HexWithAdjacents::new(
            get(position),
            get(position + AxialVector::direction(0)),
            get(position + AxialVector::direction(1)),
            get(position + AxialVector::direction(2)),
            get(position + AxialVector::direction(3)),
            get(position + AxialVector::direction(4)),
            get(position + AxialVector::direction(5)),
        )
    }

    pub fn hex_with_adjacents_mut(
        &mut self,
        position: AxialVector,
    ) -> HexWithAdjacentsMut<Option<&mut H>, H> {
        let mut rects_len = 0;
        let mut rects: [(Vector2ISize, Option<&mut RectStorage<H>>); 4] = Default::default();
        let mut get = |pos: AxialVector| -> Option<&mut H> {
            let x = pos.q().div_euclid(RECT_X_LEN as isize);
            let y = pos.r().div_euclid(RECT_Y_LEN as isize);
            let rect_pos = Vector2ISize { x, y };
            let mut index = 0;
            while index < rects_len && rects[index].0 != rect_pos {
                index += 1;
            }
            if index >= rects_len {
                if index < rects.len() {
                    rects[index] = (
                        rect_pos,
                        unsafe { &mut *(self as *mut Self) }
                            .rects
                            .get_mut(&rect_pos),
                    );
                    rects_len += 1;
                } else {
                    unreachable!();
                }
            }
            rects[index].1.as_mut().and_then(|rect| {
                unsafe { &mut *(*rect as *mut RectStorage<H>) }.get_mut(
                    pos.q().rem_euclid(RECT_X_LEN as isize) as usize,
                    pos.r().rem_euclid(RECT_Y_LEN as isize) as usize,
                )
            })
        };
        HexWithAdjacentsMut::new(
            get(position),
            get(position + AxialVector::direction(0)),
            get(position + AxialVector::direction(1)),
            get(position + AxialVector::direction(2)),
            get(position + AxialVector::direction(3)),
            get(position + AxialVector::direction(4)),
            get(position + AxialVector::direction(5)),
        )
    }

    pub fn insert(&mut self, position: AxialVector, hex: H) -> Option<H> {
        let x = position.q().div_euclid(RECT_X_LEN as isize);
        let y = position.r().div_euclid(RECT_Y_LEN as isize);
        let old = self
            .rects
            .entry(Vector2ISize { x, y })
            .or_insert_with(RectStorage::new)
            .insert(
                position.q().rem_euclid(RECT_X_LEN as isize) as usize,
                position.r().rem_euclid(RECT_Y_LEN as isize) as usize,
                hex,
            );
        if old.is_none() {
            self.len += 1;
        }
        old
    }

    pub fn remove(&mut self, position: AxialVector) -> Option<H> {
        let x = position.q().div_euclid(RECT_X_LEN as isize);
        let y = position.r().div_euclid(RECT_Y_LEN as isize);
        let mut hex = None;
        self.rects.entry(Vector2ISize { x, y }).and_modify(|rect| {
            hex = rect.remove(
                position.q().rem_euclid(RECT_X_LEN as isize) as usize,
                position.r().rem_euclid(RECT_Y_LEN as isize) as usize,
            );
        });
        if hex.is_some() {
            self.len -= 1;
        }
        hex
    }

    pub fn clear(&mut self) {
        for rect in &mut self.rects.values_mut() {
            rect.clear();
        }
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn entry(&mut self, position: AxialVector) -> RectHashEntry<H> {
        let x = position.q().div_euclid(RECT_X_LEN as isize);
        let y = position.r().div_euclid(RECT_Y_LEN as isize);
        let rect_x = position.q().rem_euclid(RECT_X_LEN as isize) as usize;
        let rect_y = position.r().rem_euclid(RECT_Y_LEN as isize) as usize;
        let storage_len = &mut self.len;
        match self.rects.entry(Vector2ISize { x, y }) {
            Entry::Occupied(hash_entry) => {
                let rect_entry = hash_entry.into_mut().entry(rect_x, rect_y);
                match rect_entry {
                    RectEntry::Occupied(rect_entry) => {
                        RectHashEntry::Occupied(RectHashOccupiedEntry { rect_entry })
                    }
                    RectEntry::Vacant(rect_entry) => RectHashEntry::Vacant(RectHashVacantEntry {
                        storage_len,
                        rect_entry,
                    }),
                }
            }
            Entry::Vacant(hash_entry) => RectHashEntry::Vacant(
                match hash_entry.insert(RectStorage::new()).entry(rect_x, rect_y) {
                    RectEntry::Occupied(_) => unreachable!(),
                    RectEntry::Vacant(rect_entry) => RectHashVacantEntry {
                        storage_len,
                        rect_entry,
                    },
                },
            ),
        }
    }
}

impl<H> Default for RectHashStorage<H> {
    fn default() -> Self {
        Self::new()
    }
}

pub enum RectHashEntry<'a, H> {
    Occupied(RectHashOccupiedEntry<'a, H>),
    Vacant(RectHashVacantEntry<'a, H>),
}

impl<'a, H> RectHashEntry<'a, H> {
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut H),
    {
        match self {
            RectHashEntry::Occupied(mut entry) => {
                f(entry.get_mut());
                RectHashEntry::Occupied(entry)
            }
            RectHashEntry::Vacant(entry) => RectHashEntry::Vacant(entry),
        }
    }

    pub fn or_insert(self, default: H) -> &'a mut H {
        match self {
            RectHashEntry::Occupied(entry) => entry.into_mut(),
            RectHashEntry::Vacant(entry) => entry.insert(default),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> H>(self, default: F) -> &'a mut H {
        match self {
            RectHashEntry::Occupied(entry) => entry.into_mut(),
            RectHashEntry::Vacant(entry) => entry.insert(default()),
        }
    }
}

pub struct RectHashOccupiedEntry<'a, H> {
    rect_entry: RectOccupiedEntry<'a, H>,
}

impl<'a, H> RectHashOccupiedEntry<'a, H> {
    pub fn get(&self) -> &H {
        self.rect_entry.get()
    }

    pub fn get_mut(&mut self) -> &mut H {
        self.rect_entry.get_mut()
    }

    pub fn into_mut(self) -> &'a mut H {
        self.rect_entry.into_mut()
    }
}

pub struct RectHashVacantEntry<'a, H> {
    storage_len: &'a mut usize,
    rect_entry: RectVacantEntry<'a, H>,
}

impl<'a, H> RectHashVacantEntry<'a, H> {
    pub fn insert(self, hex: H) -> &'a mut H {
        *self.storage_len += 1;
        self.rect_entry.insert(hex)
    }
}

#[test]
fn test_rect_hash_storage_should_give_access_to_hex() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectHashStorage::new();
    storage.insert(AxialVector::new(12, -42), Hex { value: 42 });
    assert_eq!(
        storage.get(AxialVector::new(12, -42)),
        Some(&Hex { value: 42 })
    );
    assert_eq!(storage.get(AxialVector::new(0, 0)), None);

    assert_eq!(storage.len(), 1);
    assert!(!storage.is_empty());
}

#[test]
fn test_rect_hash_storage_should_give_mutable_access_to_hex() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectHashStorage::new();
    storage.insert(AxialVector::new(12, -42), Hex { value: 42 });
    storage.get_mut(AxialVector::new(12, -42)).unwrap().value = 12;
    assert_eq!(storage.get_mut(AxialVector::new(0, 0)), None);
    assert_eq!(
        storage.get(AxialVector::new(12, -42)),
        Some(&Hex { value: 12 })
    );
    assert_eq!(storage.get(AxialVector::new(0, 0)), None);

    assert_eq!(storage.len(), 1);
    assert!(!storage.is_empty());
}

#[test]
fn test_rect_hash_storage_should_contain_position() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex;
    let mut storage = RectHashStorage::new();
    storage.insert(AxialVector::new(12, -42), Hex);
    assert!(storage.contains_position(AxialVector::new(12, -42)));
    assert!(!storage.contains_position(AxialVector::new(0, 0)));

    assert_eq!(storage.len(), 1);
    assert!(!storage.is_empty());
}

#[test]
fn test_rect_hash_storage_coordinates() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: isize,
    }
    let mut storage = RectHashStorage::new();
    for x in -10..10 {
        for y in -5..15 {
            storage.insert(
                AxialVector::new(x, y),
                Hex {
                    value: x * 89 + y * 97,
                },
            );
        }
    }
    for x in -10..10 {
        for y in -5..15 {
            assert_eq!(
                storage.get(AxialVector::new(x, y)),
                Some(&Hex {
                    value: x * 89 + y * 97
                })
            );
        }
    }

    assert_eq!(storage.len(), 400);
    assert!(!storage.is_empty());
}

#[test]
fn test_rect_hash_storage_should_iterate_over_positions_and_hexes() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectHashStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y, value) in [(12, -42, 93), (-5, 24, 7), (12, -42, 42), (0, 0, 1)].iter() {
        storage.insert(AxialVector::new(*x, *y), Hex { value: *value });
    }
    assert_eq!(
        storage.get(AxialVector::new(12, -42)),
        Some(&Hex { value: 42 })
    );
    assert_eq!(
        storage.get(AxialVector::new(-5, 24)),
        Some(&Hex { value: 7 })
    );
    assert_eq!(storage.get(AxialVector::new(0, 0)), Some(&Hex { value: 1 }));
    assert_eq!(
        storage
            .iter()
            .map(|(position, hex)| (position, hex.value))
            .collect::<std::collections::HashSet<_>>(),
        hashset![
            (AxialVector::new(0, 0), 1),
            (AxialVector::new(12, -42), 42),
            (AxialVector::new(-5, 24), 7)
        ]
    );

    assert_eq!(storage.len(), 3);
    assert!(!storage.is_empty());
}

#[test]
fn test_rect_hash_storage_should_iterate_over_positions_and_mutable_hexes() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectHashStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y, value) in [(12, -42, 93), (-5, 24, 7), (12, -42, 42), (0, 0, 1)].iter() {
        storage.insert(AxialVector::new(*x, *y), Hex { value: *value });
    }
    assert_eq!(
        storage.get(AxialVector::new(12, -42)),
        Some(&Hex { value: 42 })
    );
    assert_eq!(
        storage.get(AxialVector::new(-5, 24)),
        Some(&Hex { value: 7 })
    );
    assert_eq!(storage.get(AxialVector::new(0, 0)), Some(&Hex { value: 1 }));
    assert_eq!(
        storage
            .iter_mut()
            .map(|(position, hex)| {
                let value = hex.value;
                hex.value = 0;
                (position, value)
            })
            .collect::<std::collections::HashSet<_>>(),
        hashset![
            (AxialVector::new(0, 0), 1),
            (AxialVector::new(12, -42), 42),
            (AxialVector::new(-5, 24), 7)
        ]
    );
    assert_eq!(
        storage
            .iter_mut()
            .map(|(position, hex)| (position, hex.value))
            .collect::<std::collections::HashSet<_>>(),
        hashset![
            (AxialVector::new(0, 0), 0),
            (AxialVector::new(12, -42), 0),
            (AxialVector::new(-5, 24), 0)
        ]
    );

    assert_eq!(storage.len(), 3);
    assert!(!storage.is_empty());
}

#[test]
fn test_rect_hash_storage_should_iterate_over_positions() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectHashStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y, value) in [(12, -42, 93), (-5, 24, 7), (12, -42, 42), (0, 0, 1)].iter() {
        storage.insert(AxialVector::new(*x, *y), Hex { value: *value });
    }
    assert_eq!(
        storage.get(AxialVector::new(12, -42)),
        Some(&Hex { value: 42 })
    );
    assert_eq!(
        storage.get(AxialVector::new(-5, 24)),
        Some(&Hex { value: 7 })
    );
    assert_eq!(storage.get(AxialVector::new(0, 0)), Some(&Hex { value: 1 }));
    assert_eq!(
        storage
            .positions()
            .collect::<std::collections::HashSet<_>>(),
        hashset![
            AxialVector::new(0, 0),
            AxialVector::new(12, -42),
            AxialVector::new(-5, 24)
        ]
    );

    assert_eq!(storage.len(), 3);
    assert!(!storage.is_empty());
}

#[test]
fn test_rect_hash_storage_should_iterate_over_hexes() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectHashStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y, value) in [(12, -42, 93), (-5, 24, 7), (12, -42, 42), (0, 0, 1)].iter() {
        storage.insert(AxialVector::new(*x, *y), Hex { value: *value });
    }
    assert_eq!(
        storage.get(AxialVector::new(12, -42)),
        Some(&Hex { value: 42 })
    );
    assert_eq!(
        storage.get(AxialVector::new(-5, 24)),
        Some(&Hex { value: 7 })
    );
    assert_eq!(storage.get(AxialVector::new(0, 0)), Some(&Hex { value: 1 }));
    assert_eq!(
        storage
            .hexes()
            .map(|hex| hex.value)
            .collect::<std::collections::HashSet<_>>(),
        hashset![1, 42, 7]
    );

    assert_eq!(storage.len(), 3);
    assert!(!storage.is_empty());
}

#[test]
fn test_rect_hash_storage_should_iterate_over_mutable_hexes() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectHashStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y, value) in [(12, -42, 93), (-5, 24, 7), (12, -42, 42), (0, 0, 1)].iter() {
        storage.insert(AxialVector::new(*x, *y), Hex { value: *value });
    }
    assert_eq!(
        storage.get(AxialVector::new(12, -42)),
        Some(&Hex { value: 42 })
    );
    assert_eq!(
        storage.get(AxialVector::new(-5, 24)),
        Some(&Hex { value: 7 })
    );
    assert_eq!(storage.get(AxialVector::new(0, 0)), Some(&Hex { value: 1 }));
    assert_eq!(
        storage
            .hexes_mut()
            .map(|hex| {
                let value = hex.value;
                hex.value = 0;
                value
            })
            .collect::<std::collections::HashSet<_>>(),
        hashset![1, 42, 7]
    );
    assert_eq!(
        storage
            .hexes_mut()
            .map(|hex| hex.value)
            .collect::<std::collections::HashSet<_>>(),
        hashset![0, 0, 0]
    );

    assert_eq!(storage.len(), 3);
    assert!(!storage.is_empty());
}

#[test]
fn test_rect_hash_storage_should_iterate_over_mutable_hexes_with_adjacents() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectHashStorage::new();

    storage.insert(AxialVector::new(12, -42), Hex { value: 42 });
    storage.insert(AxialVector::new(13, -42), Hex { value: 61 });
    storage.insert(AxialVector::new(12, -43), Hex { value: 62 });
    storage.insert(AxialVector::new(11, -41), Hex { value: 63 });

    let mut count = 0;
    for (position, mut hex_with_adjacents) in storage.positions_and_hexes_with_adjacents_mut() {
        if position == AxialVector::new(12, -42) {
            assert_eq!(hex_with_adjacents.hex().value, 42);
            assert_eq!(hex_with_adjacents.adjacent(0).map(|h| h.value), Some(61));
            assert_eq!(hex_with_adjacents.adjacent(1), None);
            assert_eq!(hex_with_adjacents.adjacent(2).map(|h| h.value), Some(62));
            assert_eq!(hex_with_adjacents.adjacent(3), None);
            assert_eq!(hex_with_adjacents.adjacent(4).map(|h| h.value), Some(63));
            assert_eq!(hex_with_adjacents.adjacent(5), None);
            count += 1;
        }
        if position == AxialVector::new(13, -42) {
            assert_eq!(hex_with_adjacents.hex().value, 61);
            assert_eq!(hex_with_adjacents.adjacent(0), None);
            assert_eq!(hex_with_adjacents.adjacent(1), None);
            assert_eq!(hex_with_adjacents.adjacent(2), None);
            assert_eq!(hex_with_adjacents.adjacent(3).map(|h| h.value), Some(42));
            assert_eq!(hex_with_adjacents.adjacent(4), None);
            assert_eq!(hex_with_adjacents.adjacent(5), None);
            count += 1;
        }
        if position == AxialVector::new(12, -43) {
            assert_eq!(hex_with_adjacents.hex().value, 62);
            assert_eq!(hex_with_adjacents.adjacent(0), None);
            assert_eq!(hex_with_adjacents.adjacent(1), None);
            assert_eq!(hex_with_adjacents.adjacent(2), None);
            assert_eq!(hex_with_adjacents.adjacent(3), None);
            assert_eq!(hex_with_adjacents.adjacent(4), None);
            assert_eq!(hex_with_adjacents.adjacent(5).map(|h| h.value), Some(42));
            count += 1;
        }
        if position == AxialVector::new(11, -41) {
            assert_eq!(hex_with_adjacents.hex().value, 63);
            assert_eq!(hex_with_adjacents.adjacent(0), None);
            assert_eq!(hex_with_adjacents.adjacent(1).map(|h| h.value), Some(42));
            assert_eq!(hex_with_adjacents.adjacent(2), None);
            assert_eq!(hex_with_adjacents.adjacent(3), None);
            assert_eq!(hex_with_adjacents.adjacent(4), None);
            assert_eq!(hex_with_adjacents.adjacent(5), None);
            count += 1;
        }
    }
    assert_eq!(count, 4);

    assert_eq!(storage.len(), 4);
    assert!(!storage.is_empty());
}

#[test]
fn test_rect_hash_storage_should_remove_hexes() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex;
    let mut storage = RectHashStorage::new();
    storage.insert(AxialVector::new(12, -42), Hex);
    assert!(storage.get(AxialVector::new(12, -42)).is_some());
    let removed = storage.remove(AxialVector::new(12, -42));
    assert!(removed.is_some());
    assert!(storage.get(AxialVector::new(12, -42)).is_none());

    assert_eq!(storage.len(), 0);
    assert!(storage.is_empty());
}

#[test]
fn test_rect_hash_storage_should_have_entry_api() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectHashStorage::new();

    // or_insert...
    // and_modify...
    assert_eq!(storage.get(AxialVector::new(12, -42)), None);
    storage
        .entry(AxialVector::new(12, -42))
        .or_insert(Hex { value: 1 });
    assert_eq!(
        storage.get(AxialVector::new(12, -42)),
        Some(&Hex { value: 1 })
    );
    storage
        .entry(AxialVector::new(12, -42))
        .and_modify(|hex| hex.value += 1);
    assert_eq!(
        storage.get(AxialVector::new(12, -42)),
        Some(&Hex { value: 2 })
    );
    storage
        .entry(AxialVector::new(12, -42))
        .or_insert(Hex { value: 1 });
    assert_eq!(
        storage.get(AxialVector::new(12, -42)),
        Some(&Hex { value: 2 })
    );
    storage
        .entry(AxialVector::new(12, -42))
        .or_insert(Hex { value: 1 })
        .value += 1;
    assert_eq!(
        storage.get(AxialVector::new(12, -42)),
        Some(&Hex { value: 3 })
    );

    // or_insert_with...
    // and_modify...
    // and_modify...
    assert_eq!(storage.get(AxialVector::new(-5, 24)), None);
    storage
        .entry(AxialVector::new(-5, 24))
        .or_insert_with(|| Hex { value: 11 });
    assert_eq!(
        storage.get(AxialVector::new(-5, 24)),
        Some(&Hex { value: 11 })
    );
    storage
        .entry(AxialVector::new(-5, 24))
        .and_modify(|hex| hex.value += 1)
        .and_modify(|hex| hex.value += 1);
    assert_eq!(
        storage.get(AxialVector::new(-5, 24)),
        Some(&Hex { value: 13 })
    );
    storage
        .entry(AxialVector::new(-5, 24))
        .or_insert_with(|| Hex { value: 11 });
    assert_eq!(
        storage.get(AxialVector::new(-5, 24)),
        Some(&Hex { value: 13 })
    );
    storage
        .entry(AxialVector::new(-5, 24))
        .or_insert_with(|| Hex { value: 11 })
        .value += 1;
    assert_eq!(
        storage.get(AxialVector::new(-5, 24)),
        Some(&Hex { value: 14 })
    );

    // get, get_mut, into_mut
    if let RectHashEntry::Occupied(mut entry) = storage.entry(AxialVector::new(-5, 24)) {
        assert_eq!(entry.get().value, 14);
        entry.get_mut().value += 1;
        assert_eq!(entry.get_mut().value, 15);
        assert_eq!(entry.into_mut().value, 15);
    } else {
        panic!();
    }

    assert_eq!(storage.len(), 2);
    assert!(!storage.is_empty());
}
