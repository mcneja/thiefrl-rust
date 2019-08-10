use crate::cell_grid::*;
use rand::prelude::*;
use std::cmp::min;
use std::cmp::max;
use multiarray::Array2D;

const OUTER_BORDER: i32 = 3;

const ROOM_SIZE_X: i32 = 5;
const ROOM_SIZE_Y: i32 = 5;

pub fn generate_map(seed: u64) -> (CellGrid, Point) {
    let mut rng = rand_pcg::Pcg32::seed_from_u64(seed);

    let map = generate_siheyuan(4, &mut rng);

    /*
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
    */

    let player_x = (map.extents()[0] / 2) as i32;
    let player_y = (map.extents()[1] - 1) as i32;

    (map, Point::new(player_x, player_y))
}

fn generate_siheyuan(level: i32, rng: &mut impl Rng) -> CellGrid {
	let mut size_x: i32 = 0;
    for _ in 0..min(3, level) {
        size_x += rng.gen_range(0, 2);
    }
	size_x *= 2;
	size_x += 3;

	let mut size_y: i32;
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

	// Convert the room descriptions to walls.

    let mut map = plot_walls(&inside, &offset_x, &offset_y);

	// Fix up walls.

	fixup_walls(&mut map);

    map

    /*
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

fn plot_walls(inside: &Array2D<bool>, offset_x: &Array2D<i32>, offset_y: &Array2D<i32>) -> CellGrid {
	let cx = inside.extents()[0];
	let cy = inside.extents()[1];

	let mut map_x = 0;
	let mut map_y = 0;

	for y in 0..cy {
		map_x = max(map_x, offset_x[[cx, y]]);
	}

	for x in 0..cx {
		map_y = max(map_y, offset_y[[x, cy]]);
	}

	map_x += OUTER_BORDER + 1;
	map_y += OUTER_BORDER + 1;

    let default_cell = Cell {
        cell_type: CellType::GroundNormal,
        visible: false,
        lit: false,
        seen: false,
        visited: false,
        region: 0,
        visit_stamp: 0,
    };
    let mut map = CellGrid::new([map_x as usize, map_y as usize], default_cell);

	// Super hacky: put down grass under all the rooms to plug holes, and light the interior.

	for rx in 0..cx {
		for ry in 0..cy {
			let x0 = offset_x[[rx, ry]];
			let x1 = offset_x[[rx + 1, ry]] + 1;
			let y0 = offset_y[[rx, ry]];
			let y1 = offset_y[[rx, ry + 1]] + 1;

			for x in x0..x1 {
				for y in y0..y1 {
                    let cell = &mut map[[x as usize, y as usize]];
					cell.cell_type = CellType::GroundGrass;
					cell.lit = true;
				}
			}
		}
	}

	// Draw walls. Really this should be done in createExits, where the
	//  walls are getting decorated with doors and windows.

	for rx in 0..cx {
		for ry in 0..cy {
			let indoors = inside[[rx, ry]];

			let x0 = offset_x[[rx, ry]];
			let x1 = offset_x[[rx + 1, ry]];
			let y0 = offset_y[[rx, ry]];
			let y1 = offset_y[[rx, ry + 1]];

			if rx == 0 || indoors {
				plot_ns_wall(&mut map, x0, y0, y1);
            }

			if rx == cx - 1 || indoors {
				plot_ns_wall(&mut map, x1, y0, y1);
            }

			if ry == 0 || indoors {
				plot_ew_wall(&mut map, x0, y0, x1);
            }

			if ry == cy - 1 || indoors {
				plot_ew_wall(&mut map, x0, y1, x1);
            }
		}
	}

    map
}

fn plot_ns_wall(map: &mut CellGrid, x0: i32, y0: i32, y1: i32) {
	for y in y0..=y1 {
		map[[x0 as usize, y as usize]].cell_type = CellType::Wall0000;
	}
}

fn plot_ew_wall(map: &mut CellGrid, x0: i32, y0: i32, x1: i32) {
	for x in x0..=x1 {
		map[[x as usize, y0 as usize]].cell_type = CellType::Wall0000;
	}
}

fn fixup_walls(map: &mut CellGrid) {
	for x in 0..map.extents()[0] {
		for y in 0..map.extents()[1] {
			if is_wall(map[[x, y]].cell_type) {
				map[[x, y]].cell_type = wall_type_from_neighbors(neighboring_walls(&map, x, y));
			}
		}
	}
}

fn wall_type_from_neighbors(neighbors: u32) -> CellType {
    match neighbors {
        0  => CellType::Wall0000,
        1  => CellType::Wall0001,
        2  => CellType::Wall0010,
        3  => CellType::Wall0011,
        4  => CellType::Wall0100,
        5  => CellType::Wall0101,
        6  => CellType::Wall0110,
        7  => CellType::Wall0111,
        8  => CellType::Wall1000,
        9  => CellType::Wall1001,
        10 => CellType::Wall1010,
        11 => CellType::Wall1011,
        12 => CellType::Wall1100,
        13 => CellType::Wall1101,
        14 => CellType::Wall1110,
        15 => CellType::Wall1111,
        _  => CellType::Wall0000,
    }
}

fn is_wall(cell_type: CellType) -> bool {
    match cell_type {
        CellType::GroundNormal     => false,
        CellType::GroundGravel     => false,
        CellType::GroundGrass      => false,
        CellType::GroundWater      => false,
        CellType::GroundMarble     => false,
        CellType::GroundWood       => false,
        CellType::GroundWoodCreaky => false,

                  //  NSEW
        CellType::Wall0000 => true,
        CellType::Wall0001 => true,
        CellType::Wall0010 => true,
        CellType::Wall0011 => true,
        CellType::Wall0100 => true,
        CellType::Wall0101 => true,
        CellType::Wall0110 => true,
        CellType::Wall0111 => true,
        CellType::Wall1000 => true,
        CellType::Wall1001 => true,
        CellType::Wall1010 => true,
        CellType::Wall1011 => true,
        CellType::Wall1100 => true,
        CellType::Wall1101 => true,
        CellType::Wall1110 => true,
        CellType::Wall1111 => true,

        CellType::OneWayWindowE => true,
        CellType::OneWayWindowW => true,
        CellType::OneWayWindowN => true,
        CellType::OneWayWindowS => true,
        CellType::PortcullisNS  => true,
        CellType::PortcullisEW  => true,
        CellType::WindowNS      => true,
        CellType::WindowEW      => true,
        CellType::DoorNS        => true,
        CellType::DoorEW        => true,
    }
}

fn neighboring_walls(map: &CellGrid, x: usize, y: usize) -> u32 {
    let size_x = map.extents()[0];
    let size_y = map.extents()[1];
	let mut wall_bits = 0;

	if y < size_y-1 && is_wall(map[[x, y+1]].cell_type) {
		wall_bits |= 8;
    }
	if y > 0 && is_wall(map[[x, y-1]].cell_type) {
		wall_bits |= 4;
    }
	if x < size_x-1 && is_wall(map[[x+1, y]].cell_type) {
		wall_bits |= 2;
    }
	if x > 0 && is_wall(map[[x-1, y]].cell_type) {
		wall_bits |= 1;
    }

	wall_bits
}
