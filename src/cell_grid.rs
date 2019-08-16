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

pub type CellGrid = multiarray::Array2D<Cell>;
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

pub fn is_guard_at(map: &Map, x: i32, y: i32) -> bool {
    for guard in &map.guards {
        if guard.pos.x == x && guard.pos.y == y {
            return true;
        }
    }
    return false;
}
