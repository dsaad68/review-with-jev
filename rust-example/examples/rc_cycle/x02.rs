use std::cell::RefCell;
use std::rc::Rc;

struct Owner {
    name: String,
    pets: Vec<Rc<RefCell<Pet>>>,
}

struct Pet {
    name: String,
    owner: Option<Rc<RefCell<Owner>>>,
}

fn adopt(owner: &Rc<RefCell<Owner>>, pet: &Rc<RefCell<Pet>>) {
    pet.borrow_mut().owner = Some(Rc::clone(owner));
    owner.borrow_mut().pets.push(Rc::clone(pet));
}

fn main() {
    let owner = Rc::new(RefCell::new(Owner { name: "Ines".to_string(), pets: Vec::new() }));
    for name in ["Biscuit", "Pepper"] {
        adopt(&owner, &Rc::new(RefCell::new(Pet { name: name.to_string(), owner: None })));
    }
    for pet in &owner.borrow().pets {
        let pet = pet.borrow();
        let keeper = pet.owner.as_ref().map(|o| o.borrow().name.len());
        println!("{} belongs to an owner with a {keeper:?}-letter name", pet.name);
    }
    println!("owner refs: {}", Rc::strong_count(&owner));
}
