const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;
let serverPortEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  serverPortEl = document.querySelector("#server-port");
  invoke("server_port").then((port) => {
    serverPortEl.textContent = `Server running on port ${port}`;
  });
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});
