import { createApp } from "vue";
import App from "./App.vue";
import { initializeAppearance } from "./appearance";
import "./style.css";

async function start() {
  await initializeAppearance();
  createApp(App).mount("#app");
}

void start().catch((error: unknown) => {
  const message = document.createElement("p");
  message.setAttribute("role", "alert");
  message.className = "m-0 h-full bg-dc-surface p-3 text-[12px] text-dc-text-primary";
  message.textContent = `DropCove could not initialize its appearance: ${String(error)}`;
  document.querySelector("#app")?.append(message);
});
