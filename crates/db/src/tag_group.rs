use crate::TagList;
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Serialize, Deserialize)]
pub struct TagGroupList(Vec<TagGroup>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TagGroupId(usize);

impl TagGroupList {
    pub fn add(&mut self, mut group: TagGroup) -> Result<(), String> {
        let max_id = self
            .0
            .iter()
            .map(|group| group.id.0)
            .max()
            .unwrap_or_default();
        group.id = TagGroupId(max_id + 1);

        self.0.push(group);

        Ok(())
    }

    pub fn move_up(&mut self, id: &TagGroupId) -> bool {
        let Some(index) = self.0.iter().position(|group| group.id == *id) else {
            return false;
        };

        if index == 0 {
            return false;
        }

        self.0.swap(index, index - 1);

        true
    }

    pub fn move_down(&mut self, id: &TagGroupId) -> bool {
        let Some(index) = self.0.iter().position(|group| group.id == *id) else {
            return false;
        };

        if index + 1 >= self.len() {
            return false;
        }

        self.0.swap(index, index + 1);

        true
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|group| group.name == name)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &TagGroup> {
        self.0.iter()
    }

    pub fn get(&self, id: &TagGroupId) -> Option<&TagGroup> {
        self.0.iter().find(|group| group.id == *id)
    }

    pub fn get_mut(&mut self, id: &TagGroupId) -> Option<&mut TagGroup> {
        self.0.iter_mut().find(|group| group.id == *id)
    }
}

#[derive(Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagGroup {
    pub id: TagGroupId,
    pub name: String,
    pub tags: TagList,
}

impl TagGroup {
    pub fn update(&mut self, other: Self) -> bool {
        assert_eq!(self.id, other.id);

        if self.name != other.name || self.tags != other.tags {
            self.name = other.name;
            self.tags = other.tags;

            true
        } else {
            false
        }
    }
}
