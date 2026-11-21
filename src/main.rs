// task here is to build app which take any value and return  the value inserted by key here
//
//
//

use std::{
    any::Any,
    collections::{HashMap, HashSet, hash_map},
    marker::PhantomData,
    ops::Deref,
};

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
struct EntityId(usize);

struct App {
    entries: HashMap<EntityId, Box<dyn Any>>,
    next_id: usize,
    being_updated: HashSet<EntityId>,
    observers: HashMap<EntityId, Vec<Box<dyn FnMut(&mut App)>>>,
    pending_notifications: Vec<EntityId>,
    update_depth: usize,
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
            being_updated: HashSet::new(),
            observers: HashMap::new(),
            pending_notifications: Vec::new(),
            update_depth: 0,
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
        if self.being_updated.contains(&entity.id) {
            panic!("cannot update an entity while it is already being updated");
        }

        self.being_updated.insert(entity.id);
        self.update_depth += 1;

        let updated = match self
            .entries
            .remove(&entity.id)
            .and_then(|b| b.downcast::<T>().ok())
        {
            Some(mut value) => {
                f(&mut value, self);
                self.entries.insert(entity.id, value);

                true
            }
            None => false,
        };

        self.being_updated.remove(&entity.id);

        self.update_depth -= 1;
        if self.update_depth == 0 {
            self.flush_notifications();
        }

        updated
    }

    //** there are two here like FnOnce I trid to implement here next is the impl FnMut(&mut App without static here
    fn observe<T: 'static>(
        &mut self,
        entity: &Entity<T>,
        callback: impl FnMut(&mut App) + 'static,
    ) {
        // push the closure to the vec here
        // so here I have to push like if entityId is there then I have to push the code else I have to  create new one here

        self.observers
            .entry(entity.id)
            .or_default()
            .push(Box::new(callback));
    }

    fn notify(&mut self, id: EntityId) {
        self.pending_notifications.push(id);
    }

    fn flush_notifications(&mut self) {
        let notifications = std::mem::take(&mut self.pending_notifications);
        // instead of clone I used the p
        for id in notifications {
            let callback = self.observers.remove(&id);

            if let Some(mut closures) = callback {
                for closure in closures.iter_mut() {
                    closure(self)
                }
                self.observers.insert(id, closures);
            }
        }
    }
}

// app.observe(&entity, |_app| {
//     println!("counter changed!");
// });
//

// app.notify(entity.id);   // should print "counter changed!"
struct Counter {
    count: i32,
}

fn main() {
    let mut app = App::new();
    let entity = app.insert(Counter { count: 5 });
    let entity_id = entity.id;
    let entity_copy = Entity {
        id: entity.id,
        _marker: PhantomData::<Counter>,
    };

    app.observe(&entity, move |app| {
        let count = app.read(&entity_copy).map(|counter| counter.count);
        println!("observer sees: {:?}", count);
    });

    app.update(&entity, |counter, app| {
        counter.count += 1;
        app.notify(entity_id); // notify INSIDE the update
    });
    println!("--- second update ---");
    app.update(&entity, |counter, app| {
        counter.count += 1;
        app.notify(entity_id);
    });
}
