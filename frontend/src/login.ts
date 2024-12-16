import { GameApi } from "./api";

type OnLoggedSuccess = () => void;

export function setLoading(loading = true) {
  const login = document.querySelector("#login");

  login?.addEventListener("click", (e) => e.stopPropagation());
  if (loading) {
    login?.classList.add("loading");
  } else {
    login?.classList.remove("loading");
  }
}

export function login(api: GameApi, onLogged: OnLoggedSuccess) {
  const login = document.querySelector("#login");
  const form = login?.querySelector("#login > form");

  if (!form || !login) {
    throw new Error("Cannot get form (#login > form)");
  }

  const input = form.querySelector("#username") as HTMLInputElement;

  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    e.stopPropagation();

    setLoading(true);

    const value = input.value;
    // TODO: validation & error handling
    await api.login(value);

    login.classList.add("success");
    onLogged();
  });
}
