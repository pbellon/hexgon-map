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

export function ownerOf(
  { users }: GameData,
  tileOrUserId: Tile | string
): User {
  const userId =
    typeof tileOrUserId === "string" ? tileOrUserId : tileOrUserId.user_id;
  const owner = users.find(({ id }) => id === userId);
  if (!owner) {
    throw new Error(
      `Owner with ${userId} ID not found. This should not happen`
    );
  }
  return owner;
}
