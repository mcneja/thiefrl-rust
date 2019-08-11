use crate::cell_grid::*;
use rand::prelude::*;
use std::cmp::min;
use std::cmp::max;
use std::mem::swap;
use multiarray::Array2D;

const OUTER_BORDER: i32 = 3;

const ROOM_SIZE_X: i32 = 5;
const ROOM_SIZE_Y: i32 = 5;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum RoomType
{
	Exterior,
	Courtyard,
	Interior,
	MasterSuite,
}

struct Room
{
	pub room_type: RoomType,
	pub group: usize,
	pub depth: usize,
	pub pos_min: Point,
	pub pos_max: Point,
	pub edges: Vec<usize>,
}

struct Adjacency
{
	pub origin: Point,
	pub dir: Point,
	pub length: i32,
	pub room_left: usize,
	pub room_right: usize,
	pub next_matching: usize,
	pub door: bool,
}

pub fn generate_map(seed: u64) -> (CellGrid, Point) {
    let mut rng = rand_pcg::Pcg32::seed_from_u64(seed);

    let (map, pos_start) = generate_siheyuan(4, &mut rng);

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

    (map, pos_start)
}

fn generate_siheyuan(level: i32, mut rng: &mut impl Rng) -> (CellGrid, Point) {
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

	// Create exits connecting rooms.

	let (rooms, adjacencies, pos_start) = create_exits(
        &mut rng,
		level,
		mirror_x,
		mirror_y,
		&inside,
		&offset_x,
		&offset_y,
		&mut map);

    (map, pos_start)

    /*
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

fn create_exits(
    mut rng: &mut impl Rng,
	level: i32,
	mirror_x: bool,
	mirror_y: bool,
	inside: &Array2D<bool>,
	offset_x: &Array2D<i32>,
	offset_y: &Array2D<i32>,
	mut map: &mut CellGrid
) -> (Vec<Room>, Vec<Adjacency>, Point) {
	// Make a set of rooms.

	let rooms_x: usize = inside.extents()[0];
	let rooms_y: usize = inside.extents()[1];

	let mut room_index: Array2D<usize> = Array2D::new([rooms_x, rooms_y], 0);
    let mut rooms: Vec<Room> = Vec::new();

	// This room represents the area surrounding the map.

    rooms.push(
        Room {
            room_type: RoomType::Exterior,
            group: 0,
            depth: 0,
            pos_min: Point::new(0, 0), // not meaningful for this room
            pos_max: Point::new(0, 0), // not meaningful for this room
            edges: Vec::new(),
        }
    );

	for rx in 0..rooms_x {
		for ry in 0..rooms_y {
			let group_index = rooms.len();

			room_index[[rx, ry]] = group_index;

            rooms.push(
                Room {
                    room_type: if inside[[rx, ry]] {RoomType::Interior} else {RoomType::Courtyard},
                    group: group_index,
                    depth: 0,
                    pos_min: Point::new(offset_x[[rx, ry]] + 1, offset_y[[rx, ry]] + 1),
                    pos_max: Point::new(offset_x[[rx + 1, ry]], offset_y[[rx, ry + 1]]),
                    edges: Vec::new(),
                }
            );
		}
	}

	// Compute a list of room adjacencies.

	let mut adjacencies = compute_adjacencies(mirror_x, mirror_y, &inside, &offset_x, &offset_y, &room_index);
	store_adjacencies_in_rooms(&adjacencies, &mut rooms);

	// Connect rooms together.

	let pos_start = connect_rooms(&mut rng, &mut rooms, &mut adjacencies);

	// Assign types to the rooms.

	assign_room_types(&room_index, &adjacencies, &mut rooms);

    /*

	// Generate pathing information.

	generatePatrolRoutes(rooms, adjacencies, map);
    */

	// Render doors and windows.

	render_walls(&mut rng, &rooms, &adjacencies, &mut map);

	// Render floors.

	render_rooms(level, &rooms, &mut map, &mut rng);

    (rooms, adjacencies, pos_start)
}

