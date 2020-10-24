use crate::hex::render::{
    area::AreaRenderer,
    edge::EdgeRenderer,
    square::{SquareRenderer, SquareScale},
    tile::{HexScale, TileRenderer},
};

pub mod bumpy_builder;
pub mod cellular;
pub mod cubic_range_shape;
pub mod custom;
pub mod directions;
pub mod flat_builder;
pub mod pointer;
pub mod render;
pub mod ring;
pub mod rooms_and_mazes;
pub mod shape;
pub mod snake;

const HEX_SCALE_HORIZONTAL: f32 = 0.8;
const GROUND_HEX_SCALE_VERTICAL: f32 = 0.1;
const WALL_HEX_SCALE_VERTICAL: f32 = 1.0;

pub fn new_tile_renderer() -> TileRenderer {
    TileRenderer::new(
        HexScale {
            horizontal: HEX_SCALE_HORIZONTAL,
            vertical: GROUND_HEX_SCALE_VERTICAL,
        },
        HexScale {
            horizontal: HEX_SCALE_HORIZONTAL,
            vertical: WALL_HEX_SCALE_VERTICAL,
        },
        0,
    )
}

const SQUARE_SCALE_HORIZONTAL: f32 = 0.7;

pub fn new_square_renderer() -> SquareRenderer {
    SquareRenderer::new(
        SquareScale {
            horizontal: SQUARE_SCALE_HORIZONTAL,
        },
        SquareScale {
            horizontal: SQUARE_SCALE_HORIZONTAL,
        },
        0,
    )
}

pub fn new_edge_renderer() -> EdgeRenderer {
    EdgeRenderer::new()
}

pub fn new_area_renderer() -> AreaRenderer {
    AreaRenderer::new()
}
