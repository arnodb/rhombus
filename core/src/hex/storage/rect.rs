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
        if x >= RECT_X_LEN || y >= RECT_Y_LEN {
            panic!("Coordinates out of bounds");
        }
        let offset = x + y * RECT_X_LEN;
        if self.option_bits & (1 << offset as u64) != 0 {
            Some(unsafe { &*self.hexes[offset].as_ptr() })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut H> {
        if x >= RECT_X_LEN || y >= RECT_Y_LEN {
            panic!("Coordinates out of bounds");
        }
        let offset = x + y * RECT_X_LEN;
        if self.option_bits & (1 << offset as u64) != 0 {
            Some(unsafe { &mut *self.hexes[offset].as_mut_ptr() })
        } else {
            None
        }
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

    pub fn insert(&mut self, x: usize, y: usize, hex: H) -> Option<H> {
        if x >= RECT_X_LEN || y >= RECT_Y_LEN {
            panic!("Coordinates out of bounds");
        }
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
        if x >= RECT_X_LEN || y >= RECT_Y_LEN {
            panic!("Coordinates out of bounds");
        }
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
        storage
            .iter()
            .map(|(x, y, hex)| (x, y, hex.value))
            .collect::<Vec<_>>(),
        vec![(0, 0, 1), (3, 5, 42), (7, 7, 12)]
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
