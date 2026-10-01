#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjectId(pub usize);

/// A release is a click only if the pointer never left the drag threshold.
#[derive(Default)]
pub struct PointerGesture {
    pressed_at: Option<(f32, f32)>,
    pub dragging: bool,
}

impl PointerGesture {
    pub fn press(&mut self, position: Option<(f32, f32)>) {
        self.pressed_at = position;
        self.dragging = false;
    }

    pub fn update(&mut self, position: Option<(f32, f32)>) {
        if let (Some((x, y)), Some((sx, sy))) = (position, self.pressed_at) {
            self.dragging |= (x - sx).powi(2) + (y - sy).powi(2) > 25.0;
        } else {
            self.dragging = true;
        }
    }

    pub fn release(&mut self, position: Option<(f32, f32)>) -> Option<(f32, f32)> {
        self.update(position);
        let click = self.pressed_at.take().and(position).filter(|_| !self.dragging);
        self.dragging = false;
        click
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractiveKind {
    Clue(usize),
    BackDoor,
    Movable,
}

#[derive(Debug, Default)]
pub struct GameState {
    pub clues_found: [bool; 3],
    pub door_unlocked: bool,
    pub secret_room_open: bool,
    pub selected_object: Option<ObjectId>,
}

impl GameState {
    pub fn found_clue(&mut self, clue_index: usize) -> bool {
        let Some(found) = self.clues_found.get_mut(clue_index) else {
            return false;
        };
        let newly_found = !*found;
        *found = true;
        self.door_unlocked = self.clues_found.iter().all(|found| *found);
        newly_found
    }

    pub fn clues_count(&self) -> usize {
        self.clues_found.iter().filter(|found| **found).count()
    }
}

#[cfg(test)]
mod tests {
    use super::GameState;

    #[test]
    fn door_unlocks_after_all_three_clues() {
        let mut state = GameState::default();
        assert!(!state.door_unlocked);

        assert!(state.found_clue(0));
        assert!(state.found_clue(1));
        assert!(!state.door_unlocked);
        assert!(state.found_clue(2));
        assert!(state.door_unlocked);
        assert_eq!(state.clues_count(), 3);
        assert!(!state.found_clue(2));
    }

    #[test]
    fn invalid_clue_index_does_not_change_progress() {
        let mut state = GameState::default();
        assert!(!state.found_clue(3));
        assert_eq!(state.clues_count(), 0);
        assert!(!state.door_unlocked);
    }
}
