use eframe::emath::OrderedFloat;
use egui::Context;
use egui::SizeHint;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct ImageCache {
    pub max_size: usize,
    pub loaded: HashMap<String, u64>,
    pub requested: HashSet<String>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            max_size: 300,
            loaded: HashMap::new(),
            requested: HashSet::new(),
        }
    }

    pub fn is_cached(&self, uri: &String) -> bool {
        self.loaded.contains_key(uri)
    }

    pub fn request(&mut self, uri: String, ctx: &Context) {
        if let Some(time) = self.loaded.get_mut(&uri) {
            *time = ctx.cumulative_frame_nr();
        } else {
            self.requested.insert(uri);
        }
    }

    pub fn load_requested(&mut self, ctx: &Context) {
        if self.requested.is_empty() {
            return;
        }

        let loaded = self.loaded.len();
        let requested = self.requested.len();
        if loaded + requested > self.max_size {
            let to_discard = (loaded + requested) - self.max_size;

            let mut oldest = KBottom::new(to_discard);
            for (uri, time) in &self.loaded {
                oldest.add(uri, time);
            }

            for BinaryHeapEntry { uri, .. } in oldest.binheap {
                self.loaded.remove(&uri);
                ctx.forget_image(&uri);
            }
        }

        let time = ctx.cumulative_frame_nr();

        for uri in self.requested.drain() {
            let _ = ctx.try_load_image(&uri, SizeHint::Scale(OrderedFloat(1.0)));
            self.loaded.insert(uri, time);
        }
    }
}

struct BinaryHeapEntry {
    uri: String,
    time: u64,
}

impl Ord for BinaryHeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl PartialOrd for BinaryHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for BinaryHeapEntry {}

impl PartialEq for BinaryHeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

struct KBottom {
    binheap: BinaryHeap<BinaryHeapEntry>,
}

impl KBottom {
    fn new(cap: usize) -> Self {
        Self {
            binheap: BinaryHeap::<BinaryHeapEntry>::with_capacity(cap),
        }
    }

    fn add(&mut self, uri: &String, time: &u64) {
        if self.binheap.len() < self.binheap.capacity() {
            self.binheap.push(BinaryHeapEntry {
                uri: uri.clone(),
                time: *time,
            });
            return;
        }

        if let Some(top) = self.binheap.peek() {
            if *time <= top.time {
                self.binheap.pop();
                self.binheap.push(BinaryHeapEntry {
                    uri: uri.clone(),
                    time: *time,
                });
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn as_string(kb: KBottom) -> String {
        let mut v = kb
            .binheap
            .iter()
            .map(|entry| entry.uri.clone())
            .collect::<Vec<_>>();
        v.sort();

        v.join(" ")
    }

    #[test]
    fn kbottom_case1() {
        let mut oldest = KBottom::new(3);

        let mut add = |time: u64| oldest.add(&time.to_string(), &time);

        for i in [42, 5, 12] {
            add(i);
        }

        assert_eq!(as_string(oldest), "12 42 5")
    }

    #[test]
    fn kbottom_case2() {
        let mut oldest = KBottom::new(3);

        let mut add = |time: u64| oldest.add(&time.to_string(), &time);

        for i in [10, 1, 15, 7, 100, 4] {
            add(i);
        }

        assert_eq!(as_string(oldest), "1 4 7")
    }

    #[test]
    fn kbottom_case3() {
        let mut oldest = KBottom::new(3);

        let mut add = |time: u64| oldest.add(&time.to_string(), &time);

        for i in 1..=10 {
            add(i);
        }

        assert_eq!(as_string(oldest), "1 2 3")
    }
}
