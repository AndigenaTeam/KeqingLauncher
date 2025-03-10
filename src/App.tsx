import "./App.css";
import React from "react";
import RepoManager from "./components/popups/repomanager/RepoManager.tsx";
import {POPUPS} from "./components/popups/POPUPS.ts";
import AddRepo from "./components/popups/addrepo/AddRepo.tsx";
import SidebarIcon from "./components/SidebarIcon.tsx";
import {invoke} from "@tauri-apps/api/core";
import SidebarRepos from "./components/SidebarRepos.tsx";
import {Rocket, Settings} from "lucide-react";
import SidebarSettings from "./components/SidebarSettings.tsx";
import SettingsManager from "./components/popups/settings/SettingsManager.tsx";

export default class App extends React.Component<any, any> {
    constructor(props: any) {
        super(props);

        this.setCurrentGame = this.setCurrentGame.bind(this);
        this.setDisplayName = this.setDisplayName.bind(this);
        this.setBackground = this.setBackground.bind(this);
        this.setReposList = this.setReposList.bind(this);

        this.setOpenPopup = this.setOpenPopup.bind(this);

        this.pushGames = this.pushGames.bind(this);
        this.pushGamesInfo = this.pushGamesInfo.bind(this);

        this.state = {
            openPopup: POPUPS.NONE,
            currentGame: "",
            displayName: "",
            gameBackground: "",
            games: [],
            gamesinfo: [],
            repos: [],
            reposList: {},
        }
    }

    render() {
        return (
            <main className="w-full h-screen flex flex-row bg-transparent">
                <img className="w-full h-screen object-cover object-center absolute top-0 left-0 right-0 bottom-0 -z-10" alt={"?"} src={this.state.gameBackground} />
                <div className="h-full w-16 p-2 bg-black/50 flex flex-col gap-4 items-center fixed-backdrop-blur-md justify-between">
                    <div className="flex flex-col gap-4 flex-shrink overflow-scroll scrollbar-none">
                        {this.state.currentGame != "" && this.state.gamesinfo.map((game: { manifest_enabled: boolean; assets: any; filename: string; icon: string; display_name: string; biz: string; }) => {
                            return (
                                <SidebarIcon key={game.biz} popup={this.state.openPopup} icon={game.assets.game_icon} background={game.assets.game_background} name={game.display_name} enabled={game.manifest_enabled} id={game.biz} setCurrentGame={this.setCurrentGame} setOpenPopup={this.setOpenPopup} setDisplayName={this.setDisplayName} setBackground={this.setBackground} />
                            )
                        })}
                    </div>
                    <div className="flex flex-col gap-4 flex-shrink overflow-scroll scrollbar-none">
                        <SidebarRepos popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} />
                        <SidebarSettings popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} />
                    </div>
                </div>
                <div className="flex flex-row absolute bottom-8 right-16 gap-4">
                    <button>
                        <Settings className="text-white w-8 h-8" />
                    </button>
                    <button className="flex flex-row gap-2 items-center py-2 px-4 bg-blue-600 rounded-lg" onClick={() => {
                        this.setState({openPopup: POPUPS.ADDREPO});
                    }}><Rocket/>
                        <span className="font-semibold translate-y-px">Launch!</span>
                    </button>
                </div>

                <div className={`absolute items-center justify-center top-0 bottom-0 left-16 right-0 p-8 z-20 ${this.state.openPopup == POPUPS.NONE ? "hidden" : "flex fixed-backdrop-blur-lg bg-white/10"}`}>
                    {this.state.openPopup == POPUPS.REPOMANAGER && <RepoManager repos={this.state.reposList} setOpenPopup={this.setOpenPopup} />}
                    {this.state.openPopup == POPUPS.ADDREPO && <AddRepo setOpenPopup={this.setOpenPopup}/>}
                    {this.state.openPopup == POPUPS.SETTINGS && <SettingsManager setOpenPopup={this.setOpenPopup} />}

                </div>
            </main>
        )
    }

    componentDidMount() {
        invoke("list_repositories").then(r => {
            if (r === null) {
                this.setState(() => ({repos: []}));
            } else {
                this.setState(() => ({repos: JSON.parse(r as string)}), () => {
                    this.pushGames(this.state.repos);
                });
            }
        }).catch(e => {
            console.error("Error while listing database repositories information: " + e)
        });
    }

    pushGames(repos: { id: string; github_id: any; }[]) {
        repos.forEach((r: { id: string; github_id: any; }) => {
            invoke("list_manifests_by_repository_id", { repositoryId: r.id }).then(m => {
                if (m === null) {
                    console.error("Manifest database table contains nothing, some serious fuck up happened!")
                } else {
                    let g = JSON.parse(m as string);
                    this.setState(() => ({games: g}), () => {
                        this.pushGamesInfo(this.state.games);
                        let entries: any[] = [];
                        this.state.games.forEach((e: any) => entries.push(e));
                        // @ts-ignore
                        r["manifests"] = entries;
                        this.setReposList(repos);
                    });
                }
            }).catch(e => {
                console.error("Error while listing database manifest information: " + e)
            })
        });
    }

    pushGamesInfo(games: { filename: any; display_name: string; id: string; enabled: boolean; }[]) {
        invoke("list_game_manifests").then(m => {
            if (m === null) {
                console.error("GameManifest repository fetch issue, some serious fuck up happened!")
            } else {
                let gi = JSON.parse(m as string);
                // Hacky way to pass some values from DB manifest data onto the list of games we use to render SideBarIcon components
                gi.forEach((e: any) => {
                  let g = games.find(g => g.filename.replace(".json", "") === e.biz);
                  // @ts-ignore
                    e["manifest_id"] = g.id;
                  // @ts-ignore
                    e["manifest_enabled"] = g.enabled;
                  // @ts-ignore
                    e["manifest_file"] = g.filename;
                });

                this.setState(() => ({gamesinfo: gi}), () => {
                    if (games.length > 0 && this.state.currentGame == "") {
                        this.setCurrentGame(games[0].id);
                        this.setDisplayName(games[0].display_name)
                        this.setBackground(gi[0].assets.game_background);
                    }
                });
            }
        }).catch(e => {
            console.error("Error while listing game manifest information: " + e)
        })
    }

    setOpenPopup(state: POPUPS) {
        this.setState({openPopup: state});
    }

    setCurrentGame(game: string) {
        this.setState({currentGame: game});
    }

    setDisplayName(name: string) {
        this.setState({displayName: name});
    }

    setBackground(file: string) {
        this.setState({gameBackground: file});
    }

    setReposList(reposList: any) {
        this.setState({reposList: reposList});
    }
}