fn compute_adjacencies
(
	mirror_x: bool,
	mirror_y: bool,
	inside: &Array2D<bool>,
	offset_x: &Array2D<i32>,
	offset_y: &Array2D<i32>,
	room_index: &Array2D<usize>
) -> Vec<Adjacency> {

	let rooms_x = inside.extents()[0];
	let rooms_y = inside.extents()[1];

    let mut adjacencies: Vec<Adjacency> = Vec::new();

    {
        let mut adjacency_rows: Vec<Vec<usize>> = Vec::with_capacity(rooms_y + 1);

        {
            let mut adjacency_row: Vec<usize> = Vec::with_capacity(rooms_x);

            let ry = 0;

            for rx in 0..rooms_x {
                let x0 = offset_x[[rx, ry]];
                let x1 = offset_x[[rx+1, ry]];
                let y = offset_y[[rx, ry]];

                let i = adjacencies.len();
                adjacency_row.push(i);

                adjacencies.push(
                    Adjacency {
                        origin: Point::new(x0 + 1, y),
                        dir: Point::new(1, 0),
                        length: x1 - (x0 + 1),
                        room_left: room_index[[rx, ry]],
                        room_right: 0,
                        next_matching: i,
                        door: false,
                    }
                );
            }

            adjacency_rows.push(adjacency_row);
        }

        for ry in 1..rooms_y {
            let mut adjacency_row: Vec<usize> = Vec::with_capacity(3 * rooms_x);

            for rx in 0..rooms_x {
                let x0_upper = offset_x[[rx, ry]];
                let x0_lower = offset_x[[rx, ry-1]];
                let x1_upper = offset_x[[rx+1, ry]];
                let x1_lower = offset_x[[rx+1, ry-1]];
                let x0 = max(x0_lower, x0_upper);
                let x1 = min(x1_lower, x1_upper);
                let y = offset_y[[rx, ry]];

                if rx > 0 && x0_lower - x0_upper > 1 {
                    let i = adjacencies.len();
                    adjacency_row.push(i);

                    adjacencies.push(
                        Adjacency {
                            origin: Point::new(x0_upper + 1, y),
                            dir: Point::new(1, 0),
                            length: x0_lower - (x0_upper + 1),
                            room_left: room_index[[rx, ry]],
                            room_right: room_index[[rx - 1, ry - 1]],
                            next_matching: i,
                            door: false,
                        }
                    );
                }

                if x1 - x0 > 1 {
                    let i = adjacencies.len();
                    adjacency_row.push(i);

                    adjacencies.push(
                        Adjacency {
                            origin: Point::new(x0 + 1, y),
                            dir: Point::new(1, 0),
                            length: x1 - (x0 + 1),
                            room_left: room_index[[rx, ry]],
                            room_right: room_index[[rx, ry - 1]],
                            next_matching: i,
                            door: false,
                        }
                    );
                }

                if rx + 1 < rooms_x && x1_upper - x1_lower > 1 {
                    let i = adjacencies.len();
                    adjacency_row.push(i);

                    adjacencies.push(
                        Adjacency {
                            origin: Point::new(x1_lower + 1, y),
                            dir: Point::new(1, 0),
                            length: x1_upper - (x1_lower + 1),
                            room_left: room_index[[rx, ry]],
                            room_right: room_index[[rx + 1, ry - 1]],
                            next_matching: i,
                            door: false,
                        }
                    );
                }
            }

            adjacency_rows.push(adjacency_row);
        }

        {
            let mut adjacency_row: Vec<usize> = Vec::with_capacity(rooms_x);

            let ry = rooms_y;

            for rx in 0..rooms_x {
                let x0 = offset_x[[rx, ry-1]];
                let x1 = offset_x[[rx+1, ry-1]];
                let y = offset_y[[rx, ry]];

                let i = adjacencies.len();
                adjacency_row.push(i);

                adjacencies.push(
                    Adjacency {
                        origin: Point::new(x0 + 1, y),
                        dir: Point::new(1, 0),
                        length: x1 - (x0 + 1),
                        room_left: 0,
                        room_right: room_index[[rx, ry - 1]],
                        next_matching: i,
                        door: false,
                    }
                );
            }

            adjacency_rows.push(adjacency_row);
        }

        if mirror_x {
            for ry in 0..adjacency_rows.len() {
                let row = &adjacency_rows[ry];

                let mut i = 0;
                let mut j = row.len() - 1;
                while i < j {
                    let adj0 = row[i];
                    let adj1 = row[j];

                    adjacencies[adj0].next_matching = adj1;
                    adjacencies[adj1].next_matching = adj0;

                    // Flip edge a1 to point the opposite direction
                    {
                        let a1 = &mut adjacencies[adj1];
                        a1.origin += a1.dir * (a1.length - 1);
                        a1.dir = -a1.dir;
                        swap(&mut a1.room_left, &mut a1.room_right);
                    }

                    i += 1;
                    j -= 1;
                }
            }
        }

        if mirror_y {
            let mut ry0 = 0;
            let mut ry1 = adjacency_rows.len() - 1;
            while ry0 < ry1 {
                let row0 = &adjacency_rows[ry0];
                let row1 = &adjacency_rows[ry1];

                assert!(row0.len() == row1.len());

                for i in 0..row0.len() {
                    let adj0 = row0[i];
                    let adj1 = row1[i];
                    adjacencies[adj0].next_matching = adj1;
                    adjacencies[adj1].next_matching = adj0;
                }

                ry0 += 1;
                ry1 -= 1;
            }
        }
    }

    {
        let mut adjacency_rows: Vec<Vec<usize>> = Vec::with_capacity(rooms_x + 1);

        {
            let mut adjacency_row: Vec<usize> = Vec::with_capacity(rooms_y);

            let rx = 0;

            for ry in 0..rooms_y {
                let y0 = offset_y[[rx, ry]];
                let y1 = offset_y[[rx, ry+1]];
                let x = offset_x[[rx, ry]];

                let i = adjacencies.len();
                adjacency_row.push(i);

                adjacencies.push(
                    Adjacency {
                        origin: Point::new(x, y0 + 1),
                        dir: Point::new(0, 1),
                        length: y1 - (y0 + 1),
                        room_left: 0,
                        room_right: room_index[[rx, ry]],
                        next_matching: i,
                        door: false,
                    }
                );
            }

            adjacency_rows.push(adjacency_row);
        }

        for rx in 1..rooms_x {
            let mut adjacency_row: Vec<usize> = Vec::with_capacity(3 * rooms_y);

            for ry in 0..rooms_y {
                let y0_left  = offset_y[[rx-1, ry]];
                let y0_right = offset_y[[rx, ry]];
                let y1_left  = offset_y[[rx-1, ry+1]];
                let y1_right = offset_y[[rx, ry+1]];
                let y0 = max(y0_left, y0_right);
                let y1 = min(y1_left, y1_right);
                let x = offset_x[[rx, ry]];

                if ry > 0 && y0_left - y0_right > 1 {
                    let i = adjacencies.len();
                    adjacency_row.push(i);

                    adjacencies.push(
                        Adjacency {
                            origin: Point::new(x, y0_right + 1),
                            dir: Point::new(0, 1),
                            length: y0_left - (y0_right + 1),
                            room_left: room_index[[rx - 1, ry - 1]],
                            room_right: room_index[[rx, ry]],
                            next_matching: i,
                            door: false,
                        }
                    );
                }

                if y1 - y0 > 1 {
                    let i = adjacencies.len();
                    adjacency_row.push(i);

                    adjacencies.push(
                        Adjacency {
                            origin: Point::new(x, y0 + 1),
                            dir: Point::new(0, 1),
                            length: y1 - (y0 + 1),
                            room_left: room_index[[rx - 1, ry]],
                            room_right: room_index[[rx, ry]],
                            next_matching: i,
                            door: false,
                        }
                    );
                }

                if ry + 1 < rooms_y && y1_right - y1_left > 1 {
                    let i = adjacencies.len();
                    adjacency_row.push(i);

                    adjacencies.push(
                        Adjacency {
                            origin: Point::new(x, y1_left + 1),
                            dir: Point::new(0, 1),
                            length: y1_right - (y1_left + 1),
                            room_left: room_index[[rx - 1, ry + 1]],
                            room_right: room_index[[rx, ry]],
                            next_matching: i,
                            door: false,
                        }
                    );
                }
            }

            adjacency_rows.push(adjacency_row);
        }

        {
            let mut adjacency_row: Vec<usize> = Vec::with_capacity(rooms_y);

            let rx = rooms_x;

            for ry in 0..rooms_y {
                let y0 = offset_y[[rx-1, ry]];
                let y1 = offset_y[[rx-1, ry+1]];
                let x = offset_x[[rx, ry]];

                let i = adjacencies.len();
                adjacencies.push(
                    Adjacency {
                        origin: Point::new(x, y0 + 1),
                        dir: Point::new(0, 1),
                        length: y1 - (y0 + 1),
                        room_left: room_index[[rx - 1, ry]],
                        room_right: 0,
                        next_matching: i,
                        door: false,
                    }
                );
                adjacency_row.push(i);
            }

            adjacency_rows.push(adjacency_row);
        }

        if mirror_y {
            for ry in 0..adjacency_rows.len() {
                let row = &adjacency_rows[ry];
                let n = row.len() / 2;

                for i in 0..n {
                    let adj0 = row[i];
                    let adj1 = row[(row.len() - 1) - i];

                    adjacencies[adj0].next_matching = adj1;
                    adjacencies[adj1].next_matching = adj0;

                    {
                        // Flip edge a1 to point the opposite direction
                        let a1 = &mut adjacencies[adj1];
                        a1.origin += a1.dir * (a1.length - 1);
                        a1.dir = -a1.dir;
                        swap(&mut a1.room_left, &mut a1.room_right);
                    }
                }
            }
        }

        if mirror_x {
            let mut ry0 = 0;
            let mut ry1 = adjacency_rows.len() - 1;
            while ry0 < ry1 {
                let row0 = &adjacency_rows[ry0];
                let row1 = &adjacency_rows[ry1];

                assert!(row0.len() == row1.len());

                for i in 0..row0.len() {
                    let adj0 = row0[i];
                    let adj1 = row1[i];
                    adjacencies[adj0].next_matching = adj1;
                    adjacencies[adj1].next_matching = adj0;
                }

                ry0 += 1;
                ry1 -= 1;
            }
        }
    }

    adjacencies
}

