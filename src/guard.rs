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

pub fn guard_act_all(rng: &mut MyRng, lines: &mut Lines, map: &mut Map, player: &mut Player) {

    // Mark if we heard a guard last turn, and clear the speaking flag.

    for guard in map.guards.iter_mut() {
        guard.pre_turn();
    }

    // Update each guard for this turn.

    let mut guards = map.guards.split_off(0);

    for mut guard in guards.drain(..) {
        guard.act(rng, lines, player, map);
        map.guards.push(guard);
    }
}

fn pos_next_best(map: &Map, distance_field: &Array2D<usize>, pos_from: Point) -> Point {
    let mut cost_best = INFINITE_COST;
    let mut pos_best = pos_from;

    let pos_min = Point::new(max(0, pos_from.x - 1), max(0, pos_from.y - 1));
    let pos_max = Point::new(min(map.cells.extents()[0] as i32, pos_from.x + 2), min(map.cells.extents()[1] as i32, pos_from.y + 2));

    for x in pos_min.x .. pos_max.x {
        for y in pos_min.y .. pos_max. y {
            let cost = distance_field[[x as usize, y as usize]];
            if cost == INFINITE_COST {
                continue;
            }

            let pos = Point{x, y};
            if map.guard_move_cost(pos_from, pos) == INFINITE_COST {
                continue;
            }

            if map.cells[[pos.x as usize, pos.y as usize]].cell_type == CellType::GroundWater {
                continue;
            }

            if map.pos_blocked_by_guard(pos) {
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

struct LineIter {
    lines: &'static [&'static str],
    line_index: usize,
}

impl LineIter {
    fn new(lines: &'static [&'static str]) -> LineIter {
        LineIter { lines, line_index: 0 }
    }

    fn next(&mut self) -> &'static str {
        let s = self.lines[self.line_index];
        self.line_index = (self.line_index + 1) % self.lines.len();
        s
    }
}

pub struct Lines {
    see: LineIter,
    hear: LineIter,
    hear_guard: LineIter,
    chase: LineIter,
    investigate: LineIter,
    end_chase: LineIter,
    end_investigate: LineIter,
    done_looking: LineIter,
    done_listening: LineIter,
//  damage: LineIter,
}

pub fn new_lines() -> Lines {
    Lines {
        see: LineIter::new(SEE_LINES),
        hear: LineIter::new(HEAR_LINES),
        hear_guard: LineIter::new(HEAR_GUARD_LINES),
        chase: LineIter::new(CHASE_LINES),
        investigate: LineIter::new(INVESTIGATE_LINES),
        end_chase: LineIter::new(END_CHASE_LINES),
        end_investigate: LineIter::new(END_INVESTIGATION_LINES),
        done_looking: LineIter::new(DONE_LOOKING_LINES),
        done_listening: LineIter::new(DONE_LISTENING_LINES),
//      damage: LineIter::new(DAMAGE_LINES),
    }
}

impl Guard {

fn pre_turn(&mut self) {
    self.heard_guard = self.hearing_guard;
    self.hearing_guard = false;
    self.speaking = false;
    self.has_moved = false;
}

pub fn hear_thief(&mut self) {
    self.heard_thief = true;
}

fn act(&mut self, rng: &mut MyRng, lines: &mut Lines, player: &mut Player, map: &Map) {

    let mode_prev = self.mode;
    let pos_prev = self.pos;

    // See if senses will kick us into a new mode

    if self.sees_thief(map, player) {
        self.goal = player.pos;

        if self.mode == GuardMode::Patrol && !self.adjacent_to(player.pos) {
            self.mode = GuardMode::Look;
            self.mode_timeout = rng.gen_range(2, 6);
            self.dir = update_dir(self.dir, player.pos - self.pos);
        } else {
            self.mode = GuardMode::ChaseVisibleTarget;
        }
    } else if self.mode == GuardMode::ChaseVisibleTarget {
        self.mode = GuardMode::MoveToLastSighting;
        self.mode_timeout = 3;
        self.goal = player.pos;
    }

    if self.mode != GuardMode::ChaseVisibleTarget {
        if self.heard_guard {
            self.mode = GuardMode::MoveToGuardShout;
            self.mode_timeout = rng.gen_range(2, 6);
            self.goal = self.heard_guard_pos;
        }

        if self.heard_thief {
            if self.adjacent_to(player.pos) {
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
                    self.say(player, lines.done_looking.next());
                } else if mode_prev == GuardMode::Listen {
                    self.say(player, lines.done_listening.next());
                }
                else if mode_prev == GuardMode::MoveToLastSound || mode_prev == GuardMode::MoveToGuardShout {
                    self.say(player, lines.end_investigate.next());
                }
                else if mode_prev == GuardMode::MoveToLastSighting {
                    self.say(player, lines.end_chase.next());
                }
            },
            GuardMode::Look => {
                self.say(player, lines.see.next());
            },
            GuardMode::Listen => {
                self.say(player, lines.hear.next());
            },
            GuardMode::ChaseVisibleTarget => {
                if mode_prev != GuardMode::MoveToLastSighting {
//                  self.alert_nearby_guards(map);
                    self.say(player, lines.chase.next());
                }
            },
            GuardMode::MoveToLastSighting => {
            },
            GuardMode::MoveToLastSound => {
                self.say(player, lines.investigate.next());
            },
            GuardMode::MoveToGuardShout => {
                self.say(player, lines.hear_guard.next());
            },
        }
    }
}

pub fn overhead_icon(&self, map: &Map, player: &Player) -> Option<usize> {
    if self.mode == GuardMode::Patrol {
        return None;
    }

    let cell = &map.cells[[self.pos.x as usize, self.pos.y as usize]];

    let visible = player.see_all || cell.seen || self.speaking;
    if !visible {
        let dpos = player.pos - self.pos;
        if dpos.length_squared() > 25 {
            return None;
        }
    }

    Some(if self.mode == GuardMode::ChaseVisibleTarget {216} else {215})
}

fn say(&mut self, player: &Player, msg: &str) {
    let d = self.pos - player.pos;
    let dist_squared = d.length_squared();

    if dist_squared < 200 || player.see_all {
        println!("{}", msg);
//      txt::guard_speech(self.pos, msg);
    }

    self.speaking = true;
}

fn adjacent_to(&self, pos: Point) -> bool {
    let d = pos - self.pos;
    d.x.abs() < 2 && d.y.abs() < 2
}

fn sees_thief(&self, map: &Map, player: &Player) -> bool {
    let d = player.pos - self.pos;
    if Point::dot(self.dir, d) < 0 {
        return false;
    }

    let player_is_lit = map.cells[[player.pos.x as usize, player.pos.y as usize]].lit;

    let d2 = d.length_squared();
    if d2 >= self.sight_cutoff(player_is_lit) {
        return false;
    }

    if !player.hidden(map) && line_of_sight(map, self.pos, player.pos) {
        return true;
    }

    if self.mode != GuardMode::Patrol && d.x.abs() < 2 && d.y.abs() < 2 {
        return true;
    }

    return false;
}

fn cutoff_lit(&self) -> i32 {
    if self.mode == GuardMode::Patrol {40} else {75}
}

fn cutoff_unlit(&self) -> i32 {
    if self.mode == GuardMode::Patrol {3} else {33}
}

fn sight_cutoff(&self, lit_target: bool) -> i32 {
    if lit_target {self.cutoff_lit()} else {self.cutoff_unlit()}
}

fn patrol_step(&mut self, map: &Map, player: &mut Player, rng: &mut MyRng) {
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
    }
}

