use crate::cell_grid::*;
use rand::prelude::*;
use std::cmp::min;
use std::cmp::max;
use multiarray::Array2D;

pub fn is_guard_at(map: &Map, x: i32, y: i32) -> bool {
    for guard in &map.guards {
        if guard.pos.x == x && guard.pos.y == y {
            return true;
        }
    }
    return false;
}

pub fn guard_act_all(rng: &mut MyRng, map: &mut Map, player: &mut Player) {

	// Mark if we heard a guard last turn, and clear the speaking flag.

    for guard in map.guards.iter_mut() {
        guard.pre_turn();
	}

	// Update each guard for this turn.

    let mut guards = map.guards.split_off(map.guards.len());

    for mut guard in guards.drain(..) {
		guard.act(rng, player, map);
        map.guards.push(guard);
	}
}

fn pos_next_best(map: &Map, distance_field: &Array2D<usize>, pos_from: &Point) -> Point {
	let mut cost_best = INFINITE_COST;
	let mut pos_best = *pos_from;

	let pos_min = Point::new(max(0, pos_from.x - 1), max(0, pos_from.y - 1));
	let pos_max = Point::new(min(map.cells.extents()[0] as i32, pos_from.x + 2), min(map.cells.extents()[1] as i32, pos_from.y + 2));

	for x in pos_min.x .. pos_max.x {
		for y in pos_min.y .. pos_max. y {
			let cost = distance_field[[x as usize, y as usize]];
			if cost == INFINITE_COST {
				continue;
			}

			let pos = Point::new(x, y);
			if map.pos_blocked_by_guard(&pos) {
				continue;
			}

			if cost < cost_best {
				cost_best = cost;
				pos_best = pos;
			}
		}
	}

	pos_best
}

