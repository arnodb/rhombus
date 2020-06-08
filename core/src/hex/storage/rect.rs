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
fn test_rect_storage_should_drop_content() {
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
    drop(storage);
    assert_eq!(*counter.borrow(), 3);
}
