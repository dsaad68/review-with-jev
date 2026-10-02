use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
struct Speaker {
    name: String,
    topic: String,
}

struct Session {
    slot: u8,
    room: Rc<str>,
    speaker: Rc<Speaker>,
}

struct Agenda {
    sessions: Vec<Session>,
    by_room: HashMap<Rc<str>, Vec<u8>>,
}

impl Agenda {
    fn new() -> Self {
        Agenda { sessions: Vec::new(), by_room: HashMap::new() }
    }

    fn add(&mut self, slot: u8, room: &Rc<str>, speaker: Rc<Speaker>) {
        self.by_room.entry(room.clone()).or_default().push(slot);
        self.sessions.push(Session { slot, room: room.clone(), speaker });
    }

    fn print(&self) {
        let mut rooms: Vec<&Rc<str>> = self.by_room.keys().collect();
        rooms.sort();
        for room in rooms {
            println!("{room}: slots {:?}", self.by_room[room]);
        }
        for s in &self.sessions {
            println!(
                "slot {} in {}: {} on {} (shared by {})",
                s.slot,
                s.room,
                s.speaker.name,
                s.speaker.topic,
                Rc::strong_count(&s.speaker)
            );
        }
    }
}

fn main() {
    let hall: Rc<str> = Rc::from("Main Hall");
    let lab: Rc<str> = Rc::from("Lab 2");
    let ada = Rc::new(Speaker { name: String::from("Ada"), topic: String::from("Compilers") });
    let lin = Rc::new(Speaker { name: String::from("Lin"), topic: String::from("Databases") });

    let mut agenda = Agenda::new();
    agenda.add(1, &hall, ada.clone());
    agenda.add(2, &lab, lin.clone());
    agenda.add(3, &hall, lin.clone());
    agenda.add(4, &lab, ada.clone());
    agenda.print();
    println!("{} talks for {}, {} for {}", Rc::strong_count(&ada) - 1, ada.name, Rc::strong_count(&lin) - 1, lin.name);
}
