package com.example

class InMemoryRepository<T> {
    private val store = mutableMapOf<String, T>()
    fun find(id: String): T? = store[id]
    fun save(id: String, entity: T) { store[id] = entity }
}

class ShapeService(private val repo: InMemoryRepository<Circle>) {
    fun area(id: String): Double {
        val shape = repo.find(id) ?: throw IllegalArgumentException("not found")
        return shape.area()
    }
}

fun createService(): ShapeService {
    return ShapeService(InMemoryRepository())
}
