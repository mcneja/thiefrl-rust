mod cell_grid;
mod color_preset;
mod fontdata;
mod guard;
mod random_map;
mod speech_bubbles;

use rand::prelude::*;

use crate::cell_grid::*;
use crate::guard::*;
use crate::speech_bubbles::*;

use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::{Background::{Blended, Col}, Color, Image},
    input::Key,
    lifecycle::{run, Asset, Event, Settings, State, Window},
    Future, Result,
};

const BAR_HEIGHT: i32 = fontdata::LINE_HEIGHT + 2;
const BAR_BACKGROUND_COLOR: Color = Color { r: 0.0625, g: 0.0625, b: 0.0625, a: 1.0 };

struct Game {
    rng: MyRng,
    level: usize,
    map: Map,
    lines: Lines,
    player: Player,
    tileset: Asset<Vec<Image>>,
    tile_size_px: Vector,
    font_image: Image,
}

fn main() {
    let settings = Settings {
        scale: quicksilver::graphics::ImageScaleStrategy::Pixelate,
        resize: quicksilver::graphics::ResizeStrategy::Maintain,
        ..Default::default()
    };
    run::<Game>("ThiefRL 3", Vector::new(880, 760), settings);
}

fn move_player(game: &mut Game, mut dx: i32, mut dy: i32) {
    let player = &mut game.player;

    // Can't move if you're dead.

    if player.health == 0 {
        return;
    }

    // Are we trying to exit the level?

    let pos_new = Point::new(player.pos.x + dx, player.pos.y + dy);

    if !on_level(&game.map.cells, pos_new) && game.map.all_seen() && game.map.all_loot_collected() {
        game.level += 1;
        game.map = random_map::generate_map(&mut game.rng, game.level);

        game.player.pos = game.map.pos_start;
        game.player.dir = Point::new(0, 0);
        game.player.gold = 0;
        game.player.noisy = false;
        game.player.damaged_last_turn = false;
        game.player.finished_level = false;
        game.player.turns_remaining_underwater = 0;
        game.player.game_over = false;

        return;
    }

    if dx == 0 || dy == 0 {
        if blocked(&game.map, &player.pos, &pos_new) {
            return;
        }
    } else if blocked(&game.map, &player.pos, &pos_new) {
        if halts_slide(&game.map, &pos_new) {
            return;
        } else {
            // Attempting to move diagonally; may be able to slide along a wall.

            let v_blocked = blocked(&game.map, &player.pos, &(player.pos + Point::new(dx, 0)));
            let h_blocked = blocked(&game.map, &player.pos, &(player.pos + Point::new(0, dy)));

            if v_blocked {
                if h_blocked {
                    return;
                }

                dx = 0;
            } else {
                if !h_blocked {
                    return;
                }

                dy = 0;
            }
        }
    }

    pre_turn(game);

    let dpos = Point::new(dx, dy);
    game.player.dir = dpos;
    game.player.pos += dpos;
    game.player.gold += game.map.collect_loot_at(game.player.pos);

    // Generate movement noises.

    let cell_type = game.map.cells[[game.player.pos.x as usize, game.player.pos.y as usize]].cell_type;

    if cell_type == CellType::GroundWoodCreaky {
        make_noise(&mut game.map, &mut game.player, "\u{AE}creak\u{AF}");
    }

    advance_time(game);
}

fn make_noise(map: &mut Map, player: &mut Player, _noise: &str) {
    player.noisy = true;
//  txt::noise(game.player.pos, noise);

    let guards = map.find_guards_in_earshot(player.pos, 75);

    for guard in guards {
        guard.hear_thief();
    }
}

fn halts_slide(map: &Map, pos: &Point) -> bool {
    if pos.x < 0 || pos.x >= map.cells.extents()[0] as i32 || pos.y < 0 || pos.y >= map.cells.extents()[1] as i32 {
        return false;
    }

    if is_guard_at(map, pos.x, pos.y) {
        return true;
    }

    false
}

