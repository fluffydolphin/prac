const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

let sendmessageEl;


function listenfunc(event) {
    const myStruct = JSON.parse(event.payload);
    const { output_user, output } = myStruct;

    const parentDiv1 = document.getElementById('divmessages');
    const newDiv1 = document.createElement('div');
    newDiv1.classList.add('messagereceived1');
    const newP1 = document.createElement('p');
    newP1.textContent = output_user;
    newDiv1.appendChild(newP1);
    parentDiv1.appendChild(newDiv1);

    const parentDiv = document.getElementById('divmessages');
    const newDiv = document.createElement('div');
    newDiv.classList.add('messagereceived');
    const newP = document.createElement('p');
    newP.textContent = output;
    newDiv.appendChild(newP);
    parentDiv.appendChild(newDiv);
    parentDiv.scrollTop = parentDiv.scrollHeight;
}

function send_msg() {
    const message = sendmessageEl.value;
    if (message.trim() == "") {
        return
    }

    invoke("send_msg", { msg: message });

    const input = document.getElementById("msg-content");
    input.value = '';
    const parentDiv = document.getElementById('divmessages');
    const newDiv = document.createElement('div');
    newDiv.classList.add('messagesent');
    const newP = document.createElement('p');
    newP.textContent = message;
    newDiv.appendChild(newP);
    parentDiv.appendChild(newDiv);
    parentDiv.appendChild(newDiv);
    parentDiv.scrollTop = parentDiv.scrollHeight;
}

const unlisten = listen("listen", listenfunc);

function copy_key() {
    invoke("get_room_key", {});
}
const form = document.querySelector('form');
const inputField = document.querySelector('#msg-content');

inputField.addEventListener('keydown', function(event) {
    if (event.key === 'Enter') {
        event.preventDefault();
        document.getElementById('send-msgz').click();
    }
});
window.addEventListener("DOMContentLoaded", () => {
    sendmessageEl = document.querySelector("#msg-content");
    document
        .querySelector("#send-msgz")
        .addEventListener("click", () => send_msg());
});
window.addEventListener("DOMContentLoaded", () => {
    document
        .querySelector("#copy-key")
        .addEventListener("click", () => copy_key());
});