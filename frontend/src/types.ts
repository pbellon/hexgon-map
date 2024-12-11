import { Mesh } from "three";

export interface CubeCoords {
  q: number;
  r: number;
  s: number;
}

export interface AxialCoords {
  q: number;
  r: number;
}

export interface PointCoords {
  x: number;
  y: number;
}

export type OnClickCallback = (data: AxialCoords, hex: Mesh) => void;

export type WithCallback<T> = T & { onClick: OnClickCallback };
