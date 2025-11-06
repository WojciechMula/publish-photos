use crate::application::Message;
use egui::Event;
use egui::InputState;
use egui::Key;
use egui::Modifiers;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct KeyboardMapping {
    map: HashMap<Key, Vec<(Modifiers, Message)>>,
}

impl KeyboardMapping {
    pub fn key(mut self, key: Key, msg: Message) -> Self {
        self.add(key, Modifiers::NONE, msg);
        self
    }

    pub fn ctrl(mut self, key: Key, msg: Message) -> Self {
        self.add(key, Modifiers::COMMAND.plus(Modifiers::CTRL), msg);
        self
    }

    pub fn add(&mut self, key: Key, modifiers: Modifiers, msg: Message) {
        match self.map.entry(key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push((modifiers, msg));
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![(modifiers, msg)]);
            }
        };
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Key, &Vec<(Modifiers, Message)>)> {
        self.map.iter()
    }

    pub fn lookup_only_combined(&self, input: &mut InputState) -> Option<Message> {
        let key = key_pressed(input)?;
        let list = self.map.get(&key)?;

        for (modifiers, value) in list {
            if !modifiers.is_none() && consume_key(input, *modifiers, key) {
                discard_text_input(input);
                return Some(value.clone());
            }
        }

        None
    }

    pub fn lookup(&self, input: &mut InputState) -> Option<Message> {
        let key = key_pressed(input)?;
        let list = self.map.get(&key)?;

        for (modifiers, value) in list {
            if consume_key(input, *modifiers, key) {
                discard_text_input(input);
                return Some(value.clone());
            }
        }

        None
    }
}

fn consume_key(input: &mut InputState, modifiers: Modifiers, key: Key) -> bool {
    let mut found = false;

    input.events.retain(|event| match event {
        Event::Key {
            key: ev_key,
            modifiers: ev_mod,
            pressed: true,
            ..
        } => {
            if *ev_key == key && ev_mod.matches_exact(modifiers) {
                found = true;
                false
            } else {
                true
            }
        }
        _ => true,
    });

    found
}

fn discard_text_input(input: &mut InputState) {
    input
        .events
        .retain(|event| !matches!(event, Event::Text(_)));
}

fn key_pressed(input: &mut InputState) -> Option<Key> {
    for ev in &input.events {
        if let Event::Key {
            key, pressed: true, ..
        } = ev
        {
            return Some(*key);
        };
    }

    None
}
