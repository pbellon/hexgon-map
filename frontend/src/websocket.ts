import { AxialCoords, Tile, User } from "./types";

export type WebSocketHandlersParams = {
  onTileChange: (coors: AxialCoords, tile: Tile) => void;
  onNewUser: (user: User) => void;
};

export function webSocketHandler(
  url: string,
  { onTileChange, onNewUser }: WebSocketHandlersParams
): WebSocket {
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
      const coords = { q, r };
      const tile = { user_id: userId, strength };
      console.log("[ws/handleTileChange]", { q, r, userId, strength });
      onTileChange(coords, tile);
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

    if (username.length > 0 && color.length > 0 && id.length > 0) {
      onNewUser({ id, color, username });
    }
  }

  function handleScoreChangeMessage(data: Uint8Array) {
    // TODO
  }

  // WEBSOCKET
  const socket = new WebSocket(url); // Adjust the URL if needed
  socket.binaryType = "arraybuffer";

  // Function to handle incoming messages
  socket.addEventListener("message", (e) => {
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

  // Register WebSocket event listeners
  socket.addEventListener("open", function (event) {
    console.log("WebSocket is open now.");
  });

  socket.addEventListener("error", function (error) {
    console.error("WebSocket Error: ", error);
  });

  socket.addEventListener("close", function (event) {
    console.log("WebSocket is closed now.");
  });

  return socket;
}
