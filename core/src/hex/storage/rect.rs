use std::mem::MaybeUninit;

pub const RECT_X_LEN: usize = 8;
pub const RECT_Y_LEN: usize = 8;

pub struct RectStorage<H> {
    option_bits: u64,
    hexes: [MaybeUninit<H>; RECT_X_LEN * RECT_Y_LEN],
}

impl<H> RectStorage<H> {
    pub fn new() -> Self {
        Self {
            option_bits: 0,
            hexes: unsafe {
                MaybeUninit::<[MaybeUninit<H>; RECT_X_LEN * RECT_Y_LEN]>::uninit().assume_init()
            },
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&H> {
        Self::check_bounds(x, y);
        let offset = x + y * RECT_X_LEN;
        if self.option_bits & (1 << offset as u64) != 0 {
            Some(unsafe { &*self.hexes[offset].as_ptr() })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut H> {
        Self::check_bounds(x, y);
        let offset = x + y * RECT_X_LEN;
        if self.option_bits & (1 << offset as u64) != 0 {
            Some(unsafe { &mut *self.hexes[offset].as_mut_ptr() })
        } else {
            None
        }
    }

    pub fn contains_position(&self, x: usize, y: usize) -> bool {
        Self::check_bounds(x, y);
        let offset = x + y * RECT_X_LEN;
        self.option_bits & (1 << offset as u64) != 0
    }

    pub fn iter(&self) -> Iter<H> {
        Iter {
            storage: self,
            next_offset: 0,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<H> {
        IterMut {
            storage: self,
            next_offset: 0,
        }
    }

    pub fn positions(&self) -> Positions<H> {
        Positions {
            storage: self,
            next_offset: 0,
        }
    }

    pub fn hexes(&self) -> Hexes<H> {
        Hexes {
            storage: self,
            next_offset: 0,
        }
    }

    pub fn hexes_mut(&mut self) -> HexesMut<H> {
        HexesMut {
            storage: self,
            next_offset: 0,
        }
    }

    pub fn insert(&mut self, x: usize, y: usize, hex: H) -> Option<H> {
        Self::check_bounds(x, y);
        let offset = x + y * RECT_X_LEN;
        if self.option_bits & 1 << offset as u64 != 0 {
            let mut old = hex;
            std::mem::swap(unsafe { &mut *self.hexes[offset].as_mut_ptr() }, &mut old);
            Some(old)
        } else {
            self.option_bits |= 1 << offset as u64;
            unsafe {
                std::ptr::write(self.hexes[offset].as_mut_ptr(), hex);
            }
            None
        }
    }

    pub fn remove(&mut self, x: usize, y: usize) -> Option<H> {
        Self::check_bounds(x, y);
        let offset = x + y * RECT_X_LEN;
        if self.option_bits & 1 << offset as u64 != 0 {
            self.option_bits &= !(1 << offset as u64);
            let hex = unsafe { std::ptr::read(self.hexes[offset].as_ptr()) };
            Some(hex)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        for offset in 0..(self.hexes.len()) {
            if self.option_bits & 1 << offset as u64 != 0 {
                unsafe { std::ptr::drop_in_place(self.hexes[offset].as_mut_ptr()) };
            }
        }
        self.option_bits = 0;
    }

    pub fn entry(&mut self, x: usize, y: usize) -> RectEntry<H> {
        Self::check_bounds(x, y);
        let offset = x + y * RECT_X_LEN;
        if self.option_bits & (1 << offset as u64) != 0 {
            RectEntry::Occupied(RectOccupiedEntry {
                hex: unsafe { &mut *self.hexes[offset].as_mut_ptr() },
            })
        } else {
            RectEntry::Vacant(RectVacantEntry {
                storage: self,
                offset,
            })
        }
    }

    fn check_bounds(x: usize, y: usize) {
        if x >= RECT_X_LEN || y >= RECT_Y_LEN {
            panic!("Coordinates out of bounds");
        }
    }
}

impl<C> Drop for RectStorage<C> {
    fn drop(&mut self) {
        for offset in 0..(self.hexes.len()) {
            if self.option_bits & 1 << offset as u64 != 0 {
                unsafe { std::ptr::drop_in_place(self.hexes[offset].as_mut_ptr()) };
            }
        }
    }
}

pub struct Iter<'a, H> {
    storage: &'a RectStorage<H>,
    next_offset: usize,
}

impl<'a, H> Iterator for Iter<'a, H> {
    type Item = (usize, usize, &'a H);

    fn next(&mut self) -> Option<Self::Item> {
        let mut offset = self.next_offset;
        while offset < RECT_X_LEN * RECT_Y_LEN {
            if self.storage.option_bits & 1 << offset as u64 != 0 {
                self.next_offset = offset + 1;
                return Some((offset % RECT_X_LEN, offset / RECT_X_LEN, unsafe {
                    &*self.storage.hexes[offset].as_ptr()
                }));
            }
            offset += 1;
        }
        self.next_offset = offset + 1;
        None
    }

    // TODO size_hint
}

pub struct IterMut<'a, H> {
    storage: &'a mut RectStorage<H>,
    next_offset: usize,
}

impl<'a, H> Iterator for IterMut<'a, H> {
    type Item = (usize, usize, &'a mut H);

    fn next(&mut self) -> Option<Self::Item> {
        let mut offset = self.next_offset;
        while offset < RECT_X_LEN * RECT_Y_LEN {
            if self.storage.option_bits & 1 << offset as u64 != 0 {
                self.next_offset = offset + 1;
                return Some((offset % RECT_X_LEN, offset / RECT_X_LEN, unsafe {
                    &mut *self.storage.hexes[offset].as_mut_ptr()
                }));
            }
            offset += 1;
        }
        self.next_offset = offset + 1;
        None
    }

    // TODO size_hint
}

pub struct Positions<'a, H> {
    storage: &'a RectStorage<H>,
    next_offset: usize,
}

impl<'a, H> Iterator for Positions<'a, H> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let mut offset = self.next_offset;
        while offset < RECT_X_LEN * RECT_Y_LEN {
            if self.storage.option_bits & 1 << offset as u64 != 0 {
                self.next_offset = offset + 1;
                return Some((offset % RECT_X_LEN, offset / RECT_X_LEN));
            }
            offset += 1;
        }
        self.next_offset = offset + 1;
        None
    }

    // TODO size_hint
}

pub struct Hexes<'a, H> {
    storage: &'a RectStorage<H>,
    next_offset: usize,
}

impl<'a, H> Iterator for Hexes<'a, H> {
    type Item = &'a H;

    fn next(&mut self) -> Option<Self::Item> {
        let mut offset = self.next_offset;
        while offset < RECT_X_LEN * RECT_Y_LEN {
            if self.storage.option_bits & 1 << offset as u64 != 0 {
                self.next_offset = offset + 1;
                return Some(unsafe { &*self.storage.hexes[offset].as_ptr() });
            }
            offset += 1;
        }
        self.next_offset = offset + 1;
        None
    }

    // TODO size_hint
}

pub struct HexesMut<'a, H> {
    storage: &'a mut RectStorage<H>,
    next_offset: usize,
}

impl<'a, H> Iterator for HexesMut<'a, H> {
    type Item = &'a mut H;

    fn next(&mut self) -> Option<Self::Item> {
        let mut offset = self.next_offset;
        while offset < RECT_X_LEN * RECT_Y_LEN {
            if self.storage.option_bits & 1 << offset as u64 != 0 {
                self.next_offset = offset + 1;
                return Some(unsafe { &mut *self.storage.hexes[offset].as_mut_ptr() });
            }
            offset += 1;
        }
        self.next_offset = offset + 1;
        None
    }

    // TODO size_hint
}

pub enum RectEntry<'a, H> {
    Occupied(RectOccupiedEntry<'a, H>),
    Vacant(RectVacantEntry<'a, H>),
}

impl<'a, H> RectEntry<'a, H> {
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut H),
    {
        match self {
            RectEntry::Occupied(mut entry) => {
                f(entry.get_mut());
                RectEntry::Occupied(entry)
            }
            RectEntry::Vacant(entry) => RectEntry::Vacant(entry),
        }
    }

    pub fn or_insert(self, default: H) -> &'a mut H {
        match self {
            RectEntry::Occupied(entry) => entry.into_mut(),
            RectEntry::Vacant(entry) => entry.insert(default),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> H>(self, default: F) -> &'a mut H {
        match self {
            RectEntry::Occupied(entry) => entry.into_mut(),
            RectEntry::Vacant(entry) => entry.insert(default()),
        }
    }
}

pub struct RectOccupiedEntry<'a, H> {
    hex: &'a mut H,
}

impl<'a, H> RectOccupiedEntry<'a, H> {
    pub fn get(&self) -> &H {
        self.hex
    }

