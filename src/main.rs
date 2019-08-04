use quicksilver::{
    geom::{Rectangle, Shape, Vector},
    graphics::{Background::{Blended, Col, Img}, Color, Font, FontStyle, Image},
    input::Key,
    lifecycle::{run, Asset, Settings, State, Window},
    Future, Result,
};

use std::collections::HashMap;
use multiarray::*;

#[derive(Clone, Debug, PartialEq)]
struct Tile {
    glyph: char,
    color: Color,
}

#[derive(Clone, Debug, PartialEq)]
struct Entity {
    pos: Vector,
    tile: Tile,
    hp: i32,
    max_hp: i32,
}

fn main() {
    let settings = Settings {
        // scale: quicksilver::graphics::ImageScaleStrategy::Pixelate,
        ..Default::default()
    };
    run::<Game>("ThiefRL 3", Vector::new(800, 600), settings);
}

struct Game {
    title: Asset<Image>,
    mononoki_font_info: Asset<Image>,
    square_font_info: Asset<Image>,
    map: Array2D<Tile>,
    entities: Vec<Entity>,
    player_id: usize,
    tileset: Asset<HashMap<char, Image>>,
    tile_size_px: Vector,
}

impl State for Game {
    /// Load the assets and initialise the game
    fn new() -> Result<Self> {
        let font_mononoki = "mononoki-Regular.ttf";
        let font_square = "square.ttf";

        let title = Asset::new(Font::load(font_mononoki).and_then(|font| {
            font.render("ThiefRL 3", &FontStyle::new(72.0, Color::BLACK))
        }));

        let mononoki_font_info = Asset::new(Font::load(font_mononoki).and_then(|font| {
            font.render(
                "Mononoki font by Matthias Tellen, terms: SIL Open Font License 1.1",
                &FontStyle::new(20.0, Color::BLACK),
            )
        }));

        let square_font_info = Asset::new(Font::load(font_mononoki).and_then(move |font| {
            font.render(
                "Square font by Wouter Van Oortmerssen, terms: CC BY 3.0",
                &FontStyle::new(20.0, Color::BLACK),
            )
        }));

        let map = generate_map(20, 15);
        let mut entities = generate_entities();

        let player_id = entities.len();
        entities.push(Entity {
            pos: Vector::new(5, 3),
            tile: Tile{
                glyph: '@',
                color: Color::BLUE,
            },
            hp: 3,
            max_hp: 5,
        });

        let game_glyphs = "#@g.%";
        let tile_size_px = Vector::new(24, 24);

        let tileset = Asset::new(Font::load(font_square).and_then(move |text| {
            let tiles = text
                .render(game_glyphs, &FontStyle::new(tile_size_px.y, Color::WHITE))
                .expect("Could not render the font tileset.");
            let mut tileset = HashMap::new();
            for (index, glyph) in game_glyphs.chars().enumerate() {
                let pos = (index as i32 * tile_size_px.x as i32, 0);
                let tile = tiles.subimage(Rectangle::new(pos, tile_size_px));
                tileset.insert(glyph, tile);
            }
            Ok(tileset)
        }));

        Ok(Self {
            title,
            mononoki_font_info,
            square_font_info,
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
        if window.keyboard()[Key::Left] == Pressed {
            player.pos.x -= 1.0;
        }
        if window.keyboard()[Key::Right] == Pressed {
            player.pos.x += 1.0;
        }
        if window.keyboard()[Key::Up] == Pressed {
            player.pos.y -= 1.0;
        }
        if window.keyboard()[Key::Down] == Pressed {
            player.pos.y += 1.0;
        }
        if window.keyboard()[Key::Escape].is_down() {
            window.close();
        }
        Ok(())
    }

    /// Draw stuff on the screen
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;

        let window_size_x = window.screen_size().x as i32;
        let window_size_y = window.screen_size().y as i32;

        self.title.execute(|image| {
            window.draw(
                &image
                    .area()
                    .with_center((window_size_x / 2, 40)),
                Img(&image),
            );
            Ok(())
        })?;

        self.mononoki_font_info.execute(|image| {
            window.draw(
                &image
                    .area()
                    .translate((2, window_size_y - 60)),
                Img(&image),
            );
            Ok(())
        })?;

        self.square_font_info.execute(|image| {
            window.draw(
                &image
                    .area()
                    .translate((2, window_size_y - 30)),
                Img(&image),
            );
            Ok(())
        })?;

        let tile_size_px = self.tile_size_px;
        let offset_px = Vector::new(50, 120);

        let (map_size_x, map_size_y) = (self.map.extents()[0], self.map.extents()[1]);

        let tileset = &mut self.tileset;
        let map = &self.map;
        let entities = &self.entities;
        tileset.execute(|tileset| {
            for x in 0..map_size_x {
                for y in 0..map_size_y {
                    let pos = Vector::new(x as f32, y as f32);
                    let tile = &map[[x, y]];
                    if let Some(image) = tileset.get(&tile.glyph) {
                        let pos_px = offset_px + tile_size_px.times(pos);
                        window.draw(
                            &Rectangle::new(pos_px, image.area().size()),
                            Blended(&image, tile.color),
                        )
                    }
                }
            }
            for entity in entities.iter() {
                if let Some(image) = tileset.get(&entity.tile.glyph) {
                    let pos_px = offset_px + entity.pos.times(tile_size_px);
                    window.draw(
                        &Rectangle::new(pos_px, image.area().size()),
                        Blended(&image, entity.tile.color),
                    );
                }
            }
            Ok(())
        })?;

        let player = &self.entities[self.player_id];
        let full_health_width_px = 100.0;
        let current_health_width_px =
            (player.hp as f32 / player.max_hp as f32) * full_health_width_px;

        let map_size_px = tile_size_px.times(Vector::new(map_size_x as f32, map_size_y as f32));
        let health_bar_pos_px = offset_px + Vector::new(map_size_px.x, 0.0);

        // Full health
        window.draw(
            &Rectangle::new(health_bar_pos_px, (full_health_width_px, tile_size_px.y)),
            Col(Color::RED.with_alpha(0.5)),
        );

        // Current health
        window.draw(
            &Rectangle::new(health_bar_pos_px, (current_health_width_px, tile_size_px.y)),
            Col(Color::RED),
        );

        Ok(())
    }
}

fn generate_map(width: usize, height: usize) -> Array2D<Tile> {
    let default_tile = Tile {
        glyph: '.', color:
        Color::BLACK,
    };
    let mut map = Array2D::new([width, height], default_tile);
    for x in 0..width {
        for y in 0..height {
            let mut tile = Tile {
                glyph: '.',
                color: Color::BLACK,
            };

            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                tile.glyph = '#';
            }
            map[[x, y]] = tile;
        }
    }
    map
}

fn generate_entities() -> Vec<Entity> {
    vec![
        goblin(9, 6),
        goblin(2, 4),
        food(7, 5),
        food(4, 8),
    ]
}

fn goblin(x: i32, y: i32) -> Entity {
    Entity {
        pos: Vector::new(x, y),
        tile: Tile { glyph: 'g', color: Color::RED },
        hp: 1,
        max_hp: 1,
    }
}

fn food(x: i32, y: i32) -> Entity {
    Entity {
        pos: Vector::new(x, y),
        tile: Tile { glyph: '%', color: Color::PURPLE },
        hp: 0,
        max_hp: 0,
    }
}
