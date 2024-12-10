interface ApiGrid {
  // TODO
}

function initApi() {
  // state
  let state = {
    auth: false,
    grid: [],
  };

  const fetchGrid = async (): Promise<ApiGrid> => {
    const {} = await fetch("localhost:8080");
    return {};
  };

  return {
    fetchGrid,
  };
}

/** API wrapper to ease usage */
export const api = initApi();