fn store_adjacencies_in_rooms(adjacencies: &Vec<Adjacency>, rooms: &mut Vec<Room>) {
    for (i, adj) in adjacencies.iter().enumerate() {
		let i0 = adj.room_left;
		let i1 = adj.room_right;
		rooms[i0].edges.push(i);
		rooms[i1].edges.push(i);
	}
}

fn get_edge_sets(mut rng: &mut impl Rng, adjacencies: &[Adjacency]) -> Vec<Vec<usize>> {
    let mut edge_sets = Vec::with_capacity(adjacencies.len());

    for (i, adj) in adjacencies.iter().enumerate() {
        let j = adj.next_matching;
        if j >= i {
            if j > i {
                edge_sets.push(vec![i, j]);
            } else {
                edge_sets.push(vec![i]);
            }
        }
    }

    edge_sets.shuffle(&mut rng);

    edge_sets
}

fn connect_rooms(mut rng: &mut impl Rng, mut rooms: &mut Vec<Room>, adjacencies: &mut Vec<Adjacency>) -> Point {

    // Collect sets of edges that are mirrors of each other

    let edge_sets = get_edge_sets(&mut rng, &adjacencies);

	// Connect all adjacent courtyard rooms together.

    for adj in adjacencies.iter_mut() {
		let i0 = adj.room_left;
		let i1 = adj.room_right;
		if rooms[i0].room_type != RoomType::Courtyard || rooms[i1].room_type != RoomType::Courtyard {
            continue;
        }

        adj.door = true;
        let group0 = rooms[i0].group;
        let group1 = rooms[i1].group;
        join_groups(&mut rooms, group0, group1);
	}

	// Connect all the interior rooms with doors.

    for edge_set in &edge_sets {

        let mut added_door = false;

        {
            let adj = &mut adjacencies[edge_set[0]];

            let i0 = adj.room_left;
            let i1 = adj.room_right;

            if rooms[i0].room_type != RoomType::Interior || rooms[i1].room_type != RoomType::Interior {
                continue;
            }

            let group0 = rooms[i0].group;
            let group1 = rooms[i1].group;

            if group0 != group1 || rng.gen_range(0, 3) == 0 {
                adj.door = true;
                added_door = true;
                join_groups(&mut rooms, group0, group1);
            }
        }

        if added_door {
            for i in 1..edge_set.len() {
                let adj = &mut adjacencies[edge_set[i]];

                let i0 = adj.room_left;
                let i1 = adj.room_right;

                let group0 = rooms[i0].group;
                let group1 = rooms[i1].group;

                adj.door = true;
                join_groups(&mut rooms, group0, group1);
            }
        }
    }

	// Create doors between the interiors and the courtyard areas.

    for edge_set in &edge_sets {

        let mut added_door = false;

        {
            let adj = &mut adjacencies[edge_set[0]];

            let i0 = adj.room_left;
            let i1 = adj.room_right;

            let room_type0 = rooms[i0].room_type;
            let room_type1 = rooms[i1].room_type;

            if room_type0 == room_type1 {
                continue;
            }

            if room_type0 == RoomType::Exterior || room_type1 == RoomType::Exterior {
                continue;
            }

            let group0 = rooms[i0].group;
            let group1 = rooms[i1].group;

            if group0 != group1 || rng.gen_range(0, 3) == 0 {
                adj.door = true;
                added_door = true;
                join_groups(&mut rooms, group0, group1);
            }
        }

        if added_door {
            for i in 1..edge_set.len() {
                let adj = &mut adjacencies[edge_set[i]];

                let i0 = adj.room_left;
                let i1 = adj.room_right;

                let group0 = rooms[i0].group;
                let group1 = rooms[i1].group;

                adj.door = true;
                join_groups(&mut rooms, group0, group1);
            }
        }
    }

	// Create the door to the surrounding exterior. It must be on the south side.

    let mut pos_start = Point::new(0, 0);

    /*
    for (i, adj) in adjacencies.iter_mut().enumerate() {

		if adj.dir.x == 0 {
			continue;
        }

		if adj.next_matching > i {
			continue;
        }

		if adj.next_matching == i {
			if rooms[adj.room_right].room_type != RoomType::Exterior {
				continue;
            }
		} else {
			if rooms[adj.room_left].room_type != RoomType::Exterior {
				continue;
            }
		}

		// Set the player's start position based on where the door is.

		pos_start.x = adj.origin.x + adj.dir.x * (adj.length / 2);
		pos_start.y = OUTER_BORDER - 1;

		adj.door = true;

		// Break symmetry if the door is off center.

		if adj.next_matching != i {
			adjacencies[adj.next_matching].next_matching = adj.next_matching;
			adj.next_matching = i;
		}

		break;
	}
    */

    pos_start
}

