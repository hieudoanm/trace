package com.example;

import java.util.HashMap;
import java.util.Map;

public class Service {
    public static class InMemoryRepository<T> {
        private final Map<String, T> store = new HashMap<>();

        public T find(String id) {
            return store.get(id);
        }

        public void save(String id, T entity) {
            store.put(id, entity);
        }
    }

    public static class ShapeService {
        private final InMemoryRepository<Circle> repo;

        public ShapeService(InMemoryRepository<Circle> repo) {
            this.repo = repo;
        }

        public double area(String id) {
            Circle shape = repo.find(id);
            if (shape == null) {
                throw new IllegalArgumentException("not found");
            }
            return shape.area();
        }
    }

    public static ShapeService createService() {
        return new ShapeService(new InMemoryRepository<Circle>());
    }
}
