use std::collections::HashMap;
use std::fs;
use futures_core::future::BoxFuture;
use sqlx::{query, Error, Executor, Pool, Row, Sqlite, error::BoxDynError, sqlite::SqliteQueryResult, migrate::{Migration as SqlxMigration, MigrateDatabase, MigrationSource, MigrationType, Migrator}};
use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex};
use crate::commands::settings::GlobalSettings;
use crate::utils::repo_manager::{setup_official_repository, LauncherInstall, LauncherManifest, LauncherRepository};
use crate::utils::{run_async_command};

pub async fn init_db(app: &AppHandle) {
    let data_path = app.path().app_data_dir().unwrap();
    let conn_path = app.path().app_config_dir().unwrap();
    let conn_url = conn_path.join("storage.db");

    let manifests_dir = data_path.join("manifests");

    if !conn_url.exists() {
        fs::create_dir_all(&conn_path).unwrap();

        if !Sqlite::database_exists(conn_url.to_str().unwrap()).await.unwrap() {
            Sqlite::create_database(conn_url.to_str().unwrap()).await.unwrap();
            #[cfg(debug_assertions)]
            { println!("Database does not exist... Creating new one for you!"); }
        }
    }

    let migrationsl = vec![
        Migration {
            version: 1,
            description: "init_repository_table",
            sql: r#"CREATE TABLE "repository" ("id" string PRIMARY KEY,"github_id" string);"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "init_manifest_table",
            sql: r#"CREATE TABLE manifest ("id" string PRIMARY KEY, "repository_id" string, "display_name" string, "filename" string, "enabled" bool, CONSTRAINT fk_manifest_repo FOREIGN KEY(repository_id) REFERENCES repository(id));"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "init_install_table",
            sql: r#"CREATE TABLE install ("id" TEXT PRIMARY KEY, "manifest_id" TEXT, "version" TEXT, "name" TEXT, "directory" TEXT, "runner_path" TEXT, "dxvk_path" TEXT, "runner_version" TEXT, "dxvk_version" TEXT, "game_icon" TEXT, "game_background" TEXT, "ignore_updates" bool, "skip_hash_check" bool, "use_jadeite" bool, "use_xxmi" bool, "use_fps_unlock" bool, "env_vars" TEXT, "pre_launch_command" TEXT, "launch_command" TEXT, "fps_value" TEXT, CONSTRAINT fk_install_manifest FOREIGN KEY(manifest_id) REFERENCES manifest(id));"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "init_settings_table",
            sql: r#"CREATE TABLE settings ("default_game_path" string default null, "third_party_repo_updates" bool default 0 not null, "xxmi_path" string default null,fps_unlock_path string default null,jadeite_path string default null, id integer not null CONSTRAINT settings_pk primary key autoincrement);"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "populate_settings_table",
            sql: r#"INSERT INTO settings (default_game_path, third_party_repo_updates, xxmi_path, fps_unlock_path, jadeite_path, id) values (null,false, null, null, null, 1);"#,
            kind: MigrationKind::Up,
        }
    ];

    let mut migrations = add_migrations("db", migrationsl);

    let instances = DbInstances::default();
    let mut tmp = instances.0.lock().await;
    let pool: Pool<Sqlite> = Pool::connect(&conn_url.to_str().unwrap()).await.unwrap();

    tmp.insert(String::from("db"), pool.clone());

    if let Some(migrations) = migrations.as_mut().unwrap().remove("db") {
        let migrator = Migrator::new(migrations).await.unwrap();
        migrator.run(&pool).await.unwrap();
    }

    drop(tmp);
    app.manage(instances);

    // Init and setup default paths...
    let defgpath = data_path.join("games");
    let xxmipath = data_path.join("extras").join("xxmi");
    let fpsunlockpath = data_path.join("extras").join("fps_unlock");
    let jadeitepath = data_path.join("extras").join("jadeite");

    if !defgpath.exists() {
        fs::create_dir_all(&defgpath).unwrap();
        query("UPDATE settings SET 'default_game_path' = $1 WHERE id = 1;").bind(defgpath.as_path().to_str().unwrap()).execute(&pool).await.unwrap();
    }

    if !xxmipath.exists() {
        fs::create_dir_all(&xxmipath).unwrap();
        query("UPDATE settings SET 'xxmi_path' = $1 WHERE id = 1;").bind(xxmipath.as_path().to_str().unwrap()).execute(&pool).await.unwrap();
    }

    if !fpsunlockpath.exists() {
        fs::create_dir_all(&fpsunlockpath).unwrap();
        query("UPDATE settings SET 'fps_unlock_path' = $1 WHERE id = 1;").bind(fpsunlockpath.as_path().to_str().unwrap()).execute(&pool).await.unwrap();
    }

    if !jadeitepath.exists() {
        fs::create_dir_all(&jadeitepath).unwrap();
        query("UPDATE settings SET 'jadeite_path' = $1 WHERE id = 1;").bind(jadeitepath.as_path().to_str().unwrap()).execute(&pool).await.unwrap();
    }

    // Init this fuck AFTER you add shitty DB instances to state
    if !manifests_dir.exists() {
        fs::create_dir_all(&manifests_dir).unwrap();
        #[cfg(debug_assertions)]
        { println!("Manifests directory does not exist... Creating new one for you!"); }
        setup_official_repository(&app, &manifests_dir);
    } else {
        setup_official_repository(&app, &manifests_dir);
    }
}