fn join_groups(rooms: &mut Vec<Room>, group_from: usize, group_to: usize) {
	if group_from != group_to {
        for room in rooms.iter_mut() {
            if room.group == group_from {
                room.group = group_to;
            }
        }
    }
}

fn assign_room_types(room_index: &Array2D<usize>, adjacencies: &Vec<Adjacency>, rooms: &mut Vec<Room>) {

	// Assign rooms depth based on distance from the bottom row of rooms.

	let unvisited = rooms.len();

	rooms[0].depth = 0;

    for i in 1..rooms.len() {
        rooms[i].depth = unvisited;
    }

    let mut rooms_to_visit: Vec<usize> = Vec::with_capacity(rooms.len());

    for x in 0..room_index.extents()[0] {
        let i_room = room_index[[x, 0]];
        rooms[i_room].depth = 1;
        rooms_to_visit.push(i_room);
    }

	// Visit rooms in breadth-first order, assigning them distances from the seed rooms.

    let mut ii_room = 0;
    while ii_room < rooms_to_visit.len() {
		let i_room = rooms_to_visit[ii_room];

        for i_adj in &rooms[i_room].edges.clone() {
			let adj: &Adjacency = &adjacencies[*i_adj];

			if !adj.door {
				continue;
            }

			let i_room_neighbor = if adj.room_left == i_room {adj.room_right} else {adj.room_left};

			if rooms[i_room_neighbor].depth == unvisited {
				rooms[i_room_neighbor].depth = rooms[i_room].depth + 1;
				rooms_to_visit.push(i_room_neighbor);
			}
        }

        ii_room += 1;
    }

	// Assign master-suite room type to the inner rooms.

	let mut max_depth = 0;
	for room in rooms.iter() {
		max_depth = max(max_depth, room.depth);
	}

	let target_num_master_rooms = (room_index.extents()[0] * room_index.extents()[1]) / 4;

	let mut num_master_rooms = 0;

    let mut depth = max_depth;
    while depth > 0 {
		for room in rooms.iter_mut() {
			if room.room_type != RoomType::Interior {
				continue;
            }

			if room.depth != depth {
				continue;
            }

			room.room_type = RoomType::MasterSuite;
			num_master_rooms += 1;
		}

		if num_master_rooms >= target_num_master_rooms {
			break;
        }

        depth -= 1;
	}
}

