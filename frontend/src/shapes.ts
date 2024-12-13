import {
  ExtrudeGeometry,
  ExtrudeGeometryOptions,
  Group,
  Mesh,
  MeshPhongMaterial,
  Shape,
} from "three";
import { GameData, OnClickCallback, WithCallback } from "./types";
import {
  axialToPixel,
  generateHexCoordinates,
  getTileName,
  ownerOf,
  tileAt,
} from "./grid";
import { HEX_COLOR, HEX_DEPTH, HEX_SIZE, HEX_SPACING } from "./constants";
import { hexagonColor } from "./colors";

export function createHexagon(
  size: number,
  color: string,
  strength: number,
  depth: number
): Mesh {
  const shape = new Shape();
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 3) * i - Math.PI / 6; // 30° offset for pointy tops
    const x = size * Math.cos(angle);
    const y = size * Math.sin(angle);
    if (i === 0) {
      shape.moveTo(x, y);
    } else {
      shape.lineTo(x, y);
    }
  }
  shape.closePath();

  const extrudeSettings: ExtrudeGeometryOptions = {
    depth, // Height of the hexagon
    bevelSize: 1, // How much to bevel inward
    bevelSegments: 2, // Smoothness of the bevel
    bevelThickness: 1, // How "deep" the bevel is
    bevelEnabled: true, // No bevel for flat surfaces
  };

  const geometry = new ExtrudeGeometry(shape, extrudeSettings);
  geometry.translate(0, 0, -depth / 2); // Center vertically

  const material = new MeshPhongMaterial({
    color: hexagonColor(color, strength),
    shininess: 25,
    specular: 0xcccccc,
  });
  return new Mesh(geometry, material);
}

export function createHexMap(data: GameData, onClick: OnClickCallback): Group {
  const hexGroup = new Group();
  const coordinates = generateHexCoordinates(data.settings.radius);

  coordinates.forEach((coords) => {
    let color = HEX_COLOR;
    let strength = 0;

    let tile = tileAt(data, coords);

    // should have corresponding user
    if (tile && tile.user_id) {
      const user = ownerOf(data, tile);
      color = user.color;
      strength = tile.strength;
    }

    const { x, y } = axialToPixel(coords, HEX_SIZE + HEX_SPACING);
    const hex = createHexagon(HEX_SIZE, color, strength, HEX_DEPTH); // Default color

    hex.position.set(x, y, 0);
    hex.userData = coords; // Attach cube coordinates to hex
    (hex as WithCallback<typeof hex>).onClick = onClick;

    hex.name = getTileName(coords);

    hexGroup.add(hex);
  });

  return hexGroup;
}
