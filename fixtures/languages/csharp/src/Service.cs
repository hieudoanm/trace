using System;
using System.Collections.Generic;

namespace Example;

public class InMemoryRepository<T>
{
    private readonly Dictionary<string, T> store = new();

    public T? Find(string id)
    {
        store.TryGetValue(id, out var value);
        return value;
    }

    public void Save(string id, T entity)
    {
        store[id] = entity;
    }
}

public class ShapeService
{
    private readonly InMemoryRepository<Circle> repo;

    public ShapeService(InMemoryRepository<Circle> repo)
    {
        this.repo = repo;
    }

    public double Area(string id)
    {
        var shape = repo.Find(id);
        if (shape == null) throw new Exception("not found");
        return shape.Area();
    }

    public static ShapeService CreateService()
    {
        return new ShapeService(new InMemoryRepository<Circle>());
    }
}
