import { initApi } from "./api";
import { login, setLoading } from "./login";
import { render } from "./render";

(async () => {
  const api = initApi();

  setLoading(true);

  const gameData = await api.fetchGameData();

  setLoading(false);

  login(api, () => {
    setLoading(false);
    render(gameData, api);
  });
})();
