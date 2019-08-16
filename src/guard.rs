use crate::cell_grid::*;

pub fn is_guard_at(map: &Map, x: i32, y: i32) -> bool {
    for guard in &map.guards {
        if guard.pos.x == x && guard.pos.y == y {
            return true;
        }
    }
    return false;
}

/*

pub fn guard_act_all(map: &mut Map) {

	// Mark if we heard a guard last turn, and clear the speaking flag.

    for guard in map.guards.iter_mut() {
		guard.heard_guard = guard.hearing_guard;
		guard.hearing_guard = false;
		guard.speaking = false;
		guard.hasMoved = false;
	}

	// Update each guard for this turn.

	for guard in map.guards.iter_mut() {
		guard_act(map, guard);
		guard.hasMoved = true;
	}
}

fn guard_act(map: &Map, guard: &Guard) {
	Mode modePrev = m_mode;
	Point2 posPrev = m_pos;

	// See if senses will kick us into a new mode

	if sees_thief(guard, map) {
		g_player.m_seen = true;
		guard.goal = g_player.m_pos;

		if (m_mode == patrol && !adjacent_to_thief())
		{
			m_mode = look;
			m_mode_timeout = 2 + random(4);
			m_dir = updateDir(m_dir, g_player.m_pos - m_pos);
		}
		else
		{
			m_mode = chaseVisibleTarget;
		}
	}
	else
	{
		if (m_mode == chaseVisibleTarget)
		{
			m_mode = moveToLastSighting;
			m_mode_timeout = 3;
			m_goal = g_player.m_pos;
		}
	}

	if (m_mode != chaseVisibleTarget)
	{
		if (heard_guard())
		{
			m_mode = moveToGuardShout;
			m_mode_timeout = 2 + random(4);
			m_goal = m_heard_guard_pos;
		}

		if (heard_thief())
		{
			if (adjacent_to_thief())
			{
				g_player.m_seen = true;
				m_mode = chaseVisibleTarget;
				m_goal = g_player.m_pos;
			}
			else if (m_mode == patrol)
			{
				m_mode = listen;
				m_mode_timeout = 2 + random(4);
				m_dir = updateDir(m_dir, g_player.m_pos - m_pos);
			}
			else
			{
				m_mode = moveToLastSound;
				m_mode_timeout = 2 + random(4);
				m_goal = g_player.m_pos;
			}
		}
	}

	// Pass time in the current mode

    match guard.mode {
	    GuardMode::Patrol => {
		    patrolStep(map, guard);
        },
        GuardMode::Look | GuardMode::Listen => {
            guard.mode_timeout -= 1;
            if guard.mode_timeout < 0 {
                guard.mode = GuardMode::Patrol;
            }
        },
        GuardMode::ChaseVisibleTarget => {
            if (adjacent_to_thief())
            {
                m_dir = updateDir(m_dir, m_goal - m_pos);
                if (modePrev == chaseVisibleTarget)
                {
                    g_player.apply_damage(1);
                }
            }
            else
            {
                move_toward_goal(map);
            }
        },
        GuardMode::MoveToLastSighting | GuardMode::MoveToLastSound | GuardMode::MoveToGuardShout => {
            if (!move_toward_goal(map))
                --m_mode_timeout;

            if (m_mode_timeout < 0)
            {
                m_mode = patrol;
                setupGoalRegion(map);
            }
        },

	case fixAmissObject:

		if (!move_toward_goal(map))
		{
			Point2 dpos = m_goal - m_pos;

			// Problem: should move to any square adjacent to usable object, not to object itself.
			// Problem: should move to controller for controlled things rather than the thing itself.
			// Problem: should recognize a lamp is out based on any squares it should light being dark.

			if (abs(dpos[0]) < 2 && abs(dpos[1]) < 2)
			{
				if (fix(map, m_goal))
				{
					say(fixed_lines.pop_msg());
				}

				m_mode_timeout = 0;
			}
			else
			{
				--m_mode_timeout;
			}
		}

		if (m_mode_timeout < 0)
		{
			m_mode = patrol;
			setupGoalRegion(map);
		}
		break;

	default:
		break;
	}

	// If we moved, update state based on target visibility from new position

	if (m_pos != posPrev)
	{
		if (sees_thief(map))
		{
			g_player.m_seen = true;
			m_goal = g_player.m_pos;

			if (m_mode == patrol && !adjacent_to_thief())
			{
				m_mode = look;
				m_mode_timeout = 2 + random(4);
			}
			else
			{
				m_mode = chaseVisibleTarget;
			}

			m_dir = updateDir(m_dir, g_player.m_pos - m_pos);
		}
		else if (m_mode == chaseVisibleTarget)
		{
			m_mode = moveToLastSighting;
			m_mode_timeout = 3;
			m_goal = g_player.m_pos;
		}
	}

	// Clear heard-thief flag

	m_heard_thief = false;

	// Say something to indicate state changes

	if (modePrev != m_mode)
	{
		switch (m_mode)
		{
		case patrol:
			if (modePrev == look)
			{
				say(done_looking_lines.pop_msg());
			}
			else if (modePrev == listen)
			{
				say(done_listening_lines.pop_msg());
			}
			else if (modePrev == moveToLastSound || modePrev == moveToGuardShout)
			{
				say(end_investigation_lines.pop_msg());
			}
			else if (modePrev == moveToLastSighting)
			{
				say(end_search_lines.pop_msg());
			}
			break;

		case look:
			say(see_lines.pop_msg());
			break;

		case listen:
			say(hear_lines.pop_msg());
			break;

		case chaseVisibleTarget:
			if (modePrev != moveToLastSighting)
			{
				alert_nearby_guards(map);
				say(chase_lines.pop_msg());
			}
			break;

		case moveToLastSighting:
			break;

		case moveToLastSound:
			say(investigate_lines.pop_msg());
			break;

		case moveToGuardShout:
			say(hear_guard_lines.pop_msg());
			break;

		case fixAmissObject:
			break;
		}
	}
}

*/
