import { GameApi } from "./api";

type OnLoggedSuccess = () => void;

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

    const value = input.value;
    // TODO: validation & error handling
    await api.login(value);
    login.classList.add("success");
    onLogged();
  });
}