impl Guard {

fn pre_turn(self: &mut Self) {
    self.heard_guard = self.hearing_guard;
    self.hearing_guard = false;
    self.speaking = false;
    self.has_moved = false;
}

fn act(self: &mut Self, rng: &mut MyRng, player: &mut Player, map: &Map) {

	let mode_prev = self.mode;
	let pos_prev = self.pos;

	// See if senses will kick us into a new mode

	if self.sees_thief(map, player) {
		player.seen = true;
		self.goal = player.pos;

		if self.mode == GuardMode::Patrol && !self.adjacent_to(player.pos) {
			self.mode = GuardMode::Look;
			self.mode_timeout = rng.gen_range(2, 6);
			self.dir = update_dir(self.dir, player.pos - self.pos);
		} else {
			self.mode = GuardMode::ChaseVisibleTarget;
		}
	} else {
		if self.mode == GuardMode::ChaseVisibleTarget {
			self.mode = GuardMode::MoveToLastSighting;
			self.mode_timeout = 3;
			self.goal = player.pos;
		}
	}

	if self.mode != GuardMode::ChaseVisibleTarget {
		if self.heard_guard {
			self.mode = GuardMode::MoveToGuardShout;
			self.mode_timeout = rng.gen_range(2, 6);
			self.goal = self.heard_guard_pos;
		}

		if self.heard_thief {
			if self.adjacent_to(player.pos) {
				player.seen = true;
				self.mode = GuardMode::ChaseVisibleTarget;
				self.goal = player.pos;
			} else if self.mode == GuardMode::Patrol {
				self.mode = GuardMode::Listen;
				self.mode_timeout = rng.gen_range(2, 6);
				self.dir = update_dir(self.dir, player.pos - self.pos);
			} else {
				self.mode = GuardMode::MoveToLastSound;
				self.mode_timeout = rng.gen_range(2, 6);
				self.goal = player.pos;
			}
		}
	}

	// Pass time in the current mode

    match self.mode {
	    GuardMode::Patrol => {
		    self.patrol_step(map, player, rng);
        },
        GuardMode::Look |
        GuardMode::Listen => {
            self.mode_timeout -= 1;
            if self.mode_timeout == 0 {
                self.mode = GuardMode::Patrol;
            }
        },
        GuardMode::ChaseVisibleTarget => {
            if self.adjacent_to(player.pos) {
                self.dir = update_dir(self.dir, self.goal - self.pos);
                if mode_prev == GuardMode::ChaseVisibleTarget {
                    player.apply_damage(1);
                }
            } else {
                self.move_toward_goal(map, player);
            }
        },
        GuardMode::MoveToLastSighting |
        GuardMode::MoveToLastSound |
        GuardMode::MoveToGuardShout => {
            if !self.move_toward_goal(map, player) {
                self.mode_timeout -= 1;
            }

            if self.mode_timeout == 0 {
                self.mode = GuardMode::Patrol;
                self.setup_goal_region(rng, map);
            }
        },
	}

	// If we moved, update state based on target visibility from new position

	if self.pos != pos_prev {
		if self.sees_thief(map, player) {
			player.seen = true;
			self.goal = player.pos;

			if self.mode == GuardMode::Patrol && !self.adjacent_to(player.pos) {
				self.mode = GuardMode::Look;
				self.mode_timeout = rng.gen_range(2, 6);
			} else {
				self.mode = GuardMode::ChaseVisibleTarget;
			}

			self.dir = update_dir(self.dir, player.pos - self.pos);
		} else if self.mode == GuardMode::ChaseVisibleTarget {
			self.mode = GuardMode::MoveToLastSighting;
			self.mode_timeout = 3;
			self.goal = player.pos;
		}
	}

	// Clear heard-thief flag

	self.heard_thief = false;

	// Say something to indicate state changes

	if mode_prev != self.mode {
		match self.mode {
			GuardMode::Patrol => {
				if mode_prev == GuardMode::Look {
//					say(done_looking_lines.pop_msg());
				} else if mode_prev == GuardMode::Listen {
//					say(done_listening_lines.pop_msg());
				}
				else if mode_prev == GuardMode::MoveToLastSound || mode_prev == GuardMode::MoveToGuardShout {
//					say(end_investigation_lines.pop_msg());
				}
				else if mode_prev == GuardMode::MoveToLastSighting {
//					say(end_search_lines.pop_msg());
				}
			},
			GuardMode::Look => {
//				say(see_lines.pop_msg());
			},
			GuardMode::Listen => {
//				say(hear_lines.pop_msg());
			},
			GuardMode::ChaseVisibleTarget => {
				if mode_prev != GuardMode::MoveToLastSighting {
//					alert_nearby_guards(map);
//					say(chase_lines.pop_msg());
				}
			},
			GuardMode::MoveToLastSighting => {
			},
			GuardMode::MoveToLastSound => {
//				say(investigate_lines.pop_msg());
			},
			GuardMode::MoveToGuardShout => {
//				say(hear_guard_lines.pop_msg());
			},
		}
	}
}

fn adjacent_to(self: &Self, pos: Point) -> bool {
	let d = pos - self.pos;
	d.x.abs() < 2 && d.y.abs() < 2
}

fn sees_thief(self: &Self, map: &Map, player: &Player) -> bool {
	let d = player.pos - self.pos;
	if vector2d::Vector2D::dot(self.dir, d) < 0 {
		return false;
    }

/*
	let d2 = vector2d::Vector2D::dot(d, d);
	if d2 >= sightCutoff(player.lit(map)) {
		return false;
    }

	if !player.hidden(map) && line_of_sight(map, self.pos, player.pos) {
		return true;
    }
*/

	if self.mode != GuardMode::Patrol && d.x.abs() < 2 && d.y.abs() < 2 {
		return true;
    }

	return false;
}

fn patrol_step(self: &mut Self, map: &Map, player: &mut Player, rng: &mut MyRng) {
	let bumped_thief = self.move_toward_region(map, player);

	if map.cells[[self.pos.x as usize, self.pos.y as usize]].region == self.region_goal {
		let region_prev = self.region_prev;
		self.region_prev = self.region_goal;
		self.region_goal = map.random_neighbor_region(rng, self.region_goal, region_prev);
	}

	if bumped_thief {
		self.mode = GuardMode::ChaseVisibleTarget;
		self.goal = player.pos;
		self.dir = update_dir(self.dir, self.goal - self.pos);
		player.seen = true;
	}
}

pub fn move_toward_region(self: &mut Self, map: &Map, player: &Player) -> bool {
	if self.region_goal == INVALID_REGION {
		return false;
	}

	let distance_field = map.compute_distances_to_region(self.region_goal);

	let pos_next = pos_next_best(map, &distance_field, &self.pos);

	if player.pos == pos_next {
		return true;
	}

	self.dir = update_dir(self.dir, pos_next - self.pos);
	self.pos = pos_next;

	false
}

fn move_toward_goal(self: &mut Self, map: &Map, player: &Player) -> bool {
	let dist_field = map.compute_distances_to_position(&self.goal);

	let pos_next = pos_next_best(map, &dist_field, &self.pos);
	if pos_next == self.pos {
		return false;
    }

	self.dir = update_dir(self.dir, pos_next - self.pos);

	if player.pos == pos_next {
		return false;
	}

	self.pos = pos_next;
	true
}

fn setup_goal_region(self: &mut Self, rng: &mut MyRng, map: &Map) {
	let i_region_cur = map.cells[[self.pos.x as usize, self.pos.y as usize]].region;

	if self.region_goal != INVALID_REGION && i_region_cur == self.region_prev {
		return;
	}

	if i_region_cur != INVALID_REGION {
		self.region_goal = map.random_neighbor_region(rng, i_region_cur, self.region_prev);
		self.region_prev = i_region_cur;
	} else {
		self.region_goal = map.closest_region(&self.pos);
	}
}

}

fn update_dir(dir_forward: Point, dir_aim: Point) -> Point {
	let dir_left = Point::new(-dir_forward.y, dir_forward.x);

	let dot_forward = vector2d::Vector2D::dot(dir_forward, dir_aim);
	let dot_left = vector2d::Vector2D::dot(dir_left, dir_aim);

	if dot_forward.abs() > dot_left.abs() {
		if dot_forward >= 0 {dir_forward} else {-dir_forward}
	} else if dot_left.abs() > dot_forward.abs() {
		if dot_left >= 0 {dir_left} else {-dir_left}
	} else if dot_forward > 0 {
		dir_forward
	} else {
		if dot_left >= 0 {dir_left} else {-dir_left}
	}
}