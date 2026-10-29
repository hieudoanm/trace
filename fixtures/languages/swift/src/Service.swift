class InMemoryRepository<T> {
    var store: [String: T] = [:]
    func find(id: String) -> T? {
        return store[id]
    }
    func save(id: String, entity: T) {
        store[id] = entity
    }
}

class ShapeService {
    let repo: InMemoryRepository<Circle>

    init(repo: InMemoryRepository<Circle>) {
        self.repo = repo
    }

    func area(id: String) -> Double {
        guard let shape = repo.find(id: id) else {
            fatalError("not found")
        }
        return shape.area()
    }
}

func createService() -> ShapeService {
    return ShapeService(repo: InMemoryRepository<Circle>())
}
