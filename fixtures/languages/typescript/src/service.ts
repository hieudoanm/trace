import { Circle } from "./math";

export interface Repository<T> {
  find(id: string): T | undefined;
  save(entity: T): void;
}

export class InMemoryRepository<T> implements Repository<T> {
  private store = new Map<string, T>();

  find(id: string): T | undefined {
    return this.store.get(id);
  }

  save(entity: T): void {
    this.store.set((entity as any).id, entity);
  }
}

export class ShapeService {
  constructor(private repo: InMemoryRepository<Circle>) {}

  area(id: string): number {
    const shape = this.repo.find(id);
    if (!shape) throw new Error("not found");
    return shape.area();
  }
}

export function createService(): ShapeService {
  const repo = new InMemoryRepository<Circle>();
  return new ShapeService(repo);
}
