use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::{Background::{Blended}, Color, Image},
    input::Key,
    lifecycle::{run, Asset, Event, Settings, State, Window},
    Future, Result,
};

use multiarray::*;

type Point = vector2d::Vector2D<i32>;
type TileTypeGrid = Array2D<TileType>;

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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[allow(dead_code)]
enum TileType {
	GroundNormal,
	GroundGravel,
	GroundGrass,
	GroundWater,
	GroundMarble,
	GroundWood,
	GroundWoodCreaky,

	//  NSEW
	Wall0000,
	Wall0001,
	Wall0010,
	Wall0011,
	Wall0100,
	Wall0101,
	Wall0110,
	Wall0111,
	Wall1000,
	Wall1001,
	Wall1010,
	Wall1011,
	Wall1100,
	Wall1101,
	Wall1110,
	Wall1111,

	OneWayWindowE,
	OneWayWindowW,
	OneWayWindowN,
	OneWayWindowS,
	PortcullisNS,
	PortcullisEW,
	WindowNS,
	WindowEW,
	DoorNS,
	DoorEW,
}

fn tile_def(tile_type: TileType) -> Tile {
    match tile_type {
        TileType::GroundNormal     => Tile { glyph: 128, color: color_preset::LIGHT_GRAY, blocks_player: false },
        TileType::GroundGravel     => Tile { glyph: 130, color: color_preset::LIGHT_GRAY, blocks_player: false },
        TileType::GroundGrass      => Tile { glyph: 132, color: color_preset::DARK_GREEN, blocks_player: false },
        TileType::GroundWater      => Tile { glyph: 134, color: color_preset::LIGHT_BLUE, blocks_player: false },
        TileType::GroundMarble     => Tile { glyph: 136, color: color_preset::DARK_CYAN, blocks_player: false },
        TileType::GroundWood       => Tile { glyph: 138, color: color_preset::DARK_BROWN, blocks_player: false },
        TileType::GroundWoodCreaky => Tile { glyph: 138, color: color_preset::DARK_BROWN, blocks_player: false },

                  //  NSEW
        TileType::Wall0000 => Tile { glyph: 176, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall0001 => Tile { glyph: 177, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall0010 => Tile { glyph: 177, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall0011 => Tile { glyph: 177, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall0100 => Tile { glyph: 178, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall0101 => Tile { glyph: 179, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall0110 => Tile { glyph: 182, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall0111 => Tile { glyph: 185, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall1000 => Tile { glyph: 178, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall1001 => Tile { glyph: 180, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall1010 => Tile { glyph: 181, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall1011 => Tile { glyph: 184, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall1100 => Tile { glyph: 178, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall1101 => Tile { glyph: 186, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall1110 => Tile { glyph: 183, color: color_preset::LIGHT_GRAY, blocks_player: true },
        TileType::Wall1111 => Tile { glyph: 187, color: color_preset::LIGHT_GRAY, blocks_player: true },

        TileType::OneWayWindowE => Tile { glyph: 196, color: color_preset::LIGHT_GRAY, blocks_player: false },
        TileType::OneWayWindowW => Tile { glyph: 197, color: color_preset::LIGHT_GRAY, blocks_player: false },
        TileType::OneWayWindowN => Tile { glyph: 198, color: color_preset::LIGHT_GRAY, blocks_player: false },
        TileType::OneWayWindowS => Tile { glyph: 199, color: color_preset::LIGHT_GRAY, blocks_player: false },
        TileType::PortcullisNS  => Tile { glyph: 128, color: color_preset::LIGHT_GRAY, blocks_player: false },
        TileType::PortcullisEW  => Tile { glyph: 128, color: color_preset::LIGHT_GRAY, blocks_player: false },
        TileType::WindowNS      => Tile { glyph: 189, color: color_preset::LIGHT_GRAY, blocks_player: false },
        TileType::WindowEW      => Tile { glyph: 188, color: color_preset::LIGHT_GRAY, blocks_player: false },
        TileType::DoorNS        => Tile { glyph: 189, color: color_preset::LIGHT_GRAY, blocks_player: false },
        TileType::DoorEW        => Tile { glyph: 188, color: color_preset::LIGHT_GRAY, blocks_player: false },
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Tile {
    glyph: usize,
    color: Color,
    blocks_player: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct Entity {
    pos: Point,
    tile: Tile,
}

struct Game {
    map: TileTypeGrid,
    entities: Vec<Entity>,
    player_id: usize,
    tileset: Asset<Vec<Image>>,
    tile_size_px: Vector,
}

fn main() {
    let settings = Settings {
        scale: quicksilver::graphics::ImageScaleStrategy::Pixelate,
        resize: quicksilver::graphics::ResizeStrategy::Maintain,
        ..Default::default()
    };
    run::<Game>("ThiefRL 3", Vector::new(880, 760), settings);
}

fn move_player(game: &mut Game, dx: i32, dy: i32) {
    let player = &mut game.entities[game.player_id];
    let pos_new = Point::new(player.pos.x + dx, player.pos.y + dy);

	if !blocked(&game.map, &player.pos, &pos_new) {
        player.pos = pos_new;
    } else {
        // Attempting to move diagonally; may be able to slide along a wall.

        let pos_slide_v = Point::new(player.pos.x, pos_new.y);
        let pos_slide_h = Point::new(pos_new.x, player.pos.y);

        if !blocked(&game.map, &player.pos, &pos_slide_v) {
            player.pos = pos_slide_v;
        }
        else if !blocked(&game.map, &player.pos, &pos_slide_h) {
            player.pos = pos_slide_h;
        }
	}
}

fn on_level(map: &TileTypeGrid, pos: &Point) -> bool {
    let size_x = map.extents()[0] as i32;
    let size_y = map.extents()[1] as i32;
	pos.x >= 0 && pos.y >= 0 && pos.x < size_x && pos.y < size_y
}

fn blocked(map: &TileTypeGrid, pos_old: &Point, pos_new: &Point) -> bool {
	if !on_level(map, pos_new) {
		return true;
    }

    let tile_type = map[[pos_new.x as usize, pos_new.y as usize]];
    let tile = tile_def(tile_type);

	if tile.blocks_player {
		return true;
    }

    if tile_type == TileType::OneWayWindowE && pos_new.x <= pos_old.x {
        return true;
    }

    if tile_type == TileType::OneWayWindowW && pos_new.x >= pos_old.x {
        return true;
    }

    if tile_type == TileType::OneWayWindowN && pos_new.y <= pos_old.y {
        return true;
    }

    if tile_type == TileType::OneWayWindowS && pos_new.y >= pos_old.y {
        return true;
    }

//	if (Guard::at(map, pos_new) || ShipCaptain::at(map, pos_new) || Leader::at(map, pos_new))
//		return true;

	false
}

impl State for Game {
    /// Load the assets and initialise the game
    fn new() -> Result<Self> {
        let tiles_file = "tiles.png";

        let map = generate_map();
        let mut entities = generate_entities();

        let player_id = entities.len();
        entities.push(Entity {
            pos: Point::new(5, 3),
            tile: Tile{ glyph: 208, color: color_preset::LIGHT_CYAN, blocks_player: false },
        });

        let tile_size_px = Vector::new(16, 16);

        let tileset = Asset::new(Image::load(tiles_file).and_then(move |tiles| {
            let mut tileset = Vec::new();
            for y in 0..16 {
                for x in 0..16 {
                    let pos_px = tile_size_px.times(Vector::new(x, 15 - y));
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

    /// Handle input
    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        match event {
            Event::Key(key, quicksilver::input::ButtonState::Pressed) =>
                match key {
                    Key::Numpad1              => move_player(self, -1,  1),
                    Key::Numpad2 | Key::Down  => move_player(self,  0,  1),
                    Key::Numpad3              => move_player(self,  1,  1),
                    Key::Numpad4 | Key::Left  => move_player(self, -1,  0),
                    Key::Numpad6 | Key::Right => move_player(self,  1,  0),
                    Key::Numpad7              => move_player(self, -1, -1),
                    Key::Numpad8 | Key::Up    => move_player(self,  0, -1),
                    Key::Numpad9              => move_player(self,  1, -1),
                    Key::Escape               => window.close(),
                    _ => ()
                }
            _ => ()
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
                    let tile_type = map[[x, y]];
                    let tile = tile_def(tile_type);
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

fn generate_map() -> TileTypeGrid {
    let map_size = vector2d::Vector2D::new(32, 32);
    let mut map = TileTypeGrid::new([map_size.x, map_size.y], TileType::GroundNormal);
    for x in 1..map_size.x-1 {
        map[[x, 0]] = TileType::Wall0011;
        map[[x, map_size.y-1]] = TileType::Wall0011;
    }
    for y in 1..map_size.y-1 {
        map[[0, y]] = TileType::Wall1100;
        map[[map_size.x-1, y]] = TileType::Wall1100;
    }
    map[[0, 0]] = TileType::Wall0110;
    map[[map_size.x-1, 0]] = TileType::Wall0101;
    map[[0, map_size.y-1]] = TileType::Wall1010;
    map[[map_size.x-1, map_size.y-1]] = TileType::Wall1001;
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
        pos: Point::new(x, y),
        tile: Tile { glyph: 212, color: color_preset::LIGHT_MAGENTA, blocks_player: true },
    }
}

fn coin(x: i32, y: i32) -> Entity {
    Entity {
        pos: Point::new(x, y),
        tile: Tile { glyph: 158, color: color_preset::LIGHT_YELLOW, blocks_player: false },
    }
}