    pub fn get_mut(&mut self) -> &mut H {
        self.hex
    }

    pub fn into_mut(self) -> &'a mut H {
        self.hex
    }
}

pub struct RectVacantEntry<'a, H> {
    storage: &'a mut RectStorage<H>,
    offset: usize,
}

impl<'a, H> RectVacantEntry<'a, H> {
    pub fn insert(self, hex: H) -> &'a mut H {
        if self.storage.option_bits & 1 << self.offset as u64 == 0 {
            self.storage.option_bits |= 1 << self.offset as u64;
            unsafe {
                std::ptr::write(self.storage.hexes[self.offset].as_mut_ptr(), hex);
            }
            unsafe { &mut *self.storage.hexes[self.offset].as_mut_ptr() }
        } else {
            unreachable!();
        }
    }
}

#[test]
fn test_rect_storage_should_give_access_to_hex() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectStorage::new();
    storage.insert(3, 5, Hex { value: 42 });
    assert_eq!(storage.get(3, 5), Some(&Hex { value: 42 }));
    assert_eq!(storage.get(0, 0), None);
}

#[test]
fn test_rect_storage_should_give_mutable_access_to_hex() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectStorage::new();
    storage.insert(3, 5, Hex { value: 42 });
    storage.get_mut(3, 5).unwrap().value = 12;
    assert_eq!(storage.get_mut(0, 0), None);
    assert_eq!(storage.get(3, 5), Some(&Hex { value: 12 }));
    assert_eq!(storage.get(0, 0), None);
}

