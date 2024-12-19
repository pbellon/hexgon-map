import { Mesh } from "three";

export type CubeCoords = {
  q: number;
  r: number;
  s: number;
};

export type HexUserData = {
  user_id: string | undefined;
  coords: AxialCoords;
};

export type AxialCoords = {
  q: number;
  r: number;
};

export type PointCoords = {
  x: number;
  y: number;
};

export type OnClickCallback = (data: HexUserData, hex: Mesh) => void;

export type WithCallback<T> = T & { onClick: OnClickCallback };

export interface User {
  username: string;
  token: string;
  color: string;
  id: string;
}

export type PublicUser = Omit<User, "token">;

export type UserWithAuth = User & {
  token: string;
};

export type CoordsAndTile = [coords: AxialCoords, tile: Tile];

export type GameSettings = {
  radius: number;
};

export type BatchTile = [
  q: number,
  r: number,
  strength: number,
  userId: string
];

export interface Tile {
  user_id: string;
  strength: number;
}
