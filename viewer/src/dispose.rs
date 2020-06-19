use amethyst::prelude::*;
use rhombus_core::hex::storage::hash::RectHashStorage;

pub trait Dispose {
    fn dispose(&mut self, data: &mut StateData<'_, GameData<'_, '_>>);
}

impl<Hex: Dispose> Dispose for RectHashStorage<Hex> {
    fn dispose(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        for hex in self.hexes_mut() {
            hex.dispose(data);
        }
        self.clear();
    }
}

impl Dispose for () {
    fn dispose(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) {}
}

impl<T1: Dispose, T2: Dispose> Dispose for (T1, T2) {
    fn dispose(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        self.0.dispose(data);
        self.1.dispose(data);
    }
}