#[test]
fn test_rect_storage_should_contain_position() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex;
    let mut storage = RectStorage::new();
    storage.insert(3, 5, Hex);
    assert!(storage.contains_position(3, 5));
    assert!(!storage.contains_position(0, 0));
}

#[test]
fn test_rect_storage_coordinates() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectStorage::new();
    for x in 0..RECT_X_LEN {
        for y in 0..RECT_Y_LEN {
            storage.insert(
                x,
                y,
                Hex {
                    value: x * 89 + y * 97,
                },
            );
        }
    }
    for x in 0..RECT_X_LEN {
        for y in 0..RECT_Y_LEN {
            assert_eq!(
                storage.get(x, y),
                Some(&Hex {
                    value: x * 89 + y * 97
                })
            );
        }
    }
}

#[test]
fn test_rect_storage_should_iterate_over_positions_and_hexes() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y, value) in [(3, 5, 93), (7, 7, 12), (3, 5, 42), (0, 0, 1)].iter() {
        storage.insert(*x, *y, Hex { value: *value });
    }
    assert_eq!(storage.get(3, 5), Some(&Hex { value: 42 }));
    assert_eq!(storage.get(7, 7), Some(&Hex { value: 12 }));
    assert_eq!(storage.get(0, 0), Some(&Hex { value: 1 }));
    assert_eq!(
        storage
            .iter()
            .map(|(x, y, hex)| (x, y, hex.value))
            .collect::<Vec<_>>(),
        vec![(0, 0, 1), (3, 5, 42), (7, 7, 12)]
    );
}

