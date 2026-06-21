// trouble with naming stuff

import { invoke } from "@tauri-apps/api/core";

export async function setMinimizeToTray(value: boolean) {
    const rootJson = JSON.parse(await invoke("get_local_app", { vng: false }));

    const appConfig = JSON.parse(rootJson.AppConfiguration);

    appConfig.MinimizeToTray = value;

    rootJson.AppConfiguration = JSON.stringify(appConfig);

    await invoke("write_local_app", {
        content: JSON.stringify(rootJson),
        vng: false
    });
}


export async function setLaunchAtStartup(value: boolean) {
    const rootJson = JSON.parse(await invoke("get_local_app", { vng: false }));

    const appConfig = JSON.parse(rootJson.AppConfiguration);

    appConfig.LaunchAtStartup = value;

    rootJson.AppConfiguration = JSON.stringify(appConfig);

    await invoke("write_local_app", {
        content: JSON.stringify(rootJson),
        vng: false
    });
}