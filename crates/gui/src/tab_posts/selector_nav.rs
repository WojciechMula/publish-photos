use db::Selector;
use db::SelectorLevel;

pub struct SelectorNavigation<'a> {
    current: Selector,
    selectors: &'a [Selector],
}

impl<'a> SelectorNavigation<'a> {
    pub fn new(current: Selector, selectors: &'a [Selector]) -> Self {
        Self { current, selectors }
    }

    pub fn first(&self) -> Option<Selector> {
        self.selectors.first().copied()
    }

    pub fn last(&self) -> Option<Selector> {
        self.selectors.last().copied()
    }

    pub fn next(&self) -> Option<Selector> {
        let pos = self.position()?;
        let pos = pos + 1;

        self.selectors.get(pos).copied()
    }

    pub fn prev(&self) -> Option<Selector> {
        let pos = self.position()?;
        let pos = pos.checked_sub(1)?;

        self.selectors.get(pos).copied()
    }

    pub fn next_same_level(&self) -> Option<Selector> {
        let pos = self.position()?;

        let tail = &self.selectors[pos + 1..];
        tail.iter()
            .find(|selector| selector.level() == self.current.level())
            .copied()
    }

    pub fn prev_same_level(&self) -> Option<Selector> {
        let pos = self.position()?;

        let head = &self.selectors[..pos];
        head.iter()
            .rev()
            .find(|selector| selector.level() == self.current.level())
            .copied()
    }

    pub fn parent(&self) -> Option<Selector> {
        let level = self.current.level().prev()?;

        self.current.trim_to(level)
    }

    pub fn first_child_selector(&self) -> Option<Selector> {
        let level = self.current.level().next()?;
        let pos = self.position()?;

        let tail = &self.selectors[pos + 1..];
        tail.iter()
            .find(|selector| selector.level() == level)
            .copied()
    }

    pub fn first_same_level(&self) -> Option<Selector> {
        if self.current.level() == SelectorLevel::Year {
            return self.first();
        }

        let pos = self.position()?;
        let head = &self.selectors[..pos];

        let mut result = self.current;
        for selector in head.iter().rev() {
            if selector.level() == result.level() {
                result = *selector;
            } else if selector.level() > result.level() {
                return Some(result);
            }
        }

        None
    }

    pub fn last_same_level(&self) -> Option<Selector> {
        if self.current.level() == SelectorLevel::Year {
            return self.last();
        }

        let pos = self.position()?;
        let tail = &self.selectors[pos + 1..];

        let mut result = self.current;
        for selector in tail.iter() {
            if selector.level() == result.level() {
                result = *selector;
            } else if selector.level() > result.level() {
                return Some(result);
            }
        }

        None
    }

    fn position(&self) -> Option<usize> {
        self.selectors
            .iter()
            .position(|selector| *selector == self.current)
    }
}
