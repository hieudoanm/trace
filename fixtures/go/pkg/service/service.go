package service

import "github.com/example/trace/fixtures/go/pkg/math"

type Repository interface {
	Find(id string) (*math.Circle, bool)
	Save(id string, c *math.Circle)
}

type MemoryRepository struct {
	store map[string]*math.Circle
}

func NewMemoryRepository() *MemoryRepository {
	return &MemoryRepository{store: make(map[string]*math.Circle)}
}

func (m *MemoryRepository) Find(id string) (*math.Circle, bool) {
	c, ok := m.store[id]
	return c, ok
}

func (m *MemoryRepository) Save(id string, c *math.Circle) {
	m.store[id] = c
}

type ShapeService struct {
	repo Repository
}

func NewShapeService(repo Repository) *ShapeService {
	return &ShapeService{repo: repo}
}

func (s *ShapeService) Area(id string) float64 {
	c, ok := s.repo.Find(id)
	if !ok {
		panic("not found")
	}
	return c.Area()
}
