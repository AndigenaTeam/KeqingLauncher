import TextInputPart from "./TextInputPart.tsx";
import {invoke} from "@tauri-apps/api/core";

export default function TextInput({ id, name, value, placeholder, readOnly, pattern, install, fetchInstallSettings}: { id: string, name: string, value: string, placeholder?: string, readOnly: boolean, pattern?: string, install?: string, fetchInstallSettings?: (id: string) => void }) {

    return (
        <div className="flex flex-row items-center justify-between w-full h-8">
            <span className="text-white text-sm">{name}</span>
            <div className={"overflow-ellipsis inline-flex flex-row items-center justify-center"}>
                <TextInputPart id={id} initalValue={value} placeholder={placeholder} readOnly={readOnly} isPicker={false} pattern={pattern} onChange={(e) => {
                    switch (id) {
                        case "install_env_vars": {
                            if (fetchInstallSettings !== undefined) {
                                // god please have mercy this should do the sort of okayish job for anything user types for a format of env vars...
                                const regex = /^[a-zA-Z0-9_-]+=(?:\S(?:[a-zA-Z0-9\s]*\S)?|"(?:[^"\s\\]|\\.)+"|'(?:[^'\s\\]|\\.)+')(?:;[a-zA-Z0-9_-]+=(?:\S(?:[a-zA-Z0-9\s]*\S)?|"(?:[^"\s\\]|\\.)+"|'(?:[^'\s\\]|\\.)+'))*;?$/gi;
                                if (regex.test(e) || e === "") {
                                    invoke("update_install_env_vars", {envVars: `${e}`, id: install}).then(() => {
                                        fetchInstallSettings(install as string)
                                    });
                                }
                            }
                        }
                        break;
                        case "install_pre_launch_cmd": {
                            if (fetchInstallSettings !== undefined) {
                                invoke("update_install_pre_launch_cmd", {cmd: `${e}`, id: install}).then(() => {
                                    fetchInstallSettings(install as string)
                                });
                            }
                        }
                        break;
                        case "install_launch_cmd": {
                            if (fetchInstallSettings !== undefined) {
                                invoke("update_install_launch_cmd", {cmd: `${e}`, id: install}).then(() => {
                                    fetchInstallSettings(install as string)
                                });
                            }
                        }
                        break;
                    }
                }} />
            </div>
        </div>
    )
}
