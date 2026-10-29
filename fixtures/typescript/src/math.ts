export interface Shape { area(): number; }
export class Circle implements Shape {
  constructor(private radius: number) {}
  area(): number { return Math.PI * this.radius * this.radius; }
}
export function twice(n: number): number { return n * 2; }
export type ID = string;
export const label = "circle";
import { Circle, twice, Shape } from "./math";
import * as path from "path";
export function run(): number {
  const c = new Circle(2);
  return twice(c.area());
}
