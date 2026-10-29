<?php

class InMemoryRepository {
    private $store = array();

    public function find($id) {
        if (isset($this->store[$id])) {
            return $this->store[$id];
        }
        return null;
    }

    public function save($id, $entity) {
        $this->store[$id] = $entity;
    }
}

class ShapeService {
    private $repo;

    public function __construct($repo) {
        $this->repo = $repo;
    }

    public function area($id) {
        $shape = $this->repo->find($id);
        if ($shape === null) {
            throw new Exception("not found");
        }
        return $shape->area();
    }
}

function createService() {
    return new ShapeService(new InMemoryRepository());
}