#[test]
fn test_rect_storage_should_iterate_over_positions_and_mutable_hexes() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y, value) in [(3, 5, 93), (7, 7, 12), (3, 5, 42), (0, 0, 1)].iter() {
        storage.insert(*x, *y, Hex { value: *value });
    }
    assert_eq!(storage.get(3, 5), Some(&Hex { value: 42 }));
    assert_eq!(storage.get(7, 7), Some(&Hex { value: 12 }));
    assert_eq!(storage.get(0, 0), Some(&Hex { value: 1 }));
    assert_eq!(
        storage
            .iter_mut()
            .map(|(x, y, hex)| {
                let value = hex.value;
                hex.value = 0;
                (x, y, value)
            })
            .collect::<Vec<_>>(),
        vec![(0, 0, 1), (3, 5, 42), (7, 7, 12)]
    );
    assert_eq!(
        storage
            .iter_mut()
            .map(|(x, y, hex)| (x, y, hex.value))
            .collect::<Vec<_>>(),
        vec![(0, 0, 0), (3, 5, 0), (7, 7, 0)]
    );
}

#[test]
fn test_rect_storage_should_iterate_over_positions() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y, value) in [(3, 5, 93), (7, 7, 12), (3, 5, 42), (0, 0, 1)].iter() {
        storage.insert(*x, *y, Hex { value: *value });
    }
    assert_eq!(storage.get(3, 5), Some(&Hex { value: 42 }));
    assert_eq!(storage.get(7, 7), Some(&Hex { value: 12 }));
    assert_eq!(storage.get(0, 0), Some(&Hex { value: 1 }));
    assert_eq!(
        storage.positions().collect::<Vec<_>>(),
        vec![(0, 0), (3, 5), (7, 7)]
    );
}

#[test]
fn test_rect_storage_should_iterate_over_hexes() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y, value) in [(3, 5, 93), (7, 7, 12), (3, 5, 42), (0, 0, 1)].iter() {
        storage.insert(*x, *y, Hex { value: *value });
    }
    assert_eq!(storage.get(3, 5), Some(&Hex { value: 42 }));
    assert_eq!(storage.get(7, 7), Some(&Hex { value: 12 }));
    assert_eq!(storage.get(0, 0), Some(&Hex { value: 1 }));
    assert_eq!(
        storage.hexes().map(|hex| hex.value).collect::<Vec<_>>(),
        vec![1, 42, 12]
    );
}

#[test]
fn test_rect_storage_should_iterate_over_mutable_hexes() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y, value) in [(3, 5, 93), (7, 7, 12), (3, 5, 42), (0, 0, 1)].iter() {
        storage.insert(*x, *y, Hex { value: *value });
    }
    assert_eq!(storage.get(3, 5), Some(&Hex { value: 42 }));
    assert_eq!(storage.get(7, 7), Some(&Hex { value: 12 }));
    assert_eq!(storage.get(0, 0), Some(&Hex { value: 1 }));
    assert_eq!(
        storage
            .hexes_mut()
            .map(|hex| {
                let value = hex.value;
                hex.value = 0;
                value
            })
            .collect::<Vec<_>>(),
        vec![1, 42, 12]
    );
    assert_eq!(
        storage.hexes_mut().map(|hex| hex.value).collect::<Vec<_>>(),
        vec![0, 0, 0]
    );
}

#[test]
fn test_rect_storage_should_remove_hexes() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex;
    let mut storage = RectStorage::new();
    storage.insert(3, 5, Hex);
    assert!(storage.get(3, 5).is_some());
    let removed = storage.remove(3, 5);
    assert!(removed.is_some());
    assert!(storage.get(3, 5).is_none());
}

