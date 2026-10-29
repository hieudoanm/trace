require_relative './math'

class InMemoryRepository
  def initialize
    @store = {}
  end

  def find(id)
    @store[id]
  end

  def save(id, entity)
    @store[id] = entity
  end
end

class ShapeService
  def initialize(repo)
    @repo = repo
  end

  def area(id)
    shape = @repo.find(id)
    raise 'not found' if shape.nil?
    shape.area
  end
end

def create_service
  ShapeService.new(InMemoryRepository.new)
end
