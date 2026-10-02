use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::{Rc, Weak};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct EmployeeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Role {
    Engineer,
    Manager,
    Director,
}

struct Employee {
    id: EmployeeId,
    name: String,
    role: Role,
    manager: RefCell<Weak<Employee>>,
    reports: RefCell<Vec<Rc<Employee>>>,
    mentor: Option<EmployeeId>,
}

#[derive(Debug)]
enum OrgError {
    UnknownEmployee(EmployeeId),
    AlreadyManaged(EmployeeId),
}

impl fmt::Display for OrgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrgError::UnknownEmployee(id) => write!(f, "no employee {}", id.0),
            OrgError::AlreadyManaged(id) => write!(f, "employee {} already has a manager", id.0),
        }
    }
}

trait Visitor {
    fn visit(&mut self, employee: &Employee, depth: usize);
}

struct Directory {
    by_id: HashMap<EmployeeId, Rc<Employee>>,
}

impl Directory {
    fn new() -> Self {
        Directory { by_id: HashMap::new() }
    }

    fn hire(&mut self, id: u32, name: &str, role: Role, mentor: Option<u32>) -> EmployeeId {
        let id = EmployeeId(id);
        let employee = Employee {
            id,
            name: name.to_string(),
            role,
            manager: RefCell::new(Weak::new()),
            reports: RefCell::new(Vec::new()),
            mentor: mentor.map(EmployeeId),
        };
        self.by_id.insert(id, Rc::new(employee));
        id
    }

    fn get(&self, id: EmployeeId) -> Result<&Rc<Employee>, OrgError> {
        self.by_id.get(&id).ok_or(OrgError::UnknownEmployee(id))
    }

    fn assign(&self, manager: EmployeeId, report: EmployeeId) -> Result<(), OrgError> {
        let boss = self.get(manager)?;
        let worker = self.get(report)?;
        if worker.manager.borrow().upgrade().is_some() {
            return Err(OrgError::AlreadyManaged(report));
        }
        *worker.manager.borrow_mut() = Rc::downgrade(boss);
        boss.reports.borrow_mut().push(Rc::clone(worker));
        Ok(())
    }

    fn chain_of_command(&self, id: EmployeeId) -> Result<Vec<String>, OrgError> {
        let mut chain = Vec::new();
        let mut current = self.get(id)?.manager.borrow().upgrade();
        while let Some(person) = current {
            chain.push(person.name.clone());
            current = person.manager.borrow().upgrade();
        }
        Ok(chain)
    }

    fn mentor_name(&self, id: EmployeeId) -> Result<Option<&str>, OrgError> {
        match self.get(id)?.mentor {
            Some(mentor) => Ok(Some(self.get(mentor)?.name.as_str())),
            None => Ok(None),
        }
    }
}

fn walk<V: Visitor>(node: &Rc<Employee>, depth: usize, visitor: &mut V) {
    visitor.visit(node, depth);
    for report in node.reports.borrow().iter() {
        walk(report, depth + 1, visitor);
    }
}

#[derive(Default)]
struct Headcount {
    per_role: HashMap<Role, usize>,
    seen: Vec<EmployeeId>,
    deepest: usize,
}

impl Visitor for Headcount {
    fn visit(&mut self, employee: &Employee, depth: usize) {
        *self.per_role.entry(employee.role).or_insert(0) += 1;
        self.seen.push(employee.id);
        self.deepest = self.deepest.max(depth);
    }
}

fn main() {
    let mut dir = Directory::new();
    let ada = dir.hire(1, "Ada", Role::Director, None);
    let bo = dir.hire(2, "Bo", Role::Manager, Some(1));
    let cy = dir.hire(3, "Cy", Role::Engineer, Some(2));
    let di = dir.hire(4, "Di", Role::Engineer, Some(1));

    for (boss, worker) in [(ada, bo), (bo, cy), (bo, di), (ada, cy)] {
        if let Err(err) = dir.assign(boss, worker) {
            println!("skipped: {err}");
        }
    }

    match dir.chain_of_command(cy) {
        Ok(chain) => println!("Cy reports up through {}", chain.join(" -> ")),
        Err(err) => println!("lookup failed: {err}"),
    }
    match dir.mentor_name(di) {
        Ok(name) => println!("Di's mentor: {name:?}"),
        Err(err) => println!("lookup failed: {err}"),
    }

    match dir.get(ada) {
        Ok(root) => {
            let mut count = Headcount::default();
            walk(root, 0, &mut count);
            count.seen.sort();
            let mut roles: Vec<_> = count.per_role.into_iter().collect();
            roles.sort();
            println!("visited {:?}, depth {}, roles {roles:?}", count.seen, count.deepest);
        }
        Err(err) => println!("lookup failed: {err}"),
    }
}