const ONE_WAY_WINDOW: [CellType; 5] = [
	CellType::OneWayWindowS,
	CellType::OneWayWindowE,
	CellType::OneWayWindowE, // not used
	CellType::OneWayWindowW,
	CellType::OneWayWindowN,
];

fn render_walls(rng: &mut impl Rng, rooms: &Vec<Room>, adjacencies: &Vec<Adjacency>, map: &mut CellGrid) {

	// Render grass connecting courtyard rooms.

    for adj in adjacencies.iter() {
		let type0 = rooms[adj.room_left].room_type;
		let type1 = rooms[adj.room_right].room_type;

		if type0 != RoomType::Courtyard || type1 != RoomType::Courtyard {
			continue;
        }

        for j in 0..adj.length {
			let p: Point = adj.origin + adj.dir * j;
			map[[p.x as usize, p.y as usize]].cell_type = CellType::GroundGrass;
		}
    }

	// Render doors and windows for the rest of the walls.

    for i in 0..adjacencies.len() {
		let adj0 = &adjacencies[i];

		let type0 = rooms[adj0.room_left].room_type;
		let type1 = rooms[adj0.room_right].room_type;

		if type0 == RoomType::Courtyard && type1 == RoomType::Courtyard {
			continue;
        }

		let j = adj0.next_matching;

		if j < i {
			continue;
        }

		let offset =
            if j == i {
                adj0.length / 2
            } else if adj0.length > 2 {
                1 + rng.gen_range(0, adj0.length - 2)
            } else {
                rng.gen_range(0, adj0.length)
            };

        let mut walls: Vec<&Adjacency> = Vec::with_capacity(2);
        walls.push(adj0);

        if j != i {
            walls.push(&adjacencies[j]);
        }

		if !adj0.door && type0 != type1 {
			if type0 == RoomType::Exterior || type1 == RoomType::Exterior {
				if (adj0.length & 1) != 0 {
					let k = adj0.length / 2;

                    for a in &walls {
						let p = a.origin + a.dir * k;

                        let dir =
                            if rooms[a.room_right].room_type == RoomType::Exterior {
                                -a.dir
                            } else {
                                a.dir
                            };

						map[[p.x as usize, p.y as usize]].cell_type = ONE_WAY_WINDOW[(2 * dir.x + dir.y + 2) as usize];
					}
				}
			} else if type0 == RoomType::Courtyard || type1 == RoomType::Courtyard {
                let mut k = rng.gen_range(0, 2);
				let k_end = (adj0.length + 1) / 2;

                while k < k_end {
					for a in &walls {
						let dir =
                            if rooms[a.room_right].room_type == RoomType::Courtyard {
                                -a.dir
                            } else {
                                a.dir
                            };

						let window_type = ONE_WAY_WINDOW[(2 * dir.x + dir.y + 2) as usize];

						let p: Point = a.origin + a.dir * k;
						let q: Point = a.origin + a.dir * (a.length - (k + 1));

						map[[p.x as usize, p.y as usize]].cell_type = window_type;
						map[[q.x as usize, q.y as usize]].cell_type = window_type;
					}
                    k += 2;
				}
			}
		}

		let install_master_suite_door = rng.gen_range(0, 100) < 40;

		for a in &walls {
			if !a.door {
				continue;
            }

			let p = a.origin + a.dir * offset;

			let orient_ns = a.dir.x == 0;

			map[[p.x as usize, p.y as usize]].cell_type = if orient_ns {CellType::DoorNS} else {CellType::DoorEW};

			let room_type_left = rooms[a.room_left].room_type;
			let room_type_right = rooms[a.room_right].room_type;

			if room_type_left == RoomType::Exterior || room_type_right == RoomType::Exterior {
                map[[p.x as usize, p.y as usize]].cell_type = if orient_ns {CellType::PortcullisNS} else {CellType::PortcullisEW};
//				place_portcullis(map, p, orient_ns);
            } else if room_type_left != RoomType::MasterSuite || room_type_right != RoomType::MasterSuite || install_master_suite_door {
                map[[p.x as usize, p.y as usize]].cell_type = if orient_ns {CellType::DoorNS} else {CellType::DoorEW};
//				place_door(map, p, orient_ns);
            }
		}
	}
}

