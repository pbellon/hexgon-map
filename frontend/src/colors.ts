import { Color } from "three";
import { linearScale } from "./math";
import { HEX_COLOR } from "./constants";

const BASE_COLOR = new Color(HEX_COLOR);

const tileOpacity = linearScale({
  domain: [0, 19],
  range: [0.25, 1],
});

export function hexagonColor(color: number, strength: number): Color {
  if (strength === 0) {
    return BASE_COLOR;
  }

  const tColor = new Color(color);
  const opacity = tileOpacity(strength);
  // console.log(`opacity(${strength}) => ${opacity}`);
  return new Color().lerpColors(BASE_COLOR, tColor, opacity);
}
