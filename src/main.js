const { invoke } = window.__TAURI__.tauri;

let login_ipEl;
let login_usernameEl;
let login_passwordEl;
let login_roomEl;
let loginMsgEl;

async function send_login() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    console.log("Hello from Tauri!");
    loginMsgEl.textContent = await invoke("send_login", { ip: login_ipEl.value, username: login_usernameEl.value, password: login_passwordEl.value, roomNumber: login_roomEl.value });
}

async function copy_key() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    invoke("get_room_key", {});
}

window.addEventListener("DOMContentLoaded", () => {
    login_ipEl = document.querySelector("#login-ip");
    login_usernameEl = document.querySelector("#login-username");
    login_passwordEl = document.querySelector("#login-password");
    login_roomEl = document.querySelector("#login-room");
    loginMsgEl = document.querySelector("#login-msg");
    document
        .querySelector("#login-button")
        .addEventListener("click", () => send_login());
});
window.addEventListener("DOMContentLoaded", () => {
    document
        .querySelector("#copy-key")
        .addEventListener("click", () => copy_key());
});
window.addEventListener("DOMContentLoaded", () => {
    loginMsgEl = document.querySelector("#msg-content");
    document
        .querySelector("#send-msg")
        .addEventListener("click", () => send_msg());
});