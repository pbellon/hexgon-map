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

  function handeTileChange(data: Uint8Array) {
    const view = new DataView(data.buffer);
    // Process the binary data (for example, extracting coordinates and tile data)
    const q = view.getInt32(1, true); // Read the q value (i32)
    const r = view.getInt32(5, true); // Read the r value (i32)
    const strength = data[9]; // Read the strength (u8)
    const userIdLength = data[10]; // Read the user ID length (u8)

    // Read the user ID
    let userId = "";
    if (userIdLength > 0) {
      userId = new TextDecoder().decode(data.slice(11, 11 + userIdLength));
    }

    console.log("[ws/handleTileChange]", {
      q,
      r,
      strength,
      userId,
    });

    const hex = hexMap.getObjectByName(getTileName({ q, r })) as Mesh;
    if (hex) {
      let user;
      try {
        user = gameData.users.find(({ id }) => id === userId);
        console.log({
          user,
          users: [...gameData.users.map((u) => ({ ...u }))],
        });
        if (user) {
          (hex.material as MeshPhongMaterial).color.set(
            hexagonColor(user.color, strength)
          );
        }
      } catch (e) {
        console.error(`Did not found user for ${userId} ID`, gameData.users, e);
        // fail silently
      }
    }
  }

  function handleNewUserMessage(data: Uint8Array) {
    // Create a DataView instance for efficient reading of binary data
    const view = new DataView(data.buffer);

    // Index 1: length of the user ID (u8)
    const idLength = view.getUint8(1);

    // Index 2 to 2 + idLength: user ID bytes
    const id = new TextDecoder().decode(data.slice(2, 2 + idLength));

    // The next byte is the length of the user name (u8)
    const usernameLength = data[2 + idLength]; // userName length comes after user ID

    // Index 2 + idLength + 1 to 2 + idLength + usernameLength: user name bytes
    const username = new TextDecoder().decode(
      data.slice(3 + idLength, 3 + idLength + usernameLength)
    );

    // The next byte is the length of the user color (u8)
    const colorLength = data[3 + idLength + usernameLength];

    // Index 3 + idLength + usernameLength + 1 to 3 + idLength + usernameLength + colorLength: user color bytes
    const color = new TextDecoder().decode(
      data.slice(
        4 + idLength + usernameLength,
        4 + idLength + usernameLength + colorLength
      )
    );

    console.log("[ws/handleNewUserMessage]", {
      username, // please help me parse those
      color,
      id,
    });

    gameData.users.push({
      username, // please help me parse those
      color,
      id,
    });
  }

  function handleScoreChangeMessage(data: Uint8Array) {
    // TODO
  }

  // WEBSOCKET
  const socket = new WebSocket("ws://localhost:8080/ws"); // Adjust the URL if needed
  socket.binaryType = "arraybuffer";

  // Function to handle incoming messages
  socket.addEventListener("message", (e) => {
    console.log("Received message", e.data);

    if (!(e.data instanceof ArrayBuffer)) {
      return;
    }

    // Convert the ArrayBuffer into a byte array
    const data = new Uint8Array(e.data);

    // First byte is the message type
    const messageType = data[0];

    switch (messageType) {
      case 0x01:
        handeTileChange(data);
        break;
      case 0x02: // new player
        handleNewUserMessage(data);
        break;
      case 0x03: // Score change message
        handleScoreChangeMessage(data);
        break;
      // Other cases for different message types (e.g., player login)
      default:
        console.error("Unknown message type:", messageType);
    }
  });

  // Handle WebSocket errors
  socket.onerror = function (error) {
    console.error("WebSocket Error: ", error);
  };

  // Handle WebSocket connection open event
  socket.onopen = function (event) {
    console.log("WebSocket is open now.");
  };

  // Handle WebSocket connection close event
  socket.onclose = function (event) {
    console.log("WebSocket is closed now.");
  };
}

// flow
// 1. fetch game data
// 2. render grid + scores
// 3. pseudo auth
// 4. allow interactivity
