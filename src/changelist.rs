#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    Down = 0,
    DownLeft,
    DownRight,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Change {
    Clear,
    Move(usize, Direction),
    Resize(usize, usize),
    Spawn { color: u32, pos: usize },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Changelist {
    pub changes: Vec<u32>,
}

const CLEAR_TAG: u32 = 0b0000;
const RESIZE_TAG: u32 = 0b0001;
const MOVE_TAG: u32 = 0b0010;
const SPAWN_TAG: u32 = 0b0011;

impl Changelist {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    pub fn using_capacity(mut previous: Self) -> Self {
        previous.changes.clear();
        previous
    }

    pub fn clear(&mut self) {
        self.changes.clear();
        self.changes.push(CLEAR_TAG << 28);
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.changes.clear();

        // height must fit in 28 bits
        assert!(
            height < 2usize.pow(28),
            "encoding doesn't support this large of a grid"
        );

        // width must fit in 32 bits
        assert!(
            width < 2usize.pow(32),
            "encoding doesn't support this large of a grid"
        );

        self.changes.push((RESIZE_TAG << 28) | (height as u32));
        self.changes.push(width as u32);
    }

    pub fn move_pixel(&mut self, pos: usize, direction: Direction) {
        assert!(
            // Need 6 more bits
            pos < 2usize.pow(26),
            "encoding doesn't support this large of a grid"
        );
        self.changes
            .push((MOVE_TAG << 28) | ((pos as u32) << 2) | direction as u32);
    }

    pub fn spawn(&mut self, pos: usize, color: u32) {
        assert!(
            // Need 4 more bits
            pos < 2usize.pow(28),
            "encoding doesn't support this large of a grid"
        );
        self.changes.push((SPAWN_TAG << 28) | pos as u32);
        self.changes.push(color);
    }

    pub fn iter(&self) -> impl Iterator<Item = Change> + '_ {
        ChangelistIter {
            changes: &self.changes,
            index: 0,
        }
    }

    #[cfg(test)]
    pub fn from_changes(changes: &[Change]) -> Self {
        let mut changelist = Self::new();
        for change in changes {
            match change {
                Change::Clear => changelist.clear(),
                &Change::Resize(width, height) => changelist.resize(width, height),
                &Change::Move(pos, direction) => changelist.move_pixel(pos, direction),
                &Change::Spawn { color, pos } => changelist.spawn(pos, color),
            }
        }
        changelist
    }
}

struct ChangelistIter<'a> {
    changes: &'a [u32],
    index: usize,
}

impl Iterator for ChangelistIter<'_> {
    type Item = Change;

    fn next(&mut self) -> Option<Self::Item> {
        let change = self.changes.get(self.index)?;
        self.index += 1;
        let type_bits = change >> 28;

        match type_bits {
            CLEAR_TAG => Some(Change::Clear),
            RESIZE_TAG => {
                let height = (change & 0x0FFF_FFFF) as usize;
                let width = self.changes[self.index] as usize;
                self.index += 1;
                Some(Change::Resize(width, height))
            }
            MOVE_TAG => {
                // first remove the tag (<< 4), then remove the direction (>> 6)
                let pos = (change << 4) >> 6;
                let direction = match change & 0b11 {
                    0 => Direction::Down,
                    1 => Direction::DownLeft,
                    2 => Direction::DownRight,
                    d => {
                        panic!("invalid direction: {d:b}");
                    }
                };
                Some(Change::Move(pos as usize, direction))
            }
            SPAWN_TAG => {
                // Remove the tag (<< 4)
                let pos = (change << 4) >> 4;
                let color = self.changes[self.index];
                self.index += 1;
                Some(Change::Spawn {
                    color,
                    pos: pos as usize,
                })
            }
            t => {
                panic!("unknown change tag: {t:b}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn clear() {
        let mut changelist = Changelist::new();
        changelist.clear();
        assert_eq!(changelist.changes, vec![CLEAR_TAG << 28]);

        let mut iter = changelist.iter();
        assert_eq!(iter.next(), Some(Change::Clear));
    }

    #[test]
    pub fn resize() {
        let mut changelist = Changelist::new();
        changelist.resize(10, 20);
        assert_eq!(changelist.changes, vec![(RESIZE_TAG << 28) | 20, 10]);

        let mut iter = changelist.iter();
        assert_eq!(iter.next(), Some(Change::Resize(10, 20)));
    }

    #[test]
    pub fn move_pixel() {
        let mut changelist = Changelist::new();
        changelist.move_pixel(10, Direction::Down);
        assert_eq!(
            changelist.changes,
            vec![(MOVE_TAG << 28) | 10 << 2 | Direction::Down as u32]
        );

        let mut iter = changelist.iter();
        assert_eq!(iter.next(), Some(Change::Move(10, Direction::Down)));
    }

    #[test]
    pub fn spawn() {
        let mut changelist = Changelist::new();
        changelist.spawn(10, 0xFF00FF);
        assert_eq!(changelist.changes, vec![(SPAWN_TAG << 28) | 10, 0xFF00FF]);

        let mut iter = changelist.iter();
        assert_eq!(
            iter.next(),
            Some(Change::Spawn {
                color: 0xFF00FF,
                pos: 10
            })
        );
    }

    #[test]
    pub fn identity() {
        let changes = vec![
            Change::Resize(10, 20),
            Change::Move(10, Direction::Down),
            Change::Spawn {
                color: 0xFF00FF,
                pos: 10,
            },
        ];
        let changelist = Changelist::from_changes(&changes);
        let mut iter = changelist.iter();
        for change in changes {
            assert_eq!(iter.next(), Some(change));
        }
    }
}
