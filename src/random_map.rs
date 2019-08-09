use crate::cell_grid::*;

pub fn generate_map() -> CellGrid {
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
    map
}
