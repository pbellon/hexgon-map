import { Mesh } from "three";

export type CubeCoords = {
  q: number;
  r: number;
  s: number;
};

export type AxialCoords = {
  q: number;
  r: number;
};

export type PointCoords = {
  x: number;
  y: number;
};

export type OnClickCallback = (data: AxialCoords, hex: Mesh) => void;

export type WithCallback<T> = T & { onClick: OnClickCallback };

export interface User {
  username: string;
  token: string;
  color: string;
  id: string;
}

export type UserWithAuth = User & {
  token: string;
};

export type CoordsAndTile = [coords: AxialCoords, tile: Tile];

export type GameSettings = {
  radius: number;
};

export interface GameData {
  settings: GameSettings;
  users: User[];
  tiles: CoordsAndTile[];
}

export interface ApiGameData {
  settings: GameSettings;
  users: User[];
  tiles: [q: number, r: number, strength: number, userId: string | undefined][];
}

export interface Tile {
  user_id: string | undefined;
  strength: number;
}
