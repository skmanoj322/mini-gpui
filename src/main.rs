// task here is to build app which take any value and return  the value inserted by key here
//
//
//

use std::{
    any::Any,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::Deref,
};

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
struct EntityId(usize);

struct App {
    entries: HashMap<EntityId, Box<dyn Any>>,
    next_id: usize,
    being_updates: HashSet<EntityId>,
}

struct Entity<T> {
    id: EntityId,
    _marker: PhantomData<T>,
}
impl App {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            next_id: 0,
            being_updates: HashSet::new(),
        }
    }

    fn insert<T: 'static>(&mut self, value: T) -> Entity<T> {
        self.next_id += 1;
        let value = Box::new(value);
        // have to derive in the EntityId here because of the
        self.entries.insert(EntityId(self.next_id), value);

        Entity {
            id: EntityId(self.next_id),
            _marker: PhantomData,
        }
    }

    fn read<T: 'static>(&self, entity: &Entity<T>) -> Option<&T> {
        self.entries.get(&entity.id)?.downcast_ref()
    }

    fn update<T: 'static>(&mut self, entity: &Entity<T>, f: impl FnOnce(&mut T, &mut App)) -> bool {
        match self
            .entries
            .remove(&entity.id)
            .and_then(|b| b.downcast::<T>().ok())
        {
            Some(mut value) => {
                f(&mut value, self);
                self.entries.insert(entity.id, value);
                return true;
            }
            None => {
                return false;
            }
        };
    }
}
struct Counter {
    count: i32,
}

fn main() {
    let mut app = App::new();
    let entity = app.insert(Counter { count: 5 });
    let counter = app.read(&entity);

    println!("{:?}", counter.map(|counter| counter.count)); // should print Some(5)

    let entity_copy = Entity {
        id: entity.id,
        _marker: PhantomData,
    };

    app.update(&entity, |counter, app| {
        counter.count += 1;
        let inner = app.update(&entity_copy, |counter: &mut Counter, _app| {
            counter.count += 100;
        });
        println!("inner update worked? {inner}");
    });
}