pub fn initial_dir(&self, map: &Map) -> Point
{
    if self.region_goal == INVALID_REGION {
        return self.dir;
    }

    let distance_field = map.compute_distances_to_region(self.region_goal);

    let pos_next = pos_next_best(map, &distance_field, self.pos);

    update_dir(self.dir, pos_next - self.pos)
}

fn move_toward_region(&mut self, map: &Map, player: &Player) -> bool {
    if self.region_goal == INVALID_REGION {
        return false;
    }

    let distance_field = map.compute_distances_to_region(self.region_goal);

    let pos_next = pos_next_best(map, &distance_field, self.pos);

    if player.pos == pos_next {
        return true;
    }

    self.dir = update_dir(self.dir, pos_next - self.pos);
    self.pos = pos_next;

    false
}

fn move_toward_goal(&mut self, map: &Map, player: &Player) -> bool {
    let dist_field = map.compute_distances_to_position(self.goal);

    let pos_next = pos_next_best(map, &dist_field, self.pos);
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

pub fn setup_goal_region(&mut self, rng: &mut MyRng, map: &Map) {
    let region_cur = map.cells[[self.pos.x as usize, self.pos.y as usize]].region;

    if self.region_goal != INVALID_REGION && region_cur == self.region_prev {
        return;
    }

    if region_cur == INVALID_REGION {
        self.region_goal = map.closest_region(&self.pos);
    } else {
        self.region_goal = map.random_neighbor_region(rng, region_cur, self.region_prev);
        self.region_prev = region_cur;
    }
}

}

