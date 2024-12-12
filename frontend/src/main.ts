import { initApi } from "./api";
import { login } from "./login";
import { render } from "./render";

const api = initApi();

login(api, async (gameData) => {
  render(gameData, api);
});
