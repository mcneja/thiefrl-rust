mod cell_grid;
mod color_preset;
mod guard;
mod random_map;

use crate::cell_grid::*;
use crate::guard::*;

use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::{Background::{Blended}, Color, Image},
    input::Key,
    lifecycle::{run, Asset, Event, Settings, State, Window},
    Future, Result,
};

struct Game {
    map: Map,
    player: Player,
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
    let player = &mut game.player;
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

fn blocked(map: &Map, pos_old: &Point, pos_new: &Point) -> bool {
	if !on_level(&map.cells, pos_new) {
		return true;
    }

    let tile_type = map.cells[[pos_new.x as usize, pos_new.y as usize]].cell_type;
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

    if is_guard_at(map, pos_new.x, pos_new.y) {
        return true;
    }

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

        let map = random_map::generate_map(random_seed);
        let player = make_player(&map.pos_start);

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
            player,
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
        let map_size_x = map.cells.extents()[0];
        let map_size_y = map.cells.extents()[1];
        let items = &self.map.items;
        let guards = &self.map.guards;
        let player = &self.player;
        self.tileset.execute(|tileset| {
            for x in 0..map_size_x {
                for y in 0..map_size_y {
                    let pos = Vector::new(x as f32, ((map_size_y - 1) - y) as f32);
                    let cell = &map.cells[[x, y]];
                    let tile = tile_def(cell.cell_type);
                    let image = &tileset[tile.glyph];
                    let pos_px = offset_px + tile_size_px.times(pos);
                    let color = if cell.lit || tile.ignores_lighting {tile.color} else {color_preset::DARK_BLUE};
                    window.draw(
                        &Rectangle::new(pos_px, image.area().size()),
                        Blended(&image, color),
                    )
                }
            }
            for item in items {
                let pos = Vector::new(item.pos.x, (map_size_y - 1) as i32 - item.pos.y);
                let cell = &map.cells[[item.pos.x as usize, item.pos.y as usize]];
                let pos_px = offset_px + pos.times(tile_size_px);
                let glyph = glyph_for_item(item.kind);
                let color = if cell.lit {color_for_item(item.kind)} else {color_preset::DARK_BLUE};
                let image = &tileset[glyph];
                window.draw(
                    &Rectangle::new(pos_px, image.area().size()),
                    Blended(&image, color),
                );
            }
            {
                let glyph = 208;
                let color = color_preset::LIGHT_CYAN;
                let image = &tileset[glyph];
                let pos = Vector::new(player.pos.x, (map_size_y - 1) as i32 - player.pos.y);
                let pos_px = offset_px + pos.times(tile_size_px);
                window.draw(
                    &Rectangle::new(pos_px, image.area().size()),
                    Blended(&image, color),
                );
            }
            for guard in guards {
                let glyph = 212;
                let image = &tileset[glyph];
                let pos = Vector::new(guard.pos.x, (map_size_y - 1) as i32 - guard.pos.y);
                let pos_px = offset_px + pos.times(tile_size_px);
                let color = color_preset::LIGHT_MAGENTA;
                window.draw(
                    &Rectangle::new(pos_px, image.area().size()),
                    Blended(&image, color)
                );
            }
            Ok(())
        })?;

        Ok(())
    }
}
