import { GameData, AxialCoords, CoordsAndTile, User } from "./types";
import { webSocketHandler, WebSocketHandlersParams } from "./websocket";

export type GameApi = ReturnType<typeof initApi>;

interface LocalGameState {
  user: User | undefined;
  tiles: CoordsAndTile[];
  users: User[];
}

export function initApi() {
  const host = (path: string) => `localhost:8080${path}`;

  const fullUrl = (path: string) => `http://${host(path)}`;

  // TODO: store & restore from localStorage
  // state
  let state: LocalGameState = {
    user: undefined,
    tiles: [],
    users: [],
  };

  const fetchGameData = async (): Promise<GameData> => {
    const response = await fetch(fullUrl("/data"), { method: "get" });
    const data = (await response.json()) as GameData;

    state.tiles = data.tiles;
    state.users = data.users;

    return data;
  };

  const clickAt = async (coords: AxialCoords): Promise<CoordsAndTile[]> => {
    if (state.user) {
      const response = await fetch(fullUrl(`/tile/${coords.q}/${coords.r}`), {
        method: "POST",
        headers: {
          "content-type": "application/json",
        },
        body: state.user.id,
      });

      return (await response.json()) as CoordsAndTile[];
    }

    return [];
  };

  const login = async (username: string): Promise<User> => {
    const response = await fetch(fullUrl("/login"), {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify({
        username,
      }),
    });

    const user = await response.json();

    state.users.push(user);
    state.user = user;

    return user as User;
  };

  function configureWebSocket(params: WebSocketHandlersParams): WebSocket {
    return webSocketHandler(`ws://${host("/ws")}`, params);
  }

  return {
    configureWebSocket,
    fetchGameData,
    clickAt,
    login,
  };
}