fn update_dir(dir_forward: Point, dir_aim: Point) -> Point {
    let dir_left = Point::new(-dir_forward.y, dir_forward.x);

    let dot_forward = Point::dot(dir_forward, dir_aim);
    let dot_left = Point::dot(dir_left, dir_aim);

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

fn line_of_sight(map: &Map, from: Point, to: Point) -> bool {
    let mut x = from.x;
    let mut y = from.y;

    let dx = to.x - x;
    let dy = to.y - y;

    let mut ax = dx.abs();
    let mut ay = dy.abs();

    let x_inc = if dx > 0 {1} else {-1};
    let y_inc = if dy > 0 {1} else {-1};

    let mut error = ay - ax;

    let mut n = ax + ay - 1;

    ax *= 2;
    ay *= 2;

    while n > 0 {
        if error > 0 {
            y += y_inc;
            error -= ax;
        } else {
            x += x_inc;
            error += ay;
        }

        if map.blocks_sight(x, y) {
            return false;
        }

        n -= 1;
    }

    true
}

static SEE_LINES: &[&str] = &[
    "Who goes there?",
    "Huh?",
    "What?",
    "Wait...",
    "Who's that?",
    "Hey...",
    "Hmm...",
    "What moved?",
    "Did that shadow move?",
    "I see something...",
    "Hello?",
];

static HEAR_LINES: &[&str] = &[
    "Huh?",
    "What?",
    "Hark!",
    "A noise...",
    "I heard something.",
    "Hmm...",
    "Who goes there?",
    "What's that noise?",
    "I hear something...",
    "Hello?",
];

static HEAR_GUARD_LINES: &[&str] = &[
    "Where?",
    "I'm coming!",
    "Here I come!",
    "To arms!",
    "Where is he?",
];

static CHASE_LINES: &[&str] = &[
    "Halt!",
    "Hey!",
    "Aha!",
    "I see you!",
    "I'm coming!",
    "I'll get you!",
    "Just you wait...",
    "You won't get away!",
    "Oh no you don't...",
    "Get him!",
    "After him!",
    "Thief!",
];

static INVESTIGATE_LINES: &[&str] = &[
    "That noise again...",
    "I heard it again!",
    "Someone's there!",
    "Who could that be?",
    "There it is again!",
    "What was that?",
    "Better check it out...",
    "What keeps making those noises?",
    "That better be rats!",
    "Again?",
];

static END_CHASE_LINES: &[&str] = &[
    "(huff, huff)",
    "Where did he go?",
    "Lost him!",
    "Gone!",
    "Come back!",
    "Argh!",
    "He's not coming back.",
    "Blast!",
    "Next time!",
];

static END_INVESTIGATION_LINES: &[&str] = &[
    "Guess it was nothing.",
    "Wonder what it was?",
    "Better get back.",
    "It's quiet now.",
    "This is where I heard it...",
    "Nothing, now.",
];

static DONE_LOOKING_LINES: &[&str] = &[
    "Must have been rats.",
    "Too much coffee!",
    "I've got the jitters.",
    "Probably nothing.",
    "I thought I saw something.",
    "Oh well.",
    "Nothing.",
    "Can't see it now.",
    "I've been up too long.",
    "Seeing things, I guess.",
    "Hope it wasn't anything.",
    "Did I imagine that?",
];

static DONE_LISTENING_LINES: &[&str] = &[
    "Must have been rats.",
    "Too much coffee!",
    "I've got the jitters.",
    "Probably nothing.",
    "I thought I heard something.",
    "Oh well.",
    "Nothing.",
    "Can't hear it now.",
    "I've been up too long.",
    "Hearing things, I guess.",
    "Hope it wasn't anything.",
    "Did I imagine that?",
];

/*
static DAMAGE_LINES: &[&str] = &[
    "Oof!",
    "Krak!",
    "Pow!",
    "Urk!",
    "Smack!",
    "Bif!",
];
*/