// === SETTINGS ===

pub fn get_settings(app: &AppHandle) -> Option<GlobalSettings> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM settings WHERE id = 1");
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = GlobalSettings {
            default_game_path: rslt.get(0).unwrap().get("default_game_path"),
            xxmi_path: rslt.get(0).unwrap().get("xxmi_path"),
            fps_unlock_path: rslt.get(0).unwrap().get("fps_unlock_path"),
            jadeite_path: rslt.get(0).unwrap().get("jadeite_path"),
            third_party_repo_updates: rslt.get(0).unwrap().get("third_party_repo_updates"),
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn update_settings_third_party_repo_update(app: &AppHandle, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'third_party_repo_updates' = $1 WHERE id = 1").bind(enabled);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_game_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'default_game_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_xxmi_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'xxmi_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_fps_unlock_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'fps_unlock_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_jadeite_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'jadeite_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

// === REPOSITORIES ===

pub fn create_repository(app: &AppHandle, id: String, github_id: &str) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("INSERT INTO repository(id, github_id) VALUES ($1, $2)").bind(id).bind(github_id);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn delete_repository_by_id(app: &AppHandle, id: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("DELETE FROM repository WHERE id = $1").bind(id);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn get_repository_info_by_id(app: &AppHandle, id: String) -> Option<LauncherRepository> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM repository WHERE id = $1").bind(id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = LauncherRepository {
            id: rslt.get(0).unwrap().get("id"),
            github_id: rslt.get(0).unwrap().get("github_id"),
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_repository_info_by_github_id(app: &AppHandle, github_id: String) -> Option<LauncherRepository> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM repository WHERE github_id = $1").bind(github_id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = LauncherRepository {
            id: rslt.get(0).unwrap().get("id"),
            github_id: rslt.get(0).unwrap().get("github_id"),
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_repositories(app: &AppHandle) -> Option<Vec<LauncherRepository>> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM repository");
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let mut rsltt = Vec::<LauncherRepository>::new();
        for r in rslt {
            rsltt.push(LauncherRepository {
                id: r.get("id"),
                github_id: r.get("github_id"),
            })
        }

        Some(rsltt)
    } else {
        None
    }
}

// === MANIFESTS ===

pub fn create_manifest(app: &AppHandle, id: String, repository_id: String, display_name: &str, filename: &str, enabled: bool) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        rslt = db.execute(format!("INSERT INTO manifest(id, repository_id, display_name, filename, enabled) VALUES ('{id}', '{repository_id}', '{display_name}', '{filename}', {enabled})").as_str()).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn delete_manifest_by_repository_id(app: &AppHandle, repository_id: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("DELETE FROM manifest WHERE repository_id = $1").bind(repository_id);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn delete_manifest_by_id(app: &AppHandle, id: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("DELETE FROM manifest WHERE id = $1").bind(id);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn get_manifest_info_by_id(app: &AppHandle, id: String) -> Option<LauncherManifest> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM manifest WHERE id = $1").bind(id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = LauncherManifest {
            id: rslt.get(0).unwrap().get("id"),
            repository_id: rslt.get(0).unwrap().get("repository_id"),
            display_name: rslt.get(0).unwrap().get("display_name"),
            filename: rslt.get(0).unwrap().get("filename"),
            enabled: rslt.get(0).unwrap().get("enabled")
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_manifest_info_by_filename(app: &AppHandle, filename: String) -> Option<LauncherManifest> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM manifest WHERE filename = $1").bind(filename);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = LauncherManifest {
            id: rslt.get(0).unwrap().get("id"),
            repository_id: rslt.get(0).unwrap().get("repository_id"),
            display_name: rslt.get(0).unwrap().get("display_name"),
            filename: rslt.get(0).unwrap().get("filename"),
            enabled: rslt.get(0).unwrap().get("enabled")
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_manifests_by_repository_id(app: &AppHandle, repository_id: String) -> Option<Vec<LauncherManifest>> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM manifest WHERE repository_id = $1").bind(repository_id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let mut rsltt = Vec::<LauncherManifest>::new();
        for r in rslt {
            rsltt.push(LauncherManifest {
                id: r.get("id"),
                repository_id: r.get("repository_id"),
                display_name: r.get("display_name"),
                filename: r.get("filename"),
                enabled: r.get("enabled")
            })
        }

        Some(rsltt)
    } else {
        None
    }
}

pub fn update_manifest_enabled_by_id(app: &AppHandle, id: String, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE manifest SET 'enabled' = $1 WHERE id = $2").bind(enabled).bind(id);
        query.execute(&db).await.unwrap();
    });
}