fn pre_turn(game: &mut Game) {
//  s_show_msgs = true;
//  s_bump_msg.clear();
//  txt::clear();
    game.player.noisy = false;
    game.player.damaged_last_turn = false;
    game.player.dir = Point::new(0, 0);
}

fn advance_time(game: &mut Game) {
    if game.map.cells[[game.player.pos.x as usize, game.player.pos.y as usize]].cell_type == CellType::GroundWater {
        if game.player.turns_remaining_underwater > 0 {
            game.player.turns_remaining_underwater -= 1;
        }
    } else {
        game.player.turns_remaining_underwater = 7;
    }

    guard_act_all(&mut game.rng, &mut game.lines, &mut game.map, &mut game.player);

/*
    map.recomputeVisibility(game.player.pos);
*/

    if game.map.all_seen() && game.map.all_loot_collected() {
        game.player.finished_level = true;
    }
}

fn on_level(map: &CellGrid, pos: Point) -> bool {
    let size_x = map.extents()[0] as i32;
    let size_y = map.extents()[1] as i32;
    pos.x >= 0 && pos.y >= 0 && pos.x < size_x && pos.y < size_y
}

fn blocked(map: &Map, pos_old: &Point, pos_new: &Point) -> bool {
    if !on_level(&map.cells, *pos_new) {
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
        let tile_size_px = Vector::new(16, 16);

        let tileset = Asset::new(Image::load(tiles_file).and_then(move |tiles| {
            let mut tileset = Vec::with_capacity(256);
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

        let font_image = Image::from_bytes(&crate::fontdata::BITMAP_DATA).unwrap();

        let random_seed = rand::random::<u64>();
        let mut rng = MyRng::seed_from_u64(random_seed);
        let level = 0;
        let map = random_map::generate_map(&mut rng, level);
        let player = make_player(&map.pos_start);
        let lines = new_lines();

        Ok(Self {
            rng,
            level: 0,
            lines,
            map,
            player,
            tileset,
            tile_size_px,
            font_image
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
        let font_image = &self.font_image;
        let level = self.level;

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

                let lit = map.cells[[player.pos.x as usize, player.pos.y as usize]].lit;
                let noisy = player.noisy;
                let damaged = player.damaged_last_turn;
                let hidden = player.hidden(map);

                let color =
                    if damaged {Color {r: 1.0, g: 0.0, b: 0.0, a: 1.0}}
                    else if noisy {color_preset::LIGHT_CYAN}
                    else if hidden {Color {r: 0.0625, g: 0.0625, b: 0.0625, a: 0.875}}
                    else if lit {color_preset::LIGHT_GRAY}
                    else {color_preset::LIGHT_BLUE};

                let image = &tileset[glyph];
                let pos = Vector::new(player.pos.x, (map_size_y - 1) as i32 - player.pos.y);
                let pos_px = offset_px + pos.times(tile_size_px);
                window.draw(
                    &Rectangle::new(pos_px, image.area().size()),
                    Blended(&image, color),
                );
            }
            for guard in guards {
                let glyph =
                    if guard.dir.y > 0 {210}
                    else if guard.dir.y < 0 {212}
                    else if guard.dir.x > 0 {209}
                    else if guard.dir.x < 0 {211}
                    else {212};

                let image = &tileset[glyph];
                let pos = Vector::new(guard.pos.x, (map_size_y - 1) as i32 - guard.pos.y);
                let pos_px = offset_px + pos.times(tile_size_px);
                let color = color_preset::LIGHT_MAGENTA;
                window.draw(
                    &Rectangle::new(pos_px, image.area().size()),
                    Blended(&image, color)
                );
            }
            for guard in guards {
                if let Some(glyph) = guard.overhead_icon(map, player) {
                    let image = &tileset[glyph];
                    let pos = Vector::new(guard.pos.x, (map_size_y - 1) as i32 - guard.pos.y);
                    let pos_px = offset_px + pos.times(tile_size_px) - Vector::new(0, 10);
                    let color = color_preset::LIGHT_YELLOW;
                    window.draw(
                        &Rectangle::new(pos_px, image.area().size()),
                        Blended(&image, color)
                    );
                }
            }

/*
            if let Some(guard) = guards.first() {
                if guard.region_goal != INVALID_REGION {
                    let distance_field = map.compute_distances_to_region(guard.region_goal);
                    for x in 0..map_size_x {
                        for y in 0..map_size_y {
                            let pos = Vector::new(x as f32, ((map_size_y - 1) - y) as f32);
                            let d = distance_field[[x, y]];
                            if d == 0 || d == INFINITE_COST {
                                continue;
                            }
                            let digit = (d % 10) + 48;
                            let band = d / 10;
                            let image = &tileset[digit];
                            let pos_px = offset_px + tile_size_px.times(pos);
                            let color = if band == 0 {color_preset::WHITE} else if band == 1 {color_preset::LIGHT_YELLOW} else {color_preset::DARK_GRAY};
                            window.draw(
                                &Rectangle::new(pos_px, image.area().size()),
                                Blended(&image, color),
                            )
                        }
                    }
                }
            }
*/

/*
            if let Some(guard) = guards.first() {
                let image = &tileset[255];
                if guard.region_prev != INVALID_REGION {

                    let region = &map.patrol_regions[guard.region_prev];
                    for x in region.pos_min.x .. region.pos_max.x {
                        for y in region.pos_min.y .. region.pos_max.y {
                            let pos = Vector::new(x as f32, ((map_size_y - 1) as i32 - y) as f32);
                            let pos_px = offset_px + tile_size_px.times(pos);
                            let color = Color {r:1.0, g:0.0, b:0.0, a:0.25};
                            window.draw(
                                &Rectangle::new(pos_px, image.area().size()),
                                Blended(&image, color),
                            )
                        }
                    }
                }
                if guard.region_goal != INVALID_REGION {
                    let region = &map.patrol_regions[guard.region_goal];
                    for x in region.pos_min.x .. region.pos_max.x {
                        for y in region.pos_min.y .. region.pos_max.y {
                            let pos = Vector::new(x as f32, ((map_size_y - 1) as i32 - y) as f32);
                            let pos_px = offset_px + tile_size_px.times(pos);
                            let color = Color {r:0.0, g:1.0, b:0.0, a:0.25};
                            window.draw(
                                &Rectangle::new(pos_px, image.area().size()),
                                Blended(&image, color),
                            )
                        }
                    }
                }
            }
*/

            window.flush()?;

//            draw_top_status_bar(window, font_image, player, level);
            draw_bottom_status_bar(window, font_image, tileset, map, player, level);

            Ok(())
        })?;

        Ok(())
    }
}

fn draw_bottom_status_bar(window: &mut Window, font_image: &Image, tileset: &Vec<Image>, map: &Map, player: &Player, level: usize) {
    let screen_size = window.screen_size();
    let screen_size_x: i32 = screen_size.x as i32;
    let screen_size_y: i32 = screen_size.y as i32;
    window.draw(
        &Rectangle::new((0, screen_size_y - BAR_HEIGHT), (screen_size_x, BAR_HEIGHT)),
        Col(BAR_BACKGROUND_COLOR),
    );

	let y_base = screen_size_y - BAR_HEIGHT;

    const HEALTH_COLOR: Color = Color { r: 0.65625, g: 0.0, b: 0.0, a: 1.0 };
    let mut x = 8;
    x = puts_proportional(window, font_image, x, y_base, "Health", &HEALTH_COLOR);
    x += 12;

    let tile_healthy = &tileset[213];
    let tile_unhealthy = &tileset[7];

    const TILE_SIZE_X: i32 = 16;

    for i in 0..player.health {
        window.draw(
            &Rectangle::new((x, y_base + 5), tile_healthy.area().size()),
            Blended(tile_healthy, HEALTH_COLOR)
        );
        x += TILE_SIZE_X;
    }
    for i in player.health..player.max_health {
        window.draw(
            &Rectangle::new((x, y_base + 5), tile_unhealthy.area().size()),
            Blended(tile_unhealthy, HEALTH_COLOR)
        );
        x += TILE_SIZE_X;
    }

    let player_underwater = map.cells[[player.pos.x as usize, player.pos.y as usize]].cell_type == CellType::GroundWater && player.turns_remaining_underwater > 0;

    if player_underwater {
        const AIR_COLOR: Color = Color { r: 0.328, g: 0.992, b: 0.992, a: 1.0 };
        const NO_AIR_COLOR: Color = Color { r: 0.0, g: 0.65625, b: 0.65625, a: 1.0 };

        x = screen_size_x / 4 - 16;
        x = puts_proportional(window, font_image, x, y_base, "Air", &AIR_COLOR);
        x += 8;

        let tile_air = &tileset[214];
        let tile_no_air = &tileset[7];

        for i in 0..player.turns_remaining_underwater - 1 {
            window.draw(
                &Rectangle::new((x, y_base + 5), tile_air.area().size()),
                Blended(tile_air, AIR_COLOR)
            );
            x += TILE_SIZE_X;
        }
        for i in player.turns_remaining_underwater - 1 .. 5 {
            window.draw(
                &Rectangle::new((x, y_base + 5), tile_no_air.area().size()),
                Blended(tile_no_air, NO_AIR_COLOR)
            );
            x += TILE_SIZE_X;
        }
    }

    // Draw the tallies of what's been seen and collected.

    let percent_seen: usize = map.percent_seen();

    {
        const COLOR: Color = Color { r: 0.212, g: 0.212, b: 0.212, a: 1.0 };
        let seen_msg = format!("Level {}: {}% Seen", level + 1, percent_seen);
        let (x_min, x_max) = get_horizontal_extents(&seen_msg);
        let x = (screen_size_x - (x_max - x_min)) / 2;
        puts_proportional(window, font_image, x, y_base, &seen_msg, &COLOR);
    }

    {
        const COLOR: Color = Color { r: 0.996, g: 0.996, b: 0.212, a: 1.0 };
        let loot_msg =
            if percent_seen < 100 {
                format!("Loot {}/?", player.gold)
            } else {
                format!("Loot {}/{}", player.gold, map.total_loot)
            };
        let (x_min, x_max) = get_horizontal_extents(&loot_msg);
        let x = screen_size_x - (8 + (x_max - x_min));
        puts_proportional(window, font_image, x, y_base, &loot_msg, &COLOR);
    }
}

fn draw_top_status_bar(window: &mut Window, font_image: &Image, player: &Player, level: usize) {
    let screen_size = window.screen_size();
    let screen_size_x: i32 = screen_size.x as i32;
    window.draw(
        &Rectangle::new((0, 0), (screen_size_x, BAR_HEIGHT)),
        Col(BAR_BACKGROUND_COLOR),
    );

	let y_base = BAR_HEIGHT - fontdata::LINE_HEIGHT;

/*
    if s_showHelp {
        sprintf_s(msgBuffer, sizeof(msgBuffer), "Page %d of %d", s_helpPage + 1, 2);

        int xMin, xMax;
        txt::getHorizontalExtents(msgBuffer, xMin, xMax);

        int x = screenSizeX - (8 + (xMax - xMin));

        txt::putsProportional(x, y_base, msgBuffer);

        "Press left/right arrow keys to view help, or F1 to close"
    } else
*/
    {
        let msg =
            if player.game_over || player.health == 0 {
                format!("You are dead! Press Ctrl+N for a new game or Ctrl+R to restart.")
            } else if player.finished_level {
                format!("Level {} complete! Move off the edge of the map to advance to the next level.", level + 1)
            } else if level == 0 {
                format!("Welcome to level {}. Collect the gold coins and reveal the whole mansion. (Press F1 for help.)", level + 1)
            } else if level == 1 {
                format!("Welcome to level {}. Watch out for the patrolling guard! (Press F1 for help.)", level + 1)
            } else {
                format!("Press F1 for help")
            };

        puts_proportional(window, font_image, 8, y_base, &msg, &color_preset::WHITE);
    }
}
