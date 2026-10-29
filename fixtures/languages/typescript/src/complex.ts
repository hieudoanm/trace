export interface Shape { area(): number; }
export class Circle implements Shape {
    constructor(public r: number) {}
    area(): number { return Math.PI * this.r * this.r; }
}
export function total(shapes: Shape[]): number { return shapes.reduce((a,s)=>a+s.area(),0); }