// === INSTALLS ===

pub fn create_installation(app: &AppHandle, id: String, manifest_id: String, version: String, name: String, directory: String, runner_path: String, dxvk_path: String, runner_version: String, dxvk_version: String, game_icon: String, game_background: String, ignore_updates: bool, skip_hash_check: bool, use_jadeite: bool, use_xxmi: bool, use_fps_unlock: bool, env_vars: String, pre_launch_command: String, launch_command: String, fps_value: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("INSERT INTO install(id, manifest_id, version, name, directory, runner_path, dxvk_path, runner_version, dxvk_version, game_icon, game_background, ignore_updates, skip_hash_check, use_jadeite, use_xxmi, use_fps_unlock, env_vars, pre_launch_command, launch_command, fps_value) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)").bind(id).bind(manifest_id).bind(version).bind(name).bind(directory).bind(runner_path).bind(dxvk_path).bind(runner_version).bind(dxvk_version).bind(game_icon).bind(game_background).bind(ignore_updates).bind(skip_hash_check).bind(use_jadeite).bind(use_xxmi).bind(use_fps_unlock).bind(env_vars).bind(pre_launch_command).bind(launch_command).bind(fps_value);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn delete_installation_by_id(app: &AppHandle, id: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("DELETE FROM install WHERE id = $1").bind(id);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn get_install_info_by_id(app: &AppHandle, id: String) -> Option<LauncherInstall> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM install WHERE id = $1").bind(id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = LauncherInstall {
            id: rslt.get(0).unwrap().get("id"),
            manifest_id: rslt.get(0).unwrap().get("manifest_id"),
            version: rslt.get(0).unwrap().get("version"),
            name: rslt.get(0).unwrap().get("name"),
            directory: rslt.get(0).unwrap().get("directory"),
            runner_path: rslt.get(0).unwrap().get("runner_path"),
            dxvk_path: rslt.get(0).unwrap().get("dxvk_path"),
            runner_version: rslt.get(0).unwrap().get("runner_version"),
            dxvk_version: rslt.get(0).unwrap().get("dxvk_version"),
            game_icon: rslt.get(0).unwrap().get("game_icon"),
            game_background: rslt.get(0).unwrap().get("game_background"),
            ignore_updates: rslt.get(0).unwrap().get("ignore_updates"),
            skip_hash_check: rslt.get(0).unwrap().get("skip_hash_check"),
            use_jadeite: rslt.get(0).unwrap().get("use_jadeite"),
            use_xxmi: rslt.get(0).unwrap().get("use_xxmi"),
            use_fps_unlock: rslt.get(0).unwrap().get("use_fps_unlock"),
            env_vars: rslt.get(0).unwrap().get("env_vars"),
            pre_launch_command: rslt.get(0).unwrap().get("pre_launch_command"),
            launch_command: rslt.get(0).unwrap().get("launch_command"),
            fps_value: rslt.get(0).unwrap().get("fps_value")
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_installs_by_manifest_id(app: &AppHandle, manifest_id: String) -> Option<Vec<LauncherInstall>> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM install WHERE manifest_id = $1").bind(manifest_id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let mut rsltt = Vec::<LauncherInstall>::new();
        for r in rslt {
            rsltt.push(LauncherInstall {
                id: r.get("id"),
                manifest_id: r.get("manifest_id"),
                version: r.get("version"),
                name: r.get("name"),
                directory: r.get("directory"),
                runner_path: r.get("runner_path"),
                dxvk_path: r.get("dxvk_path"),
                runner_version: r.get("runner_version"),
                dxvk_version: r.get("dxvk_version"),
                game_icon: r.get("game_icon"),
                game_background: r.get("game_background"),
                ignore_updates: r.get("ignore_updates"),
                skip_hash_check: r.get("skip_hash_check"),
                use_jadeite: r.get("use_jadeite"),
                use_xxmi: r.get("use_xxmi"),
                use_fps_unlock: r.get("use_fps_unlock"),
                env_vars: r.get("env_vars"),
                pre_launch_command: r.get("pre_launch_command"),
                launch_command: r.get("launch_command"),
                fps_value: r.get("fps_value")
            })
        }

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_installs(app: &AppHandle) -> Option<Vec<LauncherInstall>> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM install");
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let mut rsltt = Vec::<LauncherInstall>::new();
        for r in rslt {
            rsltt.push(LauncherInstall {
                id: r.get("id"),
                manifest_id: r.get("manifest_id"),
                version: r.get("version"),
                name: r.get("name"),
                directory: r.get("directory"),
                runner_path: r.get("runner_path"),
                dxvk_path: r.get("dxvk_path"),
                runner_version: r.get("runner_version"),
                dxvk_version: r.get("dxvk_version"),
                game_icon: r.get("game_icon"),
                game_background: r.get("game_background"),
                ignore_updates: r.get("ignore_updates"),
                skip_hash_check: r.get("skip_hash_check"),
                use_jadeite: r.get("use_jadeite"),
                use_xxmi: r.get("use_xxmi"),
                use_fps_unlock: r.get("use_fps_unlock"),
                env_vars: r.get("env_vars"),
                pre_launch_command: r.get("pre_launch_command"),
                launch_command: r.get("launch_command"),
                fps_value: r.get("fps_value")
            })
        }

        Some(rsltt)
    } else {
        None
    }
}

