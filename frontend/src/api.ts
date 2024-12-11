import { AxialCoords } from "./types";

interface ApiGrid {
  // TODO
}

interface ApiTile {
  user_id: string;
  strength: number;
  color: string;
}

export function initApi() {
  const fullUrl = (path: string) => `http://localhost:8080${path}`;

  // state
  let state = {
    auth: false,
    grid: [],
  };

  const fetchGrid = async (): Promise<ApiGrid> => {
    const {} = await fetch("localhost:8080");
    return {};
  };

  const clickAt = async (coords: AxialCoords): Promise<ApiTile> => {
    const response = await fetch(fullUrl(`/tile/${coords.q}/${coords.r}`), {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify({
        user_id: "test",
        strength: 1,
      }),
    });

    const data = (await response.json()) as ApiTile;

    return data;
  };

  return {
    fetchGrid,
    clickAt,
  };
}
