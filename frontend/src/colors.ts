import { Color } from "three";
import { linearScale } from "./math";
import { HEX_COLOR } from "./constants";

const BASE_COLOR = new Color(HEX_COLOR);

const tileOpacity = linearScale({
  domain: [0, 19],
  range: [0.25, 1],
});

export function hexagonColor(color: string, strength: number): Color {
  if (strength === 0) {
    return BASE_COLOR;
  }

  const hex = parseInt(color.slice(1), 16);
  const tColor = new Color(hex);
  const opacity = tileOpacity(strength);
  // console.log(`opacity(${strength}) => ${opacity}`);
  return new Color().lerpColors(BASE_COLOR, tColor, opacity);
}