fn render_rooms(level: i32, rooms: &Vec<Room>, map: &mut CellGrid, rng: &mut impl Rng) {
    for i_room in 1..rooms.len() {
		let room = &rooms[i_room];

		let cell_type =
            if room.room_type == RoomType::Courtyard {
                CellType::GroundGrass
            } else if room.room_type == RoomType::MasterSuite {
                CellType::GroundMarble
            } else {
                CellType::GroundWood
            };

		for x in room.pos_min.x..room.pos_max.x {
			for y in room.pos_min.y..room.pos_max.y {
				let t =
                    if cell_type == CellType::GroundWood && level > 3 && rng.gen_range(0, 50) == 0 {
                        CellType::GroundWoodCreaky
                    } else {
                        cell_type
                    };

				map[[x as usize, y as usize]].cell_type = t;
			}
		}

		let dx = room.pos_max.x - room.pos_min.x;
		let dy = room.pos_max.y - room.pos_min.y;

		if room.room_type == RoomType::Courtyard {
			if dx >= 5 && dy >= 5 {
				for x in room.pos_min.x + 1 .. room.pos_max.x - 1 {
					for y in room.pos_min.y + 1 .. room.pos_max.y - 1 {
						map[[x as usize, y as usize]].cell_type = CellType::GroundWater;
					}
				}
			} else if dx >= 2 && dy >= 2 {
				try_place_bush(map, room.pos_min.x, room.pos_min.y);
				try_place_bush(map, room.pos_max.x - 1, room.pos_min.y);
				try_place_bush(map, room.pos_min.x, room.pos_max.y - 1);
				try_place_bush(map, room.pos_max.x - 1, room.pos_max.y - 1);
			}
		} else if room.room_type == RoomType::Interior || room.room_type == RoomType::MasterSuite {
			if dx >= 5 && dy >= 5 {
				if room.room_type == RoomType::MasterSuite {
					for x in 2..dx-2 {
						for y in 2..dy-2 {
							map[[(room.pos_min.x + x) as usize, (room.pos_min.y + y) as usize]].cell_type = CellType::GroundWater;
						}
					}
				}

				map[[(room.pos_min.x + 1) as usize, (room.pos_min.y + 1) as usize]].cell_type = CellType::Wall0000;
				map[[(room.pos_max.x - 2) as usize, (room.pos_min.y + 1) as usize]].cell_type = CellType::Wall0000;
				map[[(room.pos_min.x + 1) as usize, (room.pos_max.y - 2) as usize]].cell_type = CellType::Wall0000;
				map[[(room.pos_max.x - 2) as usize, (room.pos_max.y - 2) as usize]].cell_type = CellType::Wall0000;
			} else if dx == 5 && dy >= 3 && (room.room_type == RoomType::Interior || rng.gen_range(0, 3) == 0) {
				for y in 1..dy-1 {
					place_chair(map, room.pos_min.x + 1, room.pos_min.y + y);
					place_table(map, room.pos_min.x + 2, room.pos_min.y + y);
					place_chair(map, room.pos_min.x + 3, room.pos_min.y + y);
				}
			} else if dy == 5 && dx >= 3 && (room.room_type == RoomType::Interior || rng.gen_range(0, 3) == 0) {
				for x in 1..dx-1 {
					place_chair(map, room.pos_min.x + x, room.pos_min.y + 1);
					place_table(map, room.pos_min.x + x, room.pos_min.y + 2);
					place_chair(map, room.pos_min.x + x, room.pos_min.y + 3);
				}
			} else if dx > dy && (dy & 1) == 1 && rng.gen_range(0, 3) != 0 {
				let y = room.pos_min.y + dy / 2;

				if room.room_type == RoomType::Interior {
					try_place_table(map, room.pos_min.x + 1, y);
					try_place_table(map, room.pos_max.x - 2, y);
				} else {
					try_place_chair(map, room.pos_min.x + 1, y);
					try_place_chair(map, room.pos_max.x - 2, y);
				}
			} else if dy > dx && (dx & 1) == 1 && rng.gen_range(0, 3) != 0 {
				let x = room.pos_min.x + dx / 2;

				if room.room_type == RoomType::Interior {
					try_place_table(map, x, room.pos_min.y + 1);
					try_place_table(map, x, room.pos_max.y - 2);
				} else {
					try_place_chair(map, x, room.pos_min.y + 1);
					try_place_chair(map, x, room.pos_max.y - 2);
				}
			} else if dx > 3 && dy > 3 {
				if room.room_type == RoomType::Interior {
					try_place_table(map, room.pos_min.x, room.pos_min.y);
					try_place_table(map, room.pos_max.x - 1, room.pos_min.y);
					try_place_table(map, room.pos_min.x, room.pos_max.y - 1);
					try_place_table(map, room.pos_max.x - 1, room.pos_max.y - 1);
				} else {
					try_place_chair(map, room.pos_min.x, room.pos_min.y);
					try_place_chair(map, room.pos_max.x - 1, room.pos_min.y);
					try_place_chair(map, room.pos_min.x, room.pos_max.y - 1);
					try_place_chair(map, room.pos_max.x - 1, room.pos_max.y - 1);
				}
			}
		}
	}
}

