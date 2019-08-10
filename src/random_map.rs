use crate::cell_grid::*;
use rand::prelude::*;
use std::cmp::min;
use std::cmp::max;
use multiarray::Array2D;

const OUTER_BORDER: i32 = 3;

const ROOM_SIZE_X: i32 = 5;
const ROOM_SIZE_Y: i32 = 5;

pub fn generate_map(seed: u64) -> (CellGrid, Point) {
    let map_size = vector2d::Vector2D::new(32, 32);
    let default_cell = Cell {
        cell_type: CellType::GroundNormal,
        visible: false,
        lit: false,
        seen: false,
        visited: false,
        region: 0,
        visit_stamp: 0,
    };
    let mut map = CellGrid::new([map_size.x, map_size.y], default_cell);
    for x in 1..map_size.x-1 {
        map[[x, 0]].cell_type = CellType::Wall0011;
        map[[x, map_size.y-1]].cell_type = CellType::Wall0011;
    }
    for y in 1..map_size.y-1 {
        map[[0, y]].cell_type = CellType::Wall1100;
        map[[map_size.x-1, y]].cell_type = CellType::Wall1100;
    }
    map[[0, 0]].cell_type = CellType::Wall0110;
    map[[map_size.x-1, 0]].cell_type = CellType::Wall0101;
    map[[0, map_size.y-1]].cell_type = CellType::Wall1010;
    map[[map_size.x-1, map_size.y-1]].cell_type = CellType::Wall1001;

    let mut rng = rand_pcg::Pcg32::seed_from_u64(seed);
    let player_x = rng.gen_range(1, (map_size.x - 1) as i32);
    let player_y = rng.gen_range(1, (map_size.y - 1) as i32);

    (map, Point::new(player_x, player_y))
}

fn generate_siheyuan(level: i32, rng: &mut impl Rng) -> CellGrid {
	let mut size_x: i32 = 0;
    for _ in 0..min(3, level) {
        size_x += rng.gen_range(0, 2);
    }
	size_x *= 2;
	size_x += 3;

	let mut size_y: i32 = 0;
    if level == 0 {
		size_y = 2;
	} else {
		size_y = 3;
		for _ in 0..min(4, level - 1) {
			size_y += rng.gen_range(0, 2);
        }
	}

	let mirror_x = true;
	let mirror_y = false;

    let inside: Array2D<bool> = make_siheyuan_room_grid(size_x as usize, size_y as usize, rng);

	// Compute wall offsets.

	let (offset_x, offset_y) = offset_walls(mirror_x, mirror_y, &inside, rng);

    let default_cell = Cell {
        cell_type: CellType::GroundNormal,
        visible: false,
        lit: false,
        seen: false,
        visited: false,
        region: 0,
        visit_stamp: 0,
    };
    let map = CellGrid::new([size_x as usize, size_y as usize], default_cell);
    map

    /*

	// Convert the room descriptions to walls.

	plotWalls(inside, offset_x, offset_y, map);

	// Fix up walls.

	map.fixup();

	// Create exits connecting rooms.

	std::vector<Room> rooms;
	std::vector<Adjacency> adjacencies;

	createExits(
		level,
		mirror_x,
		mirror_y,
		inside,
		offset_x,
		offset_y,
		rooms,
		adjacencies,
		map,
		g_player.m_pos);

	// Place loot.

	placeLoot(rooms, adjacencies, map);

	// Place exterior junk.

	placeExteriorBushes(map);
	placeFrontPillars(map);

	// Place guards.

	init_pathing(map);

	placeGuards(level, rooms, map);

	markExteriorAsSeen(map);
    */
}

fn make_siheyuan_room_grid(size_x: usize, size_y: usize, rng: &mut impl Rng) -> Array2D<bool> {
    let mut inside = Array2D::new([size_x, size_y], true);

	let half_x = (size_x + 1) / 2;

    for _ in 0..(size_y * half_x) / 4 {
		let x = rng.gen_range(0, half_x);
		let y = rng.gen_range(0, size_y);
		inside[[x, y]] = false;
	}

    for y in 0..size_y {
		for x in half_x..size_x {
			inside[[x, y]] = inside[[(size_x - 1) - x, y]];
		}
	}

    inside
}

