use crate::color_preset;
use multiarray::Array2D;
use rand::Rng;
use std::cmp::max;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub type MyRng = rand_pcg::Pcg32;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum CellType {
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

pub const INVALID_REGION: usize = std::usize::MAX;
pub const INFINITE_COST: usize = std::usize::MAX;

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub cell_type: CellType,
    pub visible: bool,
    pub lit: bool,
    pub seen: bool,
    pub visited: bool,
    pub region: usize,
    pub visit_stamp: usize,
}

pub type CellGrid = Array2D<Cell>;
pub type Point = vector2d::Vector2D<i32>;

pub struct Rect {
    pub pos_min: Point,
    pub pos_max: Point,
}

pub struct Map {
    pub cells: CellGrid,
    pub patrol_regions: Vec<Rect>,
    pub patrol_routes: Vec<(usize, usize)>,
    pub items: Vec<Item>,
    pub guards: Vec<Guard>,
    pub pos_start: Point,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GuardMode
{
    Patrol,
    Look,
    Listen,
    ChaseVisibleTarget,
    MoveToLastSighting,
    MoveToLastSound,
    MoveToGuardShout,
}

pub struct Guard {
	pub pos: Point,
	pub dir: Point,
	pub mode: GuardMode,
	pub speaking: bool,
	pub has_moved: bool,
	pub heard_thief: bool,
	pub hearing_guard: bool,
	pub heard_guard: bool,
	pub heard_guard_pos: Point,

	// Chase
	pub goal: Point,
	pub mode_timeout: usize,

	// Patrol
	pub region_goal: usize,
    pub region_prev: usize,
}

pub struct Item {
    pub pos: Point,
    pub kind: ItemKind,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ItemKind {
    Chair,
    Table,
    Bush,
    Coin,
    DoorNS,
    DoorEW,
    PortcullisNS,
    PortcullisEW,
}

pub struct Player {
    pub pos: Point,
    pub dir: Point,
	pub max_health: usize,
	pub health: usize,
	pub gold: usize,

	pub noisy: bool, // did the player make noise last turn?
	pub damaged_last_turn: bool,
	pub finished_level: bool,

	pub turns_remaining_underwater: usize,

	pub seen: bool,

