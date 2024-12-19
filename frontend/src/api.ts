import {
  AxialCoords,
  CoordsAndTile,
  User,
  BatchTile,
  GameSettings,
  PublicUser,
} from "./types";
import { webSocketHandler, WebSocketHandlersParams } from "./websocket";

export type GameApi = ReturnType<typeof initApi>;

interface LocalApiState {
  users: Record<string, PublicUser>;
  user: User | undefined;
}

export function initApi() {
  const host = (path: string) => `127.0.0.1:8080${path}`;

  const fullUrl = (path: string) => `http://${host(path)}`;

  // TODO: store & restore from localStorage
  // state
  let state: LocalApiState = {
    user: undefined,
    users: {},
  };

  function getAuth(user: User): string {
    return btoa(`${user.id}:${user.token}`);
  }

  async function fetchBatch(batch: number): Promise<CoordsAndTile[]> {
    const response = await fetch(fullUrl(`/tiles?batch=${batch}`), {
      method: "get",
    });
    let tiles = (await response.json()) as BatchTile[];

    return tiles.map(
      ([q, r, strength, user_id]) =>
        [
          { q, r },
          { strength, user_id },
        ] as CoordsAndTile
    );
  }

  async function fetchBatchesList(): Promise<number[]> {
    const response = await fetch(fullUrl("/batches"), { method: "get" });
    return (await response.json()) as number[];
  }

  async function fetchGameSettings(): Promise<GameSettings> {
    const response = await fetch(fullUrl("/settings"), { method: "get" });
    return (await response.json()) as GameSettings;
  }

  async function fetchUsers(): Promise<Record<string, PublicUser>> {
    const response = await fetch(fullUrl("/users"), { method: "get" });
    const data = (await response.json()) as PublicUser[];

    state.users = Object.fromEntries(data.map((data) => [data.id, data]));

    return state.users;
  }

  const clickAt = async (coords: AxialCoords): Promise<CoordsAndTile[]> => {
    console.log("[api/clickAt]", coords);
    if (state.user != null) {
      const headers = {
        Authorization: `Basic ${getAuth(state.user)}`,
        "content-type": "application/json",
      };

      const response = await fetch(fullUrl(`/tile/${coords.q}/${coords.r}`), {
        method: "POST",
        headers,
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

    state.user = user;

    state.users[user.id] = {
      id: user.id,
      color: user.color,
      username: user.username,
    };

    return user as User;
  };

  function configureWebSocket(params: WebSocketHandlersParams): WebSocket {
    return webSocketHandler(`ws://${host("/ws")}`, params);
  }

  return {
    clickAt,
    configureWebSocket,
    fetchBatch,
    fetchBatchesList,
    fetchGameSettings,
    fetchUsers,
    login,
    state,
  };
}
