import { initApi } from "./api";
import { login, setLoading } from "./login";
import { render } from "./render";

(async () => {
  const api = initApi();

  setLoading(true);

  render({
    api,
    onReady: () => {
      setLoading(false);
      login(api, () => {
        setLoading(false);
      });
    },
  });
})();
