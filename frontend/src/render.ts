import {
  AmbientLight,
  DirectionalLight,
  Group,
  Mesh,
  MeshPhongMaterial,
  MOUSE,
  PerspectiveCamera,
  Raycaster,
  Scene,
  Vector2,
  WebGLRenderer,
} from "three";
import { getTileName, ownerOf } from "./grid";
import { GameData, AxialCoords, WithCallback } from "./types";
import { OrbitControls } from "three/examples/jsm/Addons.js";
import { initApi } from "./api";
import { createHexMap } from "./shapes";
import { hexagonColor } from "./colors";

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

export async function render(
  gameData: GameData,
  api: ReturnType<typeof initApi>
) {
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

  const hexMap = createHexMap(gameData, async (tileData) => {
    // TODO: compute strength localy based on current data

    // send data and reconcile after response
    const updatedTiles = await api.clickAt(tileData);

    updatedTiles.forEach(([coords, tile]) => {
      const hex = hexMap.getObjectByName(getTileName(coords)) as Mesh;
      if (hex) {
        // could break here
        const owner = ownerOf(gameData, tile);
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

  api.configureWebSocket({
    onNewUser: (user) => {
      gameData.users.push(user);
    },
    onTileChange: (coords, tile) => {
      const hex = hexMap.getObjectByName(getTileName(coords)) as Mesh;
      if (hex) {
        try {
          const user = ownerOf(gameData, tile);
          if (user) {
            (hex.material as MeshPhongMaterial).color.set(
              hexagonColor(user.color, tile.strength)
            );
          }
        } catch (e) {
          console.error(
            `Did not found user for ${tile.user_id} ID`,
            gameData.users,
            e
          );
          // fail silently
        }
      }
    },
  });
}

// flow
// 1. fetch game data
// 2. render grid + scores
// 3. pseudo auth
// 4. allow interactivity
