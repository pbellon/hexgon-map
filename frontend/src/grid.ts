import { CubeCoords, PointCoords } from "./types";

export function axialToPixel({ q, r }: CubeCoords, size: number): PointCoords {
  const x = size * Math.sqrt(3) * (q + r / 2);
  const y = size * (3 / 2) * r;
  return { x, y };
}
