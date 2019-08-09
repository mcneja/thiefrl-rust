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
