import { GameData, Tile, User, AxialCoords, PointCoords } from "./types";

export function axialToPixel({ q, r }: AxialCoords, size: number): PointCoords {
  const x = size * Math.sqrt(3) * (q + r / 2);
  const y = size * (3 / 2) * r;
  return { x, y };
}

export function eqAxialCoords(a: AxialCoords, b: AxialCoords): boolean {
  return a.q === b.q && a.r === b.r;
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

export function getTileName({ q, r }: AxialCoords): string {
  return `${q + 200}-${r + 200}`;
}

export function tileAt(
  { tiles }: GameData,
  coords: AxialCoords
): Tile | undefined {
  return tiles.find(([b]) => eqAxialCoords(coords, b))?.[1];
}

export function ownerOf({ users }: GameData, tile: Tile): User {
  const owner = users.find(({ id }) => id === tile.user_id);
  if (!owner) {
    throw new Error(
      `Owner with ${tile.user_id} ID not found. This should not happen`
    );
  }
  return owner;
}

export function tileOpacity(strength: number): number {
  // Define the domain and range
  const domain = [0, 19]; // Input values (e.g., 0 to 19)
  const range = [0.2, 1]; // Output values (e.g., 0.1 to 1)

  // Linear scaling formula
  return (
    range[0] +
    ((strength - domain[0]) * (range[1] - range[0])) / (domain[1] - domain[0])
  );
}
