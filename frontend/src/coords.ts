import { CubeCoords, AxialCoords } from "./types";

export function cubeAsAxial(coords: CubeCoords): AxialCoords {
  return {
    q: coords.q,
    r: coords.r,
  };
}

export function cubeSubstract(a: CubeCoords, b: CubeCoords): CubeCoords {
  return { q: a.q - b.q, r: a.r - b.r, s: a.s - b.s };
}

export function cubeAdd(a: CubeCoords, b: CubeCoords): CubeCoords {
  return { q: a.q + b.q, r: a.r + b.r, s: a.s + b.s };
}

export function cubeScale(a: CubeCoords, factor: number): CubeCoords {
  return { q: a.q * factor, r: a.r * factor, s: a.s * factor };
}

const DIRECTIONS: CubeCoords[] = [
  { q: 1, r: 0, s: -1 },
  { q: 1, r: -1, s: 0 },
  { q: 0, r: -1, s: 1 },
  { q: -1, r: 0, s: 1 },
  { q: -1, r: 1, s: 0 },
  { q: 0, r: 1, s: -1 },
];

export function cubeDirection(dir: number): CubeCoords {
  return DIRECTIONS[dir];
}

export function cubeNeighbor(coords: CubeCoords, dir: number): CubeCoords {
  return cubeAdd(coords, cubeDirection(dir));
}

export function cubeRing(center: CubeCoords, radius: number): CubeCoords[] {
  let results = [];

  let coords = cubeAdd(center, cubeScale(cubeDirection(4), radius));

  for (let i = 0; i < 6; i++) {
    for (let j = 0; j < radius; j++) {
      results.push(coords);
      coords = cubeNeighbor(coords, i);
    }
  }

  return results;
}

export function cubeSpiral(center: CubeCoords, radius: number): CubeCoords[] {
  let results = [center];

  let max = radius + 1;

  for (let k = 1; k < max; k++) {
    results = results.concat(cubeRing(center, k));
  }

  return results;
}
