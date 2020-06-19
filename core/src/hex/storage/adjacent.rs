use std::mem::MaybeUninit;

pub struct HexWithAdjacentsMut<'a, H, A> {
    hex: H,
    option_bits: u8,
    adjacents: [MaybeUninit<&'a mut A>; 6],
}

impl<'a, H, A> HexWithAdjacentsMut<'a, H, A> {
    pub fn new(
        hex: H,
        adj_0: Option<&'a mut A>,
        adj_1: Option<&'a mut A>,
        adj_2: Option<&'a mut A>,
        adj_3: Option<&'a mut A>,
        adj_4: Option<&'a mut A>,
        adj_5: Option<&'a mut A>,
    ) -> Self {
        let mut option_bits = 0;
        let mut adjacents =
            unsafe { MaybeUninit::<[MaybeUninit<&mut A>; 6]>::uninit().assume_init() };
        let mut write = |offset: usize, hex: Option<&'a mut A>| {
            if let Some(h) = hex {
                option_bits |= 1 << offset;
                unsafe {
                    std::ptr::write(adjacents[offset].as_mut_ptr(), h);
                }
            }
        };
        write(0, adj_0);
        write(1, adj_1);
        write(2, adj_2);
        write(3, adj_3);
        write(4, adj_4);
        write(5, adj_5);
        Self {
            hex,
            option_bits,
            adjacents,
        }
    }

    pub fn hex(&mut self) -> &mut H {
        &mut self.hex
    }

    pub fn adjacent(&mut self, direction: usize) -> Option<&mut A> {
        if direction > 5 {
            panic!("Direction out of bound");
        }
        if self.option_bits & 1 << direction != 0 {
            Some(unsafe { &mut *self.adjacents[direction].as_mut_ptr() })
        } else {
            None
        }
    }
}

impl<'a, H, A> HexWithAdjacentsMut<'a, Option<H>, A> {
    pub fn unwrap(self) -> HexWithAdjacentsMut<'a, H, A> {
        HexWithAdjacentsMut {
            hex: self.hex.unwrap(),
            option_bits: self.option_bits,
            adjacents: self.adjacents,
        }
    }
}

#[test]
fn test_hex_adjacents_array() {
    for i in 0..6 {
        let mut hex_0 = 10;
        let mut hex_1 = 20;
        let mut hex_2 = 30;
        let mut hex_3 = 40;
        let mut hex_4 = 50;
        let mut hex_5 = 60;
        let mut adjacents = HexWithAdjacentsMut::new(
            42,
            if i != 0 { Some(&mut hex_0) } else { None },
            if i != 1 { Some(&mut hex_1) } else { None },
            if i != 2 { Some(&mut hex_2) } else { None },
            if i != 3 { Some(&mut hex_3) } else { None },
            if i != 4 { Some(&mut hex_4) } else { None },
            if i != 5 { Some(&mut hex_5) } else { None },
        );
        for j in 0..6 {
            if j != i {
                assert_eq!(adjacents.adjacent(j), Some(&mut ((j + 1) * 10)));
                *adjacents.adjacent(j).unwrap() += 1;
            } else {
                assert_eq!(adjacents.adjacent(j), None);
            }
        }
        assert_eq!(hex_0, 10 + if i != 0 { 1 } else { 0 });
        assert_eq!(hex_1, 20 + if i != 1 { 1 } else { 0 });
        assert_eq!(hex_2, 30 + if i != 2 { 1 } else { 0 });
        assert_eq!(hex_3, 40 + if i != 3 { 1 } else { 0 });
        assert_eq!(hex_4, 50 + if i != 4 { 1 } else { 0 });
        assert_eq!(hex_5, 60 + if i != 5 { 1 } else { 0 });
    }
}
