import {DownloadCloudIcon, X} from "lucide-react";
import {POPUPS} from "./POPUPS.ts";
import FolderInput from "../common/FolderInput.tsx";
import CheckBox from "../common/CheckBox.tsx";
import TextDisplay from "../common/TextDisplay.tsx";
import SelectMenu from "../common/SelectMenu.tsx";

export default function DownloadGame({setOpenPopup, displayName, settings, biz, versions}: {versions: any, settings: any, biz: any, displayName: string, setOpenPopup: (popup: POPUPS) => void}) {

    return (
        <div className="rounded-lg h-3/4 w-2/4 flex flex-col p-4 gap-8 overflow-scroll">
            <div className="flex flex-row items-center justify-between">
                <h1 className="text-white font-bold text-2xl">Download {displayName}</h1>
                <X className="text-white cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>
            <div className="flex flex-row-reverse">
                <button className="flex flex-row gap-1 items-center p-2 bg-blue-600 rounded-lg" onClick={() => {
                    setOpenPopup(POPUPS.NONE)
                    // @ts-ignore
                    console.log("skip hash:", Boolean(document.getElementById("skip_hash_validation").checked));
                    // @ts-ignore
                    console.log("skip version:", Boolean(document.getElementById("skip_version_updates").checked));
                    // @ts-ignore
                    console.log("install path:", document.getElementById("install_game_path").value);
                    // @ts-ignore
                    let gv = document.getElementById("game_version");
                    // @ts-ignore
                    console.log("install version:", gv.options[gv.selectedIndex].value);
                }}>
                    <DownloadCloudIcon className=""/>
                    <span className="font-semibold translate-y-px">Start download</span>
                </button>
            </div>
                <div className={`w-full transition-all duration-500 overflow-hidden bg-neutral-700 gap-4 flex flex-col items-center justify-between px-4 p-4 rounded-b-lg rounded-t-lg`} style={{maxHeight: (20 * 64) + "px"}}>
                    <FolderInput name={"Install location"} clearable={true} value={`${settings.default_game_path}/${biz}`} folder={true} id={"install_game_path"}/>
                    <CheckBox enabled={false} name={"Skip version update check"} id={"skip_version_updates"}/>
                    <CheckBox enabled={false} name={"Skip hash validation"} id={"skip_hash_validation"}/>
                    <TextDisplay name={"Available disk space"} value={"33"} style={"text-white px-3"}/>
                    <TextDisplay name={"Required disk space"} value={"10"} style={"text-white px-3"}/>
                    <SelectMenu id={"game_version"} name={"Game version"} options={versions}/>
                </div>
            </div>
    )
}
