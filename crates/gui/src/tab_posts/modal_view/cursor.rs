use std::cmp::min;

#[derive(Default, Clone)]
pub struct Cursor {
    size: usize,
    cursor: usize,
}

impl Cursor {
    pub fn new(size: usize) -> Self {
        Self { size, cursor: 0 }
    }

    pub fn set_current(&mut self, cursor: usize) {
        if self.is_empty() {
            return;
        }

        self.cursor = min(cursor, self.size - 1);
    }

    pub fn first(&mut self) {
        self.cursor = 0;
    }

    pub fn last(&mut self) {
        if self.is_empty() {
            return;
        }

        self.cursor = self.size - 1;
    }

    pub fn next(&mut self) {
        if self.is_empty() {
            return;
        }

        self.cursor = (self.cursor + 1) % self.size;
    }

    pub fn prev(&mut self) {
        if self.is_empty() {
            return;
        }

        if self.cursor > 0 {
            self.cursor -= 1;
        } else {
            self.cursor = self.size - 1;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn current(&self) -> Option<usize> {
        if self.cursor < self.size {
            Some(self.cursor)
        } else {
            None
        }
    }
}