	pub day: bool,
	pub see_all: bool,
	pub game_over: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub glyph: usize,
    pub color: quicksilver::graphics::Color,
    pub blocks_player: bool,
    pub blocks_sight: bool,
    pub hides_player: bool,
    pub ignores_lighting: bool
}

pub fn tile_def(tile_type: CellType) -> &'static Tile {
    match tile_type {
        CellType::GroundNormal     => &Tile { glyph: 128, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: false, hides_player: false, ignores_lighting: false },
        CellType::GroundGravel     => &Tile { glyph: 130, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: false, hides_player: false, ignores_lighting: false },
        CellType::GroundGrass      => &Tile { glyph: 132, color: color_preset::DARK_GREEN, blocks_player: false, blocks_sight: false, hides_player: false, ignores_lighting: false },
        CellType::GroundWater      => &Tile { glyph: 134, color: color_preset::LIGHT_BLUE, blocks_player: false, blocks_sight: false, hides_player: false, ignores_lighting: false },
        CellType::GroundMarble     => &Tile { glyph: 136, color: color_preset::DARK_CYAN, blocks_player: false, blocks_sight: false, hides_player: false, ignores_lighting: false },
        CellType::GroundWood       => &Tile { glyph: 138, color: color_preset::DARK_BROWN, blocks_player: false, blocks_sight: false, hides_player: false, ignores_lighting: false },
        CellType::GroundWoodCreaky => &Tile { glyph: 138, color: color_preset::DARK_BROWN, blocks_player: false, blocks_sight: false, hides_player: false, ignores_lighting: false },

                  //  NSEW
        CellType::Wall0000 => &Tile { glyph: 176, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall0001 => &Tile { glyph: 177, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall0010 => &Tile { glyph: 177, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall0011 => &Tile { glyph: 177, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall0100 => &Tile { glyph: 178, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall0101 => &Tile { glyph: 179, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall0110 => &Tile { glyph: 182, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall0111 => &Tile { glyph: 185, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall1000 => &Tile { glyph: 178, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall1001 => &Tile { glyph: 180, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall1010 => &Tile { glyph: 181, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall1011 => &Tile { glyph: 184, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall1100 => &Tile { glyph: 178, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall1101 => &Tile { glyph: 186, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall1110 => &Tile { glyph: 183, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::Wall1111 => &Tile { glyph: 187, color: color_preset::LIGHT_GRAY, blocks_player: true, blocks_sight: true, hides_player: false, ignores_lighting: true },

        CellType::OneWayWindowE => &Tile { glyph: 196, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::OneWayWindowW => &Tile { glyph: 197, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::OneWayWindowN => &Tile { glyph: 198, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::OneWayWindowS => &Tile { glyph: 199, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::PortcullisNS  => &Tile { glyph: 128, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::PortcullisEW  => &Tile { glyph: 128, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: true, hides_player: false, ignores_lighting: true },
        CellType::WindowNS      => &Tile { glyph: 189, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: false, hides_player: false, ignores_lighting: true },
        CellType::WindowEW      => &Tile { glyph: 188, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: false, hides_player: false, ignores_lighting: true },
        CellType::DoorNS        => &Tile { glyph: 189, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: false, hides_player: false, ignores_lighting: true },
        CellType::DoorEW        => &Tile { glyph: 188, color: color_preset::LIGHT_GRAY, blocks_player: false, blocks_sight: false, hides_player: false, ignores_lighting: true },
    }
}

pub fn guard_move_cost_for_tile_type(tile_type: CellType) -> usize {
    match tile_type {
        CellType::GroundNormal     => 0,
        CellType::GroundGravel     => 2,
        CellType::GroundGrass      => 0,
        CellType::GroundWater      => 4096,
        CellType::GroundMarble     => 0,
        CellType::GroundWood       => 0,
        CellType::GroundWoodCreaky => 0,
        CellType::Wall0000         => INFINITE_COST,
        CellType::Wall0001         => INFINITE_COST,
        CellType::Wall0010         => INFINITE_COST,
        CellType::Wall0011         => INFINITE_COST,
        CellType::Wall0100         => INFINITE_COST,
        CellType::Wall0101         => INFINITE_COST,
        CellType::Wall0110         => INFINITE_COST,
        CellType::Wall0111         => INFINITE_COST,
        CellType::Wall1000         => INFINITE_COST,
        CellType::Wall1001         => INFINITE_COST,
        CellType::Wall1010         => INFINITE_COST,
        CellType::Wall1011         => INFINITE_COST,
        CellType::Wall1100         => INFINITE_COST,
        CellType::Wall1101         => INFINITE_COST,
        CellType::Wall1110         => INFINITE_COST,
        CellType::Wall1111         => INFINITE_COST,
        CellType::OneWayWindowE    => INFINITE_COST,
        CellType::OneWayWindowW    => INFINITE_COST,
        CellType::OneWayWindowN    => INFINITE_COST,
        CellType::OneWayWindowS    => INFINITE_COST,
        CellType::PortcullisNS     => 0,
        CellType::PortcullisEW     => 0,
        CellType::WindowNS         => 0,
        CellType::WindowEW         => 0,
        CellType::DoorNS           => 0,
        CellType::DoorEW           => 0,
    }
}

pub fn guard_move_cost_for_item_kind(kind: ItemKind) -> usize {
    match kind {
        ItemKind::Chair => 4,
        ItemKind::Table => 10,
        ItemKind::Bush => 10,
        ItemKind::Coin => 0,
        ItemKind::DoorNS => 0,
        ItemKind::DoorEW => 0,
        ItemKind::PortcullisNS => 0,
        ItemKind::PortcullisEW => 0,
    }
}

pub fn make_player(pos: &Point) -> Player {
    let health = 5;
    Player {
        pos: *pos,
        dir: Point::new(0, 0),
        max_health: health,
        health: health,
        gold: 0,
        noisy: false,
        damaged_last_turn: false,
        finished_level: false,
        turns_remaining_underwater: 0,
        seen: false,
        day: false,
        see_all: false,
        game_over: false,
    }
}

impl Player {
    pub fn apply_damage(self: &mut Self, d: usize) {
        if d >= self.health {
            self.health = 0;
            self.game_over = true;
        } else {
            self.health -= d;
        }

        if !self.damaged_last_turn {
            self.damaged_last_turn = true;
//            txt::damage(self.pos, damage_lines.pop_msg());
        }
    }

    pub fn hidden(self: &Self, map: &Map) -> bool {
        if map.hides_player(self.pos.x, self.pos.y) {
            return true;
        }

        let cell_type = map.cells[[self.pos.x as usize, self.pos.y as usize]].cell_type;

        if cell_type == CellType::GroundWater && self.turns_remaining_underwater > 0 {
            return true;
        }

        false
    }
}

const ADJACENT_MOVES: [(usize, Point); 8] = [
    (2, Point { x: 1, y: 0 }),
    (2, Point { x: -1, y: 0 }),
    (2, Point { x: 0, y: 1 }),
    (2, Point { x: 0, y: -1 }),
    (3, Point { x: -1, y: -1 }),
    (3, Point { x: 1, y: -1 }),
    (3, Point { x: -1, y: 1 }),
    (3, Point { x: 1, y: 1 }),
];

impl Map {

pub fn random_neighbor_region(self: &Self, rng: &mut MyRng, region: usize, region_exclude: usize) -> usize {
    let mut neighbors: Vec<usize> = Vec::with_capacity(8);

	for (region0, region1) in &self.patrol_routes {
		if *region0 == region && *region1 != region_exclude {
			neighbors.push(*region1);
        } else if *region1 == region && *region0 != region_exclude {
			neighbors.push(*region0);
        }
	}

	if neighbors.is_empty() {
		return region;
    }

	return neighbors[rng.gen_range(0, neighbors.len())];
}

pub fn guard_cell_cost(self: &Self, x: usize, y: usize) -> usize {
	let mut cost = guard_move_cost_for_tile_type(self.cells[[x, y]].cell_type);
    if cost == INFINITE_COST {
        return INFINITE_COST;
    }

    for item in &self.items {
        if item.pos.x as usize == x && item.pos.y as usize == y {
            cost = max(cost, guard_move_cost_for_item_kind(item.kind));
        }
    }

	cost
}

pub fn guard_move_cost(self: &Self, pos_old: Point, pos_new: Point) -> usize {
	let cost = self.guard_cell_cost(pos_new.x as usize, pos_new.y as usize);

	if cost == INFINITE_COST {
		return cost;
    }

    // Guards are not allowed to move diagonally around corners.

	if pos_old.x != pos_new.x &&
        pos_old.y != pos_new.y &&
		(self.guard_cell_cost(pos_old.x as usize, pos_new.y as usize) == INFINITE_COST ||
		self.guard_cell_cost(pos_new.x as usize, pos_old.y as usize) == INFINITE_COST) {
		return INFINITE_COST;
	}

	cost
}

pub fn pos_blocked_by_guard(self: &Self, pos: Point) -> bool {
	for guard in &self.guards {
		if guard.pos == pos {
			return true;
		}
	}

	false
}

pub fn closest_region(self: &Self, pos: &Point) -> usize {

    #[derive(Copy, Clone, Eq, PartialEq)]
    struct State {
        dist: usize,
        pos: Point,
    }

    impl Ord for State {
        fn cmp(&self, other: &State) -> Ordering {
            other.dist.cmp(&self.dist)
        }
    }

    impl PartialOrd for State {
        fn partial_cmp(&self, other: &State) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut heap = BinaryHeap::with_capacity(self.cells.extents()[0] * self.cells.extents()[1]);
    let mut dist_field: Array2D<usize> = Array2D::new([self.cells.extents()[0], self.cells.extents()[1]], INFINITE_COST);

    heap.push(State{dist: 0, pos: *pos});

    let size_x = self.cells.extents()[0] as i32;
    let size_y = self.cells.extents()[1] as i32;

    while let Some(State {dist, pos}) = heap.pop() {
        let p = [pos.x as usize, pos.y as usize];

        if self.cells[p].region != INVALID_REGION {
            return self.cells[p].region;
        }

        if dist >= dist_field[p] {
            continue;
        }

        dist_field[p] = dist;

        for (move_dir_cost, dir) in &ADJACENT_MOVES {
            let pos_new = pos + *dir;
            if pos_new.x < 0 || pos_new.y < 0 || pos_new.x >= size_x || pos_new.y >= size_y {
                continue;
            }

            let move_cost = self.guard_move_cost(pos, pos_new);
            if move_cost == INFINITE_COST {
                continue;
            }

            let dist_new = dist + move_cost + move_dir_cost;

            if dist_new < dist_field[[pos_new.x as usize, pos_new.y as usize]] {
                heap.push(State{dist: dist_new, pos: pos_new});
            }
        }
    }

    INVALID_REGION
}

pub fn compute_distances_to_region(self: &Self, i_region_goal: usize) -> Array2D<usize> {
	assert!(i_region_goal < self.patrol_regions.len());

    let region = &self.patrol_regions[i_region_goal];

	// Fill the priority queue with all of the region's locations.

    let mut goal = Vec::with_capacity(((region.pos_max.x - region.pos_min.x) * (region.pos_max.y - region.pos_min.y)) as usize);

    for x in region.pos_min.x .. region.pos_max.x {
        for y in region.pos_min.y .. region.pos_max.y {
            let p = Point{x, y};
            goal.push((self.guard_cell_cost(x as usize, y as usize), p));
        }
    }

    self.compute_distance_field(&goal)
}

pub fn compute_distances_to_position(self: &Self, pos_goal: Point) -> Array2D<usize> {
	assert!(pos_goal.x >= 0);
	assert!(pos_goal.y >= 0);
	assert!(pos_goal.x < self.cells.extents()[0] as i32);
	assert!(pos_goal.y < self.cells.extents()[1] as i32);

    self.compute_distance_field(&[(0, pos_goal)])
}

pub fn compute_distance_field(self: &Self, initial_distances: &[(usize, Point)]) -> Array2D<usize> {

    #[derive(Copy, Clone, Eq, PartialEq)]
    struct State {
        dist: usize,
        pos: Point,
    }

    impl Ord for State {
        fn cmp(&self, other: &State) -> Ordering {
            other.dist.cmp(&self.dist)
        }
    }

    impl PartialOrd for State {
        fn partial_cmp(&self, other: &State) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut heap = BinaryHeap::with_capacity(self.cells.extents()[0] * self.cells.extents()[1]);
    let mut dist_field: Array2D<usize> = Array2D::new([self.cells.extents()[0], self.cells.extents()[1]], INFINITE_COST);

    let size_x = self.cells.extents()[0] as i32;
    let size_y = self.cells.extents()[1] as i32;

    for (dist, pos) in initial_distances {
        heap.push(State{dist: *dist, pos: *pos});
    }

    while let Some(State {dist, pos}) = heap.pop() {
        let p = [pos.x as usize, pos.y as usize];
        if dist >= dist_field[p] {
            continue;
        }

        dist_field[p] = dist;

        for (move_dir_cost, dir) in &ADJACENT_MOVES {
            let pos_new = pos + *dir;
            if pos_new.x < 0 || pos_new.y < 0 || pos_new.x >= size_x || pos_new.y >= size_y {
                continue;
            }

            let move_cost = self.guard_move_cost(pos, pos_new);
            if move_cost == INFINITE_COST {
                continue;
            }

            let dist_new = dist + move_cost + move_dir_cost;
            let p_new = [pos_new.x as usize, pos_new.y as usize];
            if dist_new < dist_field[p_new] {
                heap.push(State{dist: dist_new, pos: pos_new});
            }
        }
    }

    dist_field
}

pub fn blocks_sight(self: &Self, x: i32, y: i32) -> bool {
    let cell_type = self.cells[[x as usize, y as usize]].cell_type;
    let tile = tile_def(cell_type);
    if tile.blocks_sight {
        return true;
    }

/*
	const Cell & cell = at(x, y);

	for (const Item * item = cell.items; item; item = item->next())
	{
		if (item->cell_info().blocks_sight)
			return true;
	}
*/

	false
}

pub fn hides_player(self: &Self, x: i32, y: i32) -> bool {
    let cell_type = self.cells[[x as usize, y as usize]].cell_type;
    let tile = tile_def(cell_type);
    if tile.hides_player {
        return true;
    }

/*
	const Cell & cell = at(x, y);

	for (const Item * item = cell.items; item; item = item->next())
	{
		if (item->cell_info().hides_player)
			return true;
	}
*/

	false
}

}
