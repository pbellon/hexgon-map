import { AxialCoords, PointCoords } from "./types";

export function axialToPixel({ q, r }: AxialCoords, size: number): PointCoords {
  const x = size * Math.sqrt(3) * (q + r / 2);
  const y = size * (3 / 2) * r;
  return { x, y };
}

export function generateHexCoordinates(radius: number): AxialCoords[] {
  const coordinates = [];
  for (let q = -radius; q <= radius; q++) {
    for (
      let r = Math.max(-radius, -q - radius);
      r <= Math.min(radius, -q + radius);
      r++
    ) {
      coordinates.push({ q, r });
    }
  }
  return coordinates;
}