fn offset_walls(mirror_x: bool, mirror_y: bool, inside: &Array2D<bool>, rng: &mut impl Rng) -> (Array2D<i32>, Array2D<i32>) {
	let rooms_x = inside.extents()[0];
	let rooms_y = inside.extents()[1];

    let mut offset_x = Array2D::new([rooms_x + 1, rooms_y], 0);
    let mut offset_y = Array2D::new([rooms_x, rooms_y + 1], 0);

    {
	    let i = rng.gen_range(0, 3) - 1;
        for y in 0..rooms_y {
            offset_x[[0, y]] = i;
        }
    }

    {
        let i = rng.gen_range(0, 3) - 1;
        for y in 0..rooms_y {
            offset_x[[rooms_x, y]] = i;
        }
    }

    {
        let i = rng.gen_range(0, 3) - 1;
        for x in 0..rooms_x {
            offset_y[[x, 0]] = i;
        }
    }

    {
        let i = rng.gen_range(0, 3) - 1;
        for x in 0..rooms_x {
            offset_y[[x, rooms_y]] = i;
        }
    }

	for x in 1..rooms_x {
		for y in 0..rooms_y {
			offset_x[[x, y]] = rng.gen_range(0, 3) - 1;
		}
	}

	for x in 0..rooms_x {
		for y in 1..rooms_y {
			offset_y[[x, y]] = rng.gen_range(0, 3) - 1;
		}
	}

	for x in 1..rooms_x {
		for y in 1..rooms_y {
			if rng.gen_range(0, 2) == 0 {
				offset_x[[x, y]] = offset_x[[x, y-1]];
			} else {
				offset_y[[x, y]] = offset_y[[x-1, y]];
			}
		}
	}

	if mirror_x {
		if (rooms_x & 1) == 0 {
			let x_mid = rooms_x / 2;
			for y in 0..rooms_y {
				offset_x[[x_mid, y]] = 0;
			}
		}

		for x in 0..(rooms_x + 1) / 2 {
			for y in 0..rooms_y {
				offset_x[[rooms_x - x, y]] = 1 - offset_x[[x, y]];
			}
		}

		for x in 0..rooms_x / 2 {
			for y in 0..rooms_y + 1 {
				offset_y[[(rooms_x - 1) - x, y]] = offset_y[[x, y]];
			}
		}
	}

	if mirror_y {
		if (rooms_y & 1) == 0 {
			let y_mid = rooms_y / 2;
			for x in 0..rooms_x {
				offset_y[[x, y_mid]] = 0;
			}
		}

		for y in 0..(rooms_y + 1) / 2 {
			for x in 0..rooms_x {
				offset_y[[x, rooms_y - y]] = 1 - offset_y[[x, y]];
			}
		}

		for y in 0..rooms_y / 2 {
			for x in 0..rooms_x + 1 {
				offset_x[[x, (rooms_y - 1) - y]] = offset_x[[x, y]];
			}
		}
	}

	let mut room_offset_x = std::i32::MIN;
	let mut room_offset_y = std::i32::MIN;

	for y in 0..rooms_y {
		room_offset_x = max(room_offset_x, -offset_x[[0, y]]);
	}

	for x in 0..rooms_x {
		room_offset_y = max(room_offset_y, -offset_y[[x, 0]]);
	}

	room_offset_x += OUTER_BORDER;
	room_offset_y += OUTER_BORDER;

	for x in 0..rooms_x + 1 {
		for y in 0..rooms_y {
			offset_x[[x, y]] += room_offset_x + (x as i32) * ROOM_SIZE_X;
		}
	}

	for x in 0..rooms_x {
		for y in 0..rooms_y + 1 {
			offset_y[[x, y]] += room_offset_y + (y as i32) * ROOM_SIZE_Y;
		}
	}

    (offset_x, offset_y)
}