// === DB RELATED ===

fn add_migrations(db_url: &str, migrations: Vec<Migration>) -> Option<HashMap<String, MigrationList>> {
    let mut migrs: Option<HashMap<String, MigrationList>> = Some(HashMap::new());

    migrs.get_or_insert(Default::default()).insert(db_url.to_string(), MigrationList(migrations));
    migrs
}

#[derive(Default, Debug)]
pub struct DbInstances(pub Mutex<HashMap<String, Pool<Sqlite>>>);

#[derive(Debug)]
pub enum MigrationKind {
    Up,
    Down,
}

impl From<MigrationKind> for MigrationType {
    fn from(kind: MigrationKind) -> Self {
        match kind {
            MigrationKind::Up => Self::ReversibleUp,
            MigrationKind::Down => Self::ReversibleDown,
        }
    }
}

/// A migration definition.
#[derive(Debug)]
pub struct Migration {
    pub version: i64,
    pub description: &'static str,
    pub sql: &'static str,
    pub kind: MigrationKind,
}

#[derive(Debug)]
struct MigrationList(Vec<Migration>);

impl MigrationSource<'static> for MigrationList {
    fn resolve(self) -> BoxFuture<'static, Result<Vec<SqlxMigration>, BoxDynError>> {
        Box::pin(async move {
            let mut migrations = Vec::new();
            for migration in self.0 {
                if matches!(migration.kind, MigrationKind::Up) {
                    migrations.push(SqlxMigration::new(
                        migration.version,
                        migration.description.into(),
                        migration.kind.into(),
                        migration.sql.into(),
                    ));
                }
            }
            Ok(migrations)
        })
    }
}