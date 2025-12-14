const { invoke } = window.__TAURI__.core;
async function launchGame() {
  const username = document.querySelector("#player_username").value;
  const version = document.querySelector("#versions").value

  if (!username || !version) {
    throw new Error("Not implemented yet")
  }

  await invoke("launch_game", {username: username, version: version});
}

async function getVersions() {
  const versionElement = document.querySelector("#versions")

  let versions = await invoke("get_versions")
  versions.versions.forEach((version) => {
    let option = document.createElement("option");
    option.innerText = version.id;
    option.value = version.id;
    versionElement.appendChild(option);
  })
}

async function getVersionsTypes() {
  const versionTypeParent = document.querySelector("#versions_types")
  let versions_types = await invoke("get_versions_types")
  versions_types.forEach((type) => {
    let inputLabel = document.createElement("label");
    inputLabel.for = type
    inputLabel.innerText = type
    let input = document.createElement("input");
    input.type = "checkbox";
    input.name = type;
    input.value = type;
    input.id = type
    versionTypeParent.appendChild(inputLabel);
    versionTypeParent.appendChild(input);
  })
}

window.addEventListener("DOMContentLoaded", () => {
  getVersions()
  getVersionsTypes()
  document.querySelector("#launch-game").addEventListener("submit", (e) => {
    e.preventDefault();
    launchGame()
  });
});


