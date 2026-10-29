use crate::shapes::{Circle, Shape};

pub trait Repository<T> {
    fn find(&self, id: &str) -> Option<T>;
    fn save(&mut self, id: String, entity: T);
}

pub struct InMemoryRepository<T> {
    store: std::collections::HashMap<String, T>,
}

impl<T> InMemoryRepository<T> {
    pub fn new() -> Self {
        Self {
            store: std::collections::HashMap::new(),
        }
    }
}

impl<T: Clone> Repository<T> for InMemoryRepository<T> {
    fn find(&self, id: &str) -> Option<T> {
        self.store.get(id).cloned()
    }

    fn save(&mut self, id: String, entity: T) {
        self.store.insert(id, entity);
    }
}

pub struct ShapeService {
    repo: std::collections::HashMap<String, Circle>,
}

impl ShapeService {
    pub fn new() -> Self {
        Self {
            repo: std::collections::HashMap::new(),
        }
    }

    pub fn add(&mut self, id: String, circle: Circle) {
        self.repo.insert(id, circle);
    }

    pub fn area(&self, id: &str) -> f64 {
        let circle = self.repo.get(id).expect("not found");
        circle.area()
    }
}
