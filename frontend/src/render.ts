import {
  AmbientLight,
  Color,
  DirectionalLight,
  ExtrudeGeometry,
  ExtrudeGeometryOptions,
  Group,
  Mesh,
  MeshPhongMaterial,
  MOUSE,
  PerspectiveCamera,
  Raycaster,
  Scene,
  Shape,
  Vector2,
  WebGLRenderer,
} from "three";
import {
  axialToPixel,
  generateHexCoordinates,
  getTileName,
  ownerOf,
  tileAt,
  tileOpacity,
} from "./grid";
import { GameData, AxialCoords, OnClickCallback, WithCallback } from "./types";
import { OrbitControls } from "three/examples/jsm/Addons.js";
import { initApi } from "./api";
import { HEX_COLOR, HEX_DEPTH, HEX_SIZE, HEX_SPACING } from "./constants";

const BASE_COLOR = new Color(HEX_COLOR);

function hexagonColor(color: string, strength: number): Color {
  const hex = parseInt(color.slice(1), 16);
  const tColor = new Color(hex);
  const opacity = tileOpacity(strength);
  console.log(`opacity(${strength}) => ${opacity}`);
  return BASE_COLOR.clone().lerp(tColor, opacity);
}

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
    color: strength > 0 ? hexagonColor(color, strength) : BASE_COLOR,
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

export function handleHexInteraction(
  camera: PerspectiveCamera,
  hexGroup: Group
) {
  const raycaster = new Raycaster();
  const mouse = new Vector2();

  function onMouseMove(event: MouseEvent) {
    mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
    mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;
  }

  function onClick(_event: MouseEvent) {
    raycaster.setFromCamera(mouse, camera);
    const intersects = raycaster.intersectObjects(hexGroup.children, true);
    if (intersects.length > 0) {
      const hex = intersects[0].object as WithCallback<Mesh>;
      if (hex.onClick) {
        hex.onClick(hex.userData as AxialCoords, hex);
      }
    }
  }

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("click", onClick);
}

export async function render(api: ReturnType<typeof initApi>) {
  // Create renderer
  const canvas = document.getElementById("render") as HTMLCanvasElement;

  const scene = new Scene();
  const camera = new PerspectiveCamera(
    75,
    window.innerWidth / window.innerHeight,
    5,
    1000
  );

  camera.position.set(0, 0, 200); // Elevated position
  // camera.lookAt(50, 50, 0); // Focus on the center of the grid

  const renderer = new WebGLRenderer({ canvas, antialias: true });

  renderer.setPixelRatio(window.devicePixelRatio); // Ensure smooth rendering on high-DPI screens
  renderer.setSize(window.innerWidth, window.innerHeight);

  renderer.setClearColor(0x111111); // Dark gray background
  renderer.setSize(window.innerWidth, window.innerHeight);

  document.body.appendChild(renderer.domElement);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableRotate = false;
  controls.minDistance = 10;
  controls.maxDistance = 300;
  controls.mouseButtons = {
    LEFT: MOUSE.PAN,
  };

  const data = await api.fetchGameData();

  const hexMap = createHexMap(data, async (tileData, hex) => {
    // TODO: compute strength localy based on current data

    // send data and reconcile after response
    const updatedTiles = await api.clickAt(tileData);

    updatedTiles.forEach(([coords, tile]) => {
      const hex = hexMap.getObjectByName(getTileName(coords)) as Mesh;
      if (hex) {
        // could break here
        const owner = ownerOf(data, tile);
        (hex.material as MeshPhongMaterial).color.set(
          hexagonColor(owner.color, tile.strength)
        );
      }
    });
  });

  handleHexInteraction(camera, hexMap);

  scene.add(hexMap);

  const ambientLight = new AmbientLight(0xffffff, 0.95); // Soft global light
  scene.add(ambientLight);

  const directionalLight = new DirectionalLight(0xffffff, 0.3);
  directionalLight.position.set(100, 100, 500); // Angled above the grid
  scene.add(directionalLight);

  function animate() {
    requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
  }

  animate();
}

// flow
// 1. fetch game data
// 2. render grid + scores
// 3. pseudo auth
// 4. allow interactivity
