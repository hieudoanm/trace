from .math import Circle


class Repository:
    def __init__(self):
        self._items = {}

    def find(self, item_id):
        return self._items.get(item_id)

    def save(self, item_id, item):
        self._items[item_id] = item


class ShapeService:
    def __init__(self, repo):
        self.repo = repo

    def area(self, item_id):
        shape = self.repo.find(item_id)
        if shape is None:
            raise ValueError("not found")
        return shape.area()


def create_service():
    repo = Repository()
    return ShapeService(repo)
