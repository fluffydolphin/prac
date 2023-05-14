const { invoke } = window.__TAURI__.tauri;

let login_ipEl;
let login_usernameEl;
let login_passwordEl;
let login_roomEl;
let loginMsgEl;

async function send_login() {
    loginMsgEl.textContent = await invoke("send_login", { domainName: login_ipEl.value, username: login_usernameEl.value, password: login_passwordEl.value, roomNumber: login_roomEl.value });
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
document.addEventListener('keydown', function(event) {
    if (event.key === 'Enter') {
        document.getElementById('login-button').click();
    }
});