#[test]
fn test_rect_storage_drop_should_drop_content() {
    use std::cell::RefCell;
    struct Hex<'a> {
        drop_callback: &'a dyn Fn(),
    }
    impl<'a> Drop for Hex<'a> {
        fn drop(&mut self) {
            (*self.drop_callback)();
        }
    }
    let counter = RefCell::new(0);
    let count = || {
        *counter.borrow_mut() += 1;
    };
    let mut storage = RectStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y) in [(3, 5), (7, 7), (3, 5)].iter() {
        storage.insert(
            *x,
            *y,
            Hex {
                drop_callback: &count,
            },
        );
    }
    assert_eq!(*counter.borrow(), 1);
    drop(storage);
    assert_eq!(*counter.borrow(), 3);
}

#[test]
fn test_rect_storage_clear_should_drop_content() {
    use std::cell::RefCell;
    struct Hex<'a> {
        drop_callback: &'a dyn Fn(),
    }
    impl<'a> Drop for Hex<'a> {
        fn drop(&mut self) {
            (*self.drop_callback)();
        }
    }
    let counter = RefCell::new(0);
    let count = || {
        *counter.borrow_mut() += 1;
    };
    let mut storage = RectStorage::new();
    // Write and sometimes overwrite hexes
    for (x, y) in [(3, 5), (7, 7), (3, 5)].iter() {
        storage.insert(
            *x,
            *y,
            Hex {
                drop_callback: &count,
            },
        );
    }
    assert_eq!(*counter.borrow(), 1);
    storage.clear();
    assert_eq!(*counter.borrow(), 3);
}

#[test]
fn test_rect_storage_should_have_entry_api() {
    #[derive(PartialEq, Eq, Debug)]
    struct Hex {
        value: usize,
    }
    let mut storage = RectStorage::new();

    // or_insert...
    // and_modify...
    assert_eq!(storage.get(3, 5), None);
    storage.entry(3, 5).or_insert(Hex { value: 1 });
    assert_eq!(storage.get(3, 5), Some(&Hex { value: 1 }));
    storage.entry(3, 5).and_modify(|hex| hex.value += 1);
    assert_eq!(storage.get(3, 5), Some(&Hex { value: 2 }));
    storage.entry(3, 5).or_insert(Hex { value: 1 });
    assert_eq!(storage.get(3, 5), Some(&Hex { value: 2 }));
    storage.entry(3, 5).or_insert(Hex { value: 1 }).value += 1;
    assert_eq!(storage.get(3, 5), Some(&Hex { value: 3 }));

    // or_insert_with...
    // and_modify...
    // and_modify...
    assert_eq!(storage.get(7, 2), None);
    storage.entry(7, 2).or_insert_with(|| Hex { value: 11 });
    assert_eq!(storage.get(7, 2), Some(&Hex { value: 11 }));
    storage
        .entry(7, 2)
        .and_modify(|hex| hex.value += 1)
        .and_modify(|hex| hex.value += 1);
    assert_eq!(storage.get(7, 2), Some(&Hex { value: 13 }));
    storage.entry(7, 2).or_insert_with(|| Hex { value: 11 });
    assert_eq!(storage.get(7, 2), Some(&Hex { value: 13 }));
    storage
        .entry(7, 2)
        .or_insert_with(|| Hex { value: 11 })
        .value += 1;
    assert_eq!(storage.get(7, 2), Some(&Hex { value: 14 }));

    // get, get_mut, into_mut
    if let RectEntry::Occupied(mut entry) = storage.entry(7, 2) {
        assert_eq!(entry.get().value, 14);
        entry.get_mut().value += 1;
        assert_eq!(entry.get_mut().value, 15);
        assert_eq!(entry.into_mut().value, 15);
    } else {
        panic!();
    }
}
