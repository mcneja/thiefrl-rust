use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::{Background::{Blended}, Color, Image},
    input::Key,
    lifecycle::{run, Asset, Settings, State, Window},
    Future, Result,
};

use multiarray::*;
use vector2d::Vector2D;

#[allow(dead_code)]
mod color_preset {
    use quicksilver::graphics::Color;

    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const DARK_BLUE: Color = Color { r: 0.0, g: 0.0, b: 0.6588, a: 1.0 };
    pub const DARK_GREEN: Color = Color { r: 0.0, g: 0.6588, b: 0.0, a: 1.0 };
    pub const DARK_CYAN: Color = Color { r: 0.0, g: 0.6588, b: 0.6588, a: 1.0 };
    pub const DARK_RED: Color = Color { r: 0.6588, g: 0.0, b: 0.0, a: 1.0 };
    pub const DARK_MAGENTA: Color = Color { r: 0.6588, g: 0.0, b: 0.6588, a: 1.0 };
    pub const DARK_BROWN: Color = Color { r: 0.6588, g: 0.3294, b: 0.0, a: 1.0 };
    pub const LIGHT_GRAY: Color = Color { r: 0.6588, g: 0.6588, b: 0.6588, a: 1.0 };
    pub const DARK_GRAY: Color = Color { r: 0.3294, g: 0.3294, b: 0.3294, a: 1.0 };
    pub const LIGHT_BLUE: Color = Color { r: 0.3294, g: 0.3294, b: 0.9961, a: 1.0 };
    pub const LIGHT_GREEN: Color = Color { r: 0.3294, g: 0.9961, b: 0.3294, a: 1.0 };
    pub const LIGHT_CYAN: Color = Color { r: 0.3294, g: 0.9961, b: 0.9961, a: 1.0 };
    pub const LIGHT_RED: Color = Color { r: 0.9961, g: 0.3294, b: 0.3294, a: 1.0 };
    pub const LIGHT_MAGENTA: Color = Color { r: 0.9961, g: 0.3294, b: 0.9961, a: 1.0 };
    pub const LIGHT_YELLOW: Color = Color { r: 0.9961, g: 0.9961, b: 0.3264, a: 1.0 };
    pub const WHITE: Color = Color { r: 0.9961, g: 0.9961, b: 0.9961, a: 1.0 };
}

#[derive(Clone, Debug, PartialEq)]
struct Tile {
    glyph: usize,
    color: Color,
}

#[derive(Clone, Debug, PartialEq)]
struct Entity {
    pos: Vector2D<i32>,
    tile: Tile,
}

struct Game {
    map: Array2D<Tile>,
    entities: Vec<Entity>,
    player_id: usize,
    tileset: Asset<Vec<Image>>,
    tile_size_px: Vector,
}

fn main() {
    let settings = Settings {
        scale: quicksilver::graphics::ImageScaleStrategy::Pixelate,
        ..Default::default()
    };
    run::<Game>("ThiefRL 3", Vector::new(880, 760), settings);
}

impl State for Game {
    /// Load the assets and initialise the game
    fn new() -> Result<Self> {
        let tiles_file = "tiles.png";

        let map = generate_map(20, 15);
        let mut entities = generate_entities();

        let player_id = entities.len();
        entities.push(Entity {
            pos: Vector2D::new(5, 3),
            tile: Tile{ glyph: 32, color: color_preset::LIGHT_CYAN },
        });

        let tile_size_px = Vector::new(16, 16);

        let tileset = Asset::new(Image::load(tiles_file).and_then(move |tiles| {
            let mut tileset = Vec::new();
            for y in 0..16 {
                for x in 0..16 {
                    let pos_px = tile_size_px.times(Vector::new(x, y));
                    let rect = Rectangle::new(pos_px, tile_size_px);
                    let tile = tiles.subimage(rect);
                    tileset.push(tile);
                }
            }
            Ok(tileset)
        }));

        Ok(Self {
            map,
            entities,
            player_id,
            tileset,
            tile_size_px,
       })
    }

    /// Process keyboard and mouse, update the game state
    fn update(&mut self, window: &mut Window) -> Result<()> {
        use quicksilver::input::ButtonState::*;

        let player = &mut self.entities[self.player_id];
        let keys = window.keyboard();
        if keys[Key::Left] == Pressed || keys[Key::Numpad4] == Pressed {
            player.pos.x -= 1;
        }
        if keys[Key::Right] == Pressed || keys[Key::Numpad6] == Pressed {
            player.pos.x += 1;
        }
        if keys[Key::Up] == Pressed || keys[Key::Numpad8] == Pressed {
            player.pos.y -= 1;
        }
        if keys[Key::Down] == Pressed || keys[Key::Numpad2] == Pressed {
            player.pos.y += 1;
        }
        if keys[Key::Numpad7] == Pressed {
            player.pos.x -= 1;
            player.pos.y -= 1;
        }
        if keys[Key::Numpad9] == Pressed {
            player.pos.x += 1;
            player.pos.y -= 1;
        }
        if keys[Key::Numpad1] == Pressed {
            player.pos.x -= 1;
            player.pos.y += 1;
        }
        if keys[Key::Numpad3] == Pressed {
            player.pos.x += 1;
            player.pos.y += 1;
        }
        if keys[Key::Escape] == Pressed {
            window.close();
        }
        Ok(())
    }

    /// Draw stuff on the screen
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(color_preset::BLACK)?;

        let tile_size_px = self.tile_size_px;
        let offset_px = Vector::new(50, 120);

        let map = &self.map;
        let entities = &self.entities;
        self.tileset.execute(|tileset| {
            for x in 0..map.extents()[0] {
                for y in 0..map.extents()[1] {
                    let pos = Vector::new(x as f32, y as f32);
                    let tile = &map[[x, y]];
                    let image = &tileset[tile.glyph];
                    let pos_px = offset_px + tile_size_px.times(pos);
                    window.draw(
                        &Rectangle::new(pos_px, image.area().size()),
                        Blended(&image, tile.color),
                    )
                }
            }
            for entity in entities.iter() {
                let image = &tileset[entity.tile.glyph];
                let pos = Vector::new(entity.pos.x, entity.pos.y);
                let pos_px = offset_px + pos.times(tile_size_px);
                window.draw(
                    &Rectangle::new(pos_px, image.area().size()),
                    Blended(&image, entity.tile.color),
                );
            }
            Ok(())
        })?;

        Ok(())
    }
}

fn generate_map(width: usize, height: usize) -> Array2D<Tile> {
    let default_tile = Tile {
        glyph: 16,
        color: color_preset::LIGHT_GRAY,
    };
    let mut map = Array2D::new([width, height], default_tile);
    for x in 0..width {
        for y in 0..height {
            let mut glyph = 112;
            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                glyph = 15;
            }
            map[[x, y]].glyph = glyph;
        }
    }
    map
}

fn generate_entities() -> Vec<Entity> {
    vec![
        guard(9, 6),
        guard(2, 4),
        coin(7, 5),
        coin(4, 8),
    ]
}

fn guard(x: i32, y: i32) -> Entity {
    Entity {
        pos: Vector2D::new(x, y),
        tile: Tile { glyph: 36, color: color_preset::LIGHT_MAGENTA },
    }
}

fn coin(x: i32, y: i32) -> Entity {
    Entity {
        pos: Vector2D::new(x, y),
        tile: Tile { glyph: 110, color: color_preset::LIGHT_YELLOW },
    }
}
