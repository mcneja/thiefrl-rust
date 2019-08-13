mod cell_grid;
mod random_map;

use crate::cell_grid::*;

use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::{Background::{Blended}, Color, Image},
    input::Key,
    lifecycle::{run, Asset, Event, Settings, State, Window},
    Future, Result,
};

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

fn tile_def(tile_type: CellType) -> Tile {
    match tile_type {
        CellType::GroundNormal     => Tile { glyph: 128, color: color_preset::LIGHT_GRAY, blocks_player: false },
        CellType::GroundGravel     => Tile { glyph: 130, color: color_preset::LIGHT_GRAY, blocks_player: false },
        CellType::GroundGrass      => Tile { glyph: 132, color: color_preset::DARK_GREEN, blocks_player: false },
        CellType::GroundWater      => Tile { glyph: 134, color: color_preset::LIGHT_BLUE, blocks_player: false },
        CellType::GroundMarble     => Tile { glyph: 136, color: color_preset::DARK_CYAN, blocks_player: false },
        CellType::GroundWood       => Tile { glyph: 138, color: color_preset::DARK_BROWN, blocks_player: false },
        CellType::GroundWoodCreaky => Tile { glyph: 138, color: color_preset::DARK_BROWN, blocks_player: false },

                  //  NSEW
        CellType::Wall0000 => Tile { glyph: 176, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall0001 => Tile { glyph: 177, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall0010 => Tile { glyph: 177, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall0011 => Tile { glyph: 177, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall0100 => Tile { glyph: 178, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall0101 => Tile { glyph: 179, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall0110 => Tile { glyph: 182, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall0111 => Tile { glyph: 185, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall1000 => Tile { glyph: 178, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall1001 => Tile { glyph: 180, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall1010 => Tile { glyph: 181, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall1011 => Tile { glyph: 184, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall1100 => Tile { glyph: 178, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall1101 => Tile { glyph: 186, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall1110 => Tile { glyph: 183, color: color_preset::LIGHT_GRAY, blocks_player: true },
        CellType::Wall1111 => Tile { glyph: 187, color: color_preset::LIGHT_GRAY, blocks_player: true },

        CellType::OneWayWindowE => Tile { glyph: 196, color: color_preset::LIGHT_GRAY, blocks_player: false },
        CellType::OneWayWindowW => Tile { glyph: 197, color: color_preset::LIGHT_GRAY, blocks_player: false },
        CellType::OneWayWindowN => Tile { glyph: 198, color: color_preset::LIGHT_GRAY, blocks_player: false },
        CellType::OneWayWindowS => Tile { glyph: 199, color: color_preset::LIGHT_GRAY, blocks_player: false },
        CellType::PortcullisNS  => Tile { glyph: 128, color: color_preset::LIGHT_GRAY, blocks_player: false },
        CellType::PortcullisEW  => Tile { glyph: 128, color: color_preset::LIGHT_GRAY, blocks_player: false },
        CellType::WindowNS      => Tile { glyph: 189, color: color_preset::LIGHT_GRAY, blocks_player: false },
        CellType::WindowEW      => Tile { glyph: 188, color: color_preset::LIGHT_GRAY, blocks_player: false },
        CellType::DoorNS        => Tile { glyph: 189, color: color_preset::LIGHT_GRAY, blocks_player: false },
        CellType::DoorEW        => Tile { glyph: 188, color: color_preset::LIGHT_GRAY, blocks_player: false },
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
    map: CellGrid,
    items: Vec<Item>,
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

fn on_level(map: &CellGrid, pos: &Point) -> bool {
    let size_x = map.extents()[0] as i32;
    let size_y = map.extents()[1] as i32;
	pos.x >= 0 && pos.y >= 0 && pos.x < size_x && pos.y < size_y
}

fn blocked(map: &CellGrid, pos_old: &Point, pos_new: &Point) -> bool {
	if !on_level(map, pos_new) {
		return true;
    }

    let tile_type = map[[pos_new.x as usize, pos_new.y as usize]].cell_type;
    let tile = tile_def(tile_type);

	if tile.blocks_player {
		return true;
    }

    if tile_type == CellType::OneWayWindowE && pos_new.x <= pos_old.x {
        return true;
    }

    if tile_type == CellType::OneWayWindowW && pos_new.x >= pos_old.x {
        return true;
    }

    if tile_type == CellType::OneWayWindowN && pos_new.y <= pos_old.y {
        return true;
    }

    if tile_type == CellType::OneWayWindowS && pos_new.y >= pos_old.y {
        return true;
    }

//	if (Guard::at(map, pos_new))
//		return true;

	false
}

fn glyph_for_item(kind: ItemKind) -> usize {
    match kind {
        ItemKind::Chair => 148,
        ItemKind::Table => 146,
        ItemKind::Bush => 144,
        ItemKind::Coin => 158,
        ItemKind::DoorNS => 169,
        ItemKind::DoorEW => 167,
        ItemKind::PortcullisNS => 194,
        ItemKind::PortcullisEW => 194,
    }
}

fn color_for_item(kind: ItemKind) -> Color {
    match kind {
        ItemKind::Chair => color_preset::DARK_BROWN,
        ItemKind::Table => color_preset::DARK_BROWN,
        ItemKind::Bush => color_preset::DARK_GREEN,
        ItemKind::Coin => color_preset::LIGHT_YELLOW,
        ItemKind::DoorNS => color_preset::DARK_BROWN,
        ItemKind::DoorEW => color_preset::DARK_BROWN,
        ItemKind::PortcullisNS => color_preset::LIGHT_GRAY,
        ItemKind::PortcullisEW => color_preset::LIGHT_GRAY,
    }
}

impl State for Game {
    /// Load the assets and initialise the game
    fn new() -> Result<Self> {
        let tiles_file = "tiles.png";

        let random_seed = rand::random::<u64>();

        let initial_state = random_map::generate_map(random_seed);
        let mut entities = Vec::new();

        let player_id = entities.len();
        entities.push(Entity {
            pos: initial_state.pos_start,
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
            map: initial_state.cells,
            items: initial_state.items,
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
                    Key::Numpad1 | Key::End      => move_player(self, -1, -1),
                    Key::Numpad2 | Key::Down     => move_player(self,  0, -1),
                    Key::Numpad3 | Key::PageDown => move_player(self,  1, -1),
                    Key::Numpad4 | Key::Left     => move_player(self, -1,  0),
                    Key::Numpad5                 => move_player(self,  0,  0),
                    Key::Numpad6 | Key::Right    => move_player(self,  1,  0),
                    Key::Numpad7 | Key::Home     => move_player(self, -1,  1),
                    Key::Numpad8 | Key::Up       => move_player(self,  0,  1),
                    Key::Numpad9 | Key::PageUp   => move_player(self,  1,  1),
                    Key::Escape                  => window.close(),
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
        let offset_px = Vector::new(0, 0);

        let map = &self.map;
        let map_size_x = map.extents()[0];
        let map_size_y = map.extents()[1];
        let entities = &self.entities;
        let items = &self.items;
        self.tileset.execute(|tileset| {
            for x in 0..map_size_x {
                for y in 0..map_size_y {
                    let pos = Vector::new(x as f32, ((map_size_y - 1) - y) as f32);
                    let cell_type = map[[x, y]].cell_type;
                    let tile = tile_def(cell_type);
                    let image = &tileset[tile.glyph];
                    let pos_px = offset_px + tile_size_px.times(pos);
                    window.draw(
                        &Rectangle::new(pos_px, image.area().size()),
                        Blended(&image, tile.color),
                    )
                }
            }
            for item in items.iter() {
                let pos = Vector::new(item.pos.x, (map_size_y - 1) as i32 - item.pos.y);
                let pos_px = offset_px + pos.times(tile_size_px);
                let glyph = glyph_for_item(item.kind);
                let color = color_for_item(item.kind);
                let image = &tileset[glyph];
                window.draw(
                    &Rectangle::new(pos_px, image.area().size()),
                    Blended(&image, color),
                );
            }
            for entity in entities.iter() {
                let image = &tileset[entity.tile.glyph];
                let pos = Vector::new(entity.pos.x, (map_size_y - 1) as i32 - entity.pos.y);
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
