use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

struct City {
    name: String,
    roads: RefCell<Vec<(Rc<City>, u32)>>,
}

fn city(name: &str) -> Rc<City> {
    Rc::new(City { name: name.to_string(), roads: RefCell::new(Vec::new()) })
}

fn connect(a: &Rc<City>, b: &Rc<City>, km: u32) {
    a.roads.borrow_mut().push((Rc::clone(b), km));
    b.roads.borrow_mut().push((Rc::clone(a), km));
}

fn hops_between(start: &Rc<City>, goal: &str) -> Option<usize> {
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([(Rc::clone(start), 0)]);
    seen.insert(start.name.clone());
    while let Some((current, hops)) = queue.pop_front() {
        if current.name == goal {
            return Some(hops);
        }
        for (next, _) in current.roads.borrow().iter() {
            if seen.insert(next.name.clone()) {
                queue.push_back((Rc::clone(next), hops + 1));
            }
        }
    }
    None
}

fn longest_road(cities: &[Rc<City>]) -> Option<(String, String, u32)> {
    cities
        .iter()
        .filter_map(|c| {
            let roads = c.roads.borrow();
            let (next, km) = roads.iter().max_by_key(|(_, km)| *km)?;
            Some((c.name.clone(), next.name.clone(), *km))
        })
        .max_by_key(|&(_, _, km)| km)
}

fn main() {
    let names = ["Lyon", "Turin", "Geneva", "Milan", "Bern"];
    let cities: Vec<Rc<City>> = names.iter().map(|n| city(n)).collect();
    connect(&cities[0], &cities[2], 150);
    connect(&cities[2], &cities[4], 160);
    connect(&cities[0], &cities[1], 310);
    connect(&cities[1], &cities[3], 140);
    println!("Lyon to Milan: {:?} hops", hops_between(&cities[0], "Milan"));
    println!("Bern to Turin: {:?} hops", hops_between(&cities[4], "Turin"));
    println!("longest road: {:?}", longest_road(&cities));
}
