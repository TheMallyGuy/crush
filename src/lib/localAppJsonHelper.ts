// trouble with naming stuff

import { invoke } from "@tauri-apps/api/core";

export async function setMinimizeToTray(value: boolean) {
    const storage = JSON.parse(await invoke("get_local_app", { vng: false }));

    storage.MinimizeToTray = value;

    await invoke("write_local_app", {
        content: JSON.stringify(storage),
        vng: false
    });
}


export async function setLaunchAtStartup(value: boolean) {
    const storage = JSON.parse(await invoke("get_local_app", { vng: false }));

    storage.LaunchAtStartup = value;

    await invoke("write_local_app", {
        content: JSON.stringify(storage),
        vng: false
    });
}