fn door_adjacent(map: &CellGrid, x: i32, y: i32) -> bool {
	if map[[(x - 1) as usize, y as usize]].cell_type >= CellType::PortcullisNS {
		return true;
    }

	if map[[(x + 1) as usize, y as usize]].cell_type >= CellType::PortcullisNS {
		return true;
    }

	if map[[x as usize, (y - 1) as usize]].cell_type >= CellType::PortcullisNS {
		return true;
    }

	if map[[x as usize, (y + 1) as usize]].cell_type >= CellType::PortcullisNS {
		return true;
    }

    false
}

fn try_place_bush(map: &CellGrid, x: i32, y: i32) {
	if map[[x as usize, y as usize]].cell_type != CellType::GroundGrass {
		return;
    }

    if door_adjacent(&map, x, y) {
        return;
    }

	place_bush(&map, x, y);
}

fn try_place_table(map: &CellGrid, x: i32, y: i32) {
    if door_adjacent(&map, x, y) {
        return;
    }

	place_table(&map, x, y);
}

fn try_place_chair(map: &CellGrid, x: i32, y: i32) {
    if door_adjacent(&map, x, y) {
        return;
    }

	place_chair(&map, x, y);
}

fn place_chair(map: &CellGrid, x: i32, y: i32) {
}

fn place_table(map: &CellGrid, x: i32, y: i32) {
}

fn place_bush(map: &CellGrid, x: i32, y: i32) {
}
