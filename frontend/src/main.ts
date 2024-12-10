import "./style.css";

import {
  AmbientLight,
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

import { CubeCoords } from "./types";
import { OrbitControls } from "three/examples/jsm/Addons.js";
import { axialToPixel } from "./grid";

type OnClickCallback = (data: CubeCoords) => void;

type WithCallback<T> = T & { onClick: OnClickCallback };

type HexMesh = Mesh<any>;

const HEX_SIZE = 10;
const HEX_SPACING = 1;
const HEX_DEPTH = 1.5;
const HEX_COLOR = 0xf5fecf;
const MAP_RADIUS = 20;

function createHexagon(size: number, depth: number, color: number): HexMesh {
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

  const material = new MeshPhongMaterial({ color });
  return new Mesh(geometry, material);
}

export function generateHexCoordinates(radius: number): CubeCoords[] {
  const coordinates = [];
  for (let q = -radius; q <= radius; q++) {
    for (
      let r = Math.max(-radius, -q - radius);
      r <= Math.min(radius, -q + radius);
      r++
    ) {
      const s = -q - r; // Derived from q + r + s = 0
      coordinates.push({ q, r, s });
    }
  }
  return coordinates;
}

export function createHexMap(radius: number, onClick: OnClickCallback): Group {
  const hexGroup = new Group();
  const coordinates = generateHexCoordinates(radius);

  coordinates.forEach((coords) => {
    const { x, y } = axialToPixel(coords, HEX_SIZE + HEX_SPACING);
    const hex = createHexagon(HEX_SIZE, HEX_DEPTH, HEX_COLOR); // Default color

    hex.position.set(x, y, 0);
    hex.userData = coords; // Attach cube coordinates to hex
    (hex as WithCallback<typeof hex>).onClick = onClick;

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
      const hex = intersects[0].object as WithCallback<HexMesh>;
      if (hex.onClick) {
        hex.onClick(hex.userData as CubeCoords);
      }
    }
  }

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("click", onClick);
}

// Create renderer
const canvas = document.getElementById("render") as HTMLCanvasElement;

const scene = new Scene();
const camera = new PerspectiveCamera(
  75,
  window.innerWidth / window.innerHeight,
  0.1,
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

const hexMap = createHexMap(MAP_RADIUS, (data) => {
  console.log("Hex clicked: ", data);
});

handleHexInteraction(camera, hexMap);

scene.add(hexMap);

const ambientLight = new AmbientLight(0xffffff, 0.95); // Soft global light
scene.add(ambientLight);

const directionalLight = new DirectionalLight(0xffffff, 1.2);
directionalLight.position.set(100, 100, 1000); // Angled above the grid
scene.add(directionalLight);

function animate() {
  requestAnimationFrame(animate);
  controls.update();
  renderer.render(scene, camera);
}

animate();
