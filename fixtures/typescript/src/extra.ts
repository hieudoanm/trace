import Default, { a as b, c } from "./x";
import * as ns from "./y";
export { a, b as renamed } from "./z";
export * from "./w";
export default function foo() {}
export const enum E { A, B }
namespace N { export class Inner { private p: number = 1; public m(): void { this.p; } } }
const arrow = (x: number): number => x;
let plain: Circle = new Circle();
