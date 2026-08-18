use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use rusqlite::{params, Connection, OptionalExtension};
use thiserror::Error;

use crate::media::{LibraryScan, ScannedLibrary};

const SCHEMA_VERSION: i64 = 3;

#[derive(Debug)]
pub(crate) struct LibraryDatabase {
    connection: Mutex<Connection>,
}

#[derive(Debug)]
pub(crate) struct MediaLocation {
    pub root_path: PathBuf,
    pub media_path: PathBuf,
    pub extension: String,
}

#[derive(Debug)]
pub(crate) struct TrustedPeerRecord {
    pub id: String,
    pub display_name: String,
    pub capability: String,
    pub created_at: i64,
    pub revoked: bool,
}

#[derive(Debug)]
pub(crate) struct BrowserSessionRecord {
    pub peer_id: String,
    pub display_name: String,
    pub capability: String,
    pub expires_at: i64,
    pub session_revoked: bool,
    pub peer_revoked: bool,
}

pub(crate) struct NewBrowserSession<'a> {
    pub peer_id: &'a str,
    pub display_name: &'a str,
    pub peer_token_digest: &'a [u8; 32],
    pub capability: &'a str,
    pub session_digest: &'a [u8; 32],
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("the LocalStream database is unavailable")]
    Unavailable,
    #[error("the LocalStream database contains invalid text data")]
    InvalidText,
}

impl LibraryDatabase {
    pub(crate) fn open(path: &Path) -> Result<Self, DatabaseError> {
        let connection = Connection::open(path).map_err(|_| DatabaseError::Unavailable)?;
        Self::initialize(connection)
    }

    #[cfg(test)]
    pub(crate) fn in_memory() -> Result<Self, DatabaseError> {
        let connection = Connection::open_in_memory().map_err(|_| DatabaseError::Unavailable)?;
        Self::initialize(connection)
    }

    fn initialize(connection: Connection) -> Result<Self, DatabaseError> {
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|_| DatabaseError::Unavailable)?;
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|_| DatabaseError::Unavailable)?;

        match version {
            0 => connection
                .execute_batch(
                    "BEGIN IMMEDIATE;
                 CREATE TABLE libraries (
                   id TEXT PRIMARY KEY,
                   name TEXT NOT NULL,
                   root_path TEXT NOT NULL UNIQUE,
                   skipped_entries INTEGER NOT NULL DEFAULT 0
                 );
                 CREATE TABLE media_items (
                   id TEXT PRIMARY KEY,
                   library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
                   path TEXT NOT NULL UNIQUE,
                   title TEXT NOT NULL,
                   extension TEXT NOT NULL,
                   size_bytes INTEGER NOT NULL
                 );
                 CREATE TABLE app_state (
                   singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                   current_library_id TEXT REFERENCES libraries(id) ON DELETE SET NULL
                 );
                 CREATE TABLE trusted_peers (
                   id TEXT PRIMARY KEY,
                   display_name TEXT NOT NULL,
                   token_digest BLOB NOT NULL UNIQUE CHECK (length(token_digest) = 32),
                   capability TEXT NOT NULL,
                   created_at INTEGER NOT NULL,
                   revoked INTEGER NOT NULL DEFAULT 0 CHECK (revoked IN (0, 1))
                 );
                 CREATE TABLE browser_sessions (
                   token_digest BLOB PRIMARY KEY CHECK (length(token_digest) = 32),
                   peer_id TEXT NOT NULL REFERENCES trusted_peers(id) ON DELETE CASCADE,
                   capability TEXT NOT NULL,
                   created_at INTEGER NOT NULL,
                   expires_at INTEGER NOT NULL CHECK (expires_at > created_at),
                   revoked INTEGER NOT NULL DEFAULT 0 CHECK (revoked IN (0, 1))
                 );
                 CREATE INDEX browser_sessions_peer_id ON browser_sessions(peer_id);
                 INSERT INTO app_state(singleton, current_library_id) VALUES (1, NULL);
                 PRAGMA user_version = 3;
                 COMMIT;",
                )
                .map_err(|_| DatabaseError::Unavailable)?,
            1 => connection
                .execute_batch(
                    "BEGIN IMMEDIATE;
                     CREATE TABLE trusted_peers (
                       id TEXT PRIMARY KEY,
                       display_name TEXT NOT NULL,
                       token_digest BLOB NOT NULL UNIQUE CHECK (length(token_digest) = 32),
                       capability TEXT NOT NULL,
                       created_at INTEGER NOT NULL,
                       revoked INTEGER NOT NULL DEFAULT 0 CHECK (revoked IN (0, 1))
                     );
                     CREATE TABLE browser_sessions (
                       token_digest BLOB PRIMARY KEY CHECK (length(token_digest) = 32),
                       peer_id TEXT NOT NULL REFERENCES trusted_peers(id) ON DELETE CASCADE,
                       capability TEXT NOT NULL,
                       created_at INTEGER NOT NULL,
                       expires_at INTEGER NOT NULL CHECK (expires_at > created_at),
                       revoked INTEGER NOT NULL DEFAULT 0 CHECK (revoked IN (0, 1))
                     );
                     CREATE INDEX browser_sessions_peer_id ON browser_sessions(peer_id);
                     PRAGMA user_version = 3;
                     COMMIT;",
                )
                .map_err(|_| DatabaseError::Unavailable)?,
            2 => connection
                .execute_batch(
                    "BEGIN IMMEDIATE;
                     CREATE TABLE browser_sessions (
                       token_digest BLOB PRIMARY KEY CHECK (length(token_digest) = 32),
                       peer_id TEXT NOT NULL REFERENCES trusted_peers(id) ON DELETE CASCADE,
                       capability TEXT NOT NULL,
                       created_at INTEGER NOT NULL,
                       expires_at INTEGER NOT NULL CHECK (expires_at > created_at),
                       revoked INTEGER NOT NULL DEFAULT 0 CHECK (revoked IN (0, 1))
                     );
                     CREATE INDEX browser_sessions_peer_id ON browser_sessions(peer_id);
                     PRAGMA user_version = 3;
                     COMMIT;",
                )
                .map_err(|_| DatabaseError::Unavailable)?,
            SCHEMA_VERSION => {}
            _ => return Err(DatabaseError::Unavailable),
        }

        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub(crate) fn replace_library(&self, scan: &ScannedLibrary) -> Result<(), DatabaseError> {
        let root_path = scan.root_path.to_string_lossy();
        let library_id =
            uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, root_path.as_bytes()).to_string();
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?;
        let transaction = connection
            .transaction()
            .map_err(|_| DatabaseError::Unavailable)?;

        transaction
            .execute(
                "INSERT INTO libraries(id, name, root_path, skipped_entries)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(root_path) DO UPDATE SET
                   name = excluded.name,
                   skipped_entries = excluded.skipped_entries",
                params![
                    library_id,
                    scan.library_name,
                    root_path,
                    scan.skipped_entries
                ],
            )
            .map_err(|_| DatabaseError::Unavailable)?;
        transaction
            .execute(
                "DELETE FROM media_items WHERE library_id = ?1",
                [&library_id],
            )
            .map_err(|_| DatabaseError::Unavailable)?;

        {
            let mut statement = transaction
                .prepare(
                    "INSERT INTO media_items(id, library_id, path, title, extension, size_bytes)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )
                .map_err(|_| DatabaseError::Unavailable)?;
            for media in &scan.items {
                statement
                    .execute(params![
                        media.item.id,
                        library_id,
                        media.path.to_string_lossy(),
                        media.item.title,
                        media.item.extension,
                        media.item.size_bytes,
                    ])
                    .map_err(|_| DatabaseError::Unavailable)?;
            }
        }

        transaction
            .execute(
                "UPDATE app_state SET current_library_id = ?1 WHERE singleton = 1",
                [&library_id],
            )
            .map_err(|_| DatabaseError::Unavailable)?;
        transaction.commit().map_err(|_| DatabaseError::Unavailable)
    }

    pub(crate) fn current_library(&self) -> Result<Option<LibraryScan>, DatabaseError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?;
        let library = connection
            .query_row(
                "SELECT libraries.id, libraries.name, libraries.skipped_entries
                 FROM app_state
                 JOIN libraries ON libraries.id = app_state.current_library_id
                 WHERE app_state.singleton = 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, usize>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| DatabaseError::Unavailable)?;
        let Some((library_id, library_name, skipped_entries)) = library else {
            return Ok(None);
        };

        let mut statement = connection
            .prepare(
                "SELECT id, title, extension, size_bytes
                 FROM media_items WHERE library_id = ?1
                 ORDER BY title COLLATE NOCASE, id",
            )
            .map_err(|_| DatabaseError::Unavailable)?;
        let items = statement
            .query_map([library_id], |row| {
                Ok(crate::media::MediaItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    extension: row.get(2)?,
                    size_bytes: row.get(3)?,
                })
            })
            .map_err(|_| DatabaseError::Unavailable)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| DatabaseError::InvalidText)?;

        Ok(Some(LibraryScan {
            library_name,
            items,
            skipped_entries,
        }))
    }

    pub(crate) fn media_location(
        &self,
        media_id: &str,
    ) -> Result<Option<MediaLocation>, DatabaseError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?;
        connection
            .query_row(
                "SELECT libraries.root_path, media_items.path, media_items.extension
                 FROM app_state
                 JOIN libraries ON libraries.id = app_state.current_library_id
                 JOIN media_items ON media_items.library_id = libraries.id
                 WHERE app_state.singleton = 1 AND media_items.id = ?1",
                [media_id],
                |row| {
                    Ok(MediaLocation {
                        root_path: PathBuf::from(row.get::<_, String>(0)?),
                        media_path: PathBuf::from(row.get::<_, String>(1)?),
                        extension: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(|_| DatabaseError::Unavailable)
    }

    pub(crate) fn insert_peer(
        &self,
        id: &str,
        display_name: &str,
        token_digest: &[u8; 32],
        capability: &str,
        created_at: i64,
    ) -> Result<(), DatabaseError> {
        self.connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?
            .execute(
                "INSERT INTO trusted_peers(id, display_name, token_digest, capability, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    id,
                    display_name,
                    token_digest.as_slice(),
                    capability,
                    created_at
                ],
            )
            .map(|_| ())
            .map_err(|_| DatabaseError::Unavailable)
    }

    pub(crate) fn insert_browser_peer_and_session(
        &self,
        session: &NewBrowserSession<'_>,
    ) -> Result<(), DatabaseError> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?;
        let transaction = connection
            .transaction()
            .map_err(|_| DatabaseError::Unavailable)?;
        transaction
            .execute(
                "INSERT INTO trusted_peers(id, display_name, token_digest, capability, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    session.peer_id,
                    session.display_name,
                    session.peer_token_digest,
                    session.capability,
                    session.created_at
                ],
            )
            .map_err(|_| DatabaseError::Unavailable)?;
        transaction
            .execute(
                "INSERT INTO browser_sessions(
                   token_digest, peer_id, capability, created_at, expires_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    session.session_digest,
                    session.peer_id,
                    session.capability,
                    session.created_at,
                    session.expires_at
                ],
            )
            .map_err(|_| DatabaseError::Unavailable)?;
        transaction.commit().map_err(|_| DatabaseError::Unavailable)
    }

    pub(crate) fn browser_session_by_digest(
        &self,
        token_digest: &[u8; 32],
    ) -> Result<Option<BrowserSessionRecord>, DatabaseError> {
        self.connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?
            .query_row(
                "SELECT trusted_peers.id, trusted_peers.display_name,
                        browser_sessions.capability, browser_sessions.expires_at,
                        browser_sessions.revoked, trusted_peers.revoked
                 FROM browser_sessions
                 JOIN trusted_peers ON trusted_peers.id = browser_sessions.peer_id
                 WHERE browser_sessions.token_digest = ?1",
                [token_digest.as_slice()],
                |row| {
                    Ok(BrowserSessionRecord {
                        peer_id: row.get(0)?,
                        display_name: row.get(1)?,
                        capability: row.get(2)?,
                        expires_at: row.get(3)?,
                        session_revoked: row.get(4)?,
                        peer_revoked: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(|_| DatabaseError::Unavailable)
    }

    pub(crate) fn prune_expired_browser_sessions(&self, now: i64) -> Result<usize, DatabaseError> {
        self.connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?
            .execute("DELETE FROM browser_sessions WHERE expires_at <= ?1", [now])
            .map_err(|_| DatabaseError::Unavailable)
    }

    #[cfg(test)]
    pub(crate) fn set_browser_session_capability(
        &self,
        token_digest: &[u8; 32],
        capability: &str,
    ) -> Result<(), DatabaseError> {
        self.connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?
            .execute(
                "UPDATE browser_sessions SET capability = ?2 WHERE token_digest = ?1",
                params![token_digest, capability],
            )
            .map(|_| ())
            .map_err(|_| DatabaseError::Unavailable)
    }

    pub(crate) fn peer_by_digest(
        &self,
        token_digest: &[u8; 32],
    ) -> Result<Option<TrustedPeerRecord>, DatabaseError> {
        self.connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?
            .query_row(
                "SELECT id, display_name, capability, created_at, revoked
                 FROM trusted_peers WHERE token_digest = ?1",
                [token_digest.as_slice()],
                |row| {
                    Ok(TrustedPeerRecord {
                        id: row.get(0)?,
                        display_name: row.get(1)?,
                        capability: row.get(2)?,
                        created_at: row.get(3)?,
                        revoked: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|_| DatabaseError::Unavailable)
    }

    pub(crate) fn active_peers(&self) -> Result<Vec<TrustedPeerRecord>, DatabaseError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?;
        let mut statement = connection
            .prepare(
                "SELECT id, display_name, capability, created_at, revoked
                 FROM trusted_peers WHERE revoked = 0
                 ORDER BY created_at DESC, display_name COLLATE NOCASE, id",
            )
            .map_err(|_| DatabaseError::Unavailable)?;
        let peers = statement
            .query_map([], |row| {
                Ok(TrustedPeerRecord {
                    id: row.get(0)?,
                    display_name: row.get(1)?,
                    capability: row.get(2)?,
                    created_at: row.get(3)?,
                    revoked: row.get(4)?,
                })
            })
            .map_err(|_| DatabaseError::Unavailable)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| DatabaseError::InvalidText)?;
        Ok(peers)
    }

    pub(crate) fn revoke_peer(&self, peer_id: &str) -> Result<bool, DatabaseError> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?;
        let transaction = connection
            .transaction()
            .map_err(|_| DatabaseError::Unavailable)?;
        let changed = transaction
            .execute(
                "UPDATE trusted_peers SET revoked = 1 WHERE id = ?1 AND revoked = 0",
                [peer_id],
            )
            .map_err(|_| DatabaseError::Unavailable)?;
        transaction
            .execute(
                "UPDATE browser_sessions SET revoked = 1 WHERE peer_id = ?1 AND revoked = 0",
                [peer_id],
            )
            .map_err(|_| DatabaseError::Unavailable)?;
        transaction
            .commit()
            .map_err(|_| DatabaseError::Unavailable)?;
        Ok(changed == 1)
    }

    pub(crate) fn revoke_all_peers(&self) -> Result<usize, DatabaseError> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| DatabaseError::Unavailable)?;
        let transaction = connection
            .transaction()
            .map_err(|_| DatabaseError::Unavailable)?;
        let changed = transaction
            .execute("UPDATE trusted_peers SET revoked = 1 WHERE revoked = 0", [])
            .map_err(|_| DatabaseError::Unavailable)?;
        transaction
            .execute(
                "UPDATE browser_sessions SET revoked = 1 WHERE revoked = 0",
                [],
            )
            .map_err(|_| DatabaseError::Unavailable)?;
        transaction
            .commit()
            .map_err(|_| DatabaseError::Unavailable)?;
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tempfile::tempdir;

    use super::{DatabaseError, LibraryDatabase, SCHEMA_VERSION};

    #[test]
    fn migrates_a_new_database_to_the_current_version() {
        let database = LibraryDatabase::in_memory().expect("database should migrate");
        let connection = database
            .connection
            .lock()
            .expect("database lock should open");
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("schema version should load");

        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn rejects_a_database_from_a_newer_schema() {
        let connection = Connection::open_in_memory().expect("database should open");
        connection
            .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
            .expect("schema version should update");

        let error = LibraryDatabase::initialize(connection)
            .expect_err("newer schema must not be downgraded");

        assert!(matches!(error, DatabaseError::Unavailable));
    }

    #[test]
    fn migrates_version_one_without_losing_the_current_library() {
        let workspace = tempdir().expect("temporary workspace should exist");
        let path = workspace.path().join("version-one.sqlite3");
        let connection = Connection::open(&path).expect("database should open");
        connection
            .execute_batch(
                "CREATE TABLE libraries (
                   id TEXT PRIMARY KEY,
                   name TEXT NOT NULL,
                   root_path TEXT NOT NULL UNIQUE,
                   skipped_entries INTEGER NOT NULL DEFAULT 0
                 );
                 CREATE TABLE media_items (
                   id TEXT PRIMARY KEY,
                   library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
                   path TEXT NOT NULL UNIQUE,
                   title TEXT NOT NULL,
                   extension TEXT NOT NULL,
                   size_bytes INTEGER NOT NULL
                 );
                 CREATE TABLE app_state (
                   singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                   current_library_id TEXT REFERENCES libraries(id) ON DELETE SET NULL
                 );
                 INSERT INTO libraries VALUES ('library-1', 'Videos', 'C:/Videos', 0);
                 INSERT INTO media_items VALUES (
                   'media-1', 'library-1', 'C:/Videos/Movie.mp4', 'Movie', 'mp4', 5
                 );
                 INSERT INTO app_state VALUES (1, 'library-1');
                 PRAGMA user_version = 1;",
            )
            .expect("version one schema should be created");
        drop(connection);

        let database = LibraryDatabase::open(&path).expect("database should migrate");
        let library = database
            .current_library()
            .expect("library should load")
            .expect("current library should remain");

        assert_eq!(library.library_name, "Videos");
        assert_eq!(library.items[0].title, "Movie");
        let connection = database.connection.lock().expect("database should lock");
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("schema version should load");
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn migrates_version_two_with_peers_and_adds_browser_sessions() {
        let workspace = tempdir().expect("temporary workspace should exist");
        let path = workspace.path().join("version-two.sqlite3");
        let connection = Connection::open(&path).expect("database should open");
        connection
            .execute_batch(
                "CREATE TABLE libraries (
                   id TEXT PRIMARY KEY, name TEXT NOT NULL,
                   root_path TEXT NOT NULL UNIQUE, skipped_entries INTEGER NOT NULL DEFAULT 0
                 );
                 CREATE TABLE media_items (
                   id TEXT PRIMARY KEY, library_id TEXT NOT NULL REFERENCES libraries(id),
                   path TEXT NOT NULL UNIQUE, title TEXT NOT NULL,
                   extension TEXT NOT NULL, size_bytes INTEGER NOT NULL
                 );
                 CREATE TABLE app_state (
                   singleton INTEGER PRIMARY KEY, current_library_id TEXT
                 );
                 CREATE TABLE trusted_peers (
                   id TEXT PRIMARY KEY, display_name TEXT NOT NULL,
                   token_digest BLOB NOT NULL UNIQUE CHECK (length(token_digest) = 32),
                   capability TEXT NOT NULL, created_at INTEGER NOT NULL,
                   revoked INTEGER NOT NULL DEFAULT 0 CHECK (revoked IN (0, 1))
                 );
                 INSERT INTO app_state VALUES (1, NULL);
                 INSERT INTO trusted_peers VALUES (
                   'peer-1', 'Existing Peer', zeroblob(32), 'library.read', 1000, 0
                 );
                 PRAGMA user_version = 2;",
            )
            .expect("version two schema should be created");
        drop(connection);

        let database = LibraryDatabase::open(&path).expect("database should migrate");
        assert_eq!(database.active_peers().expect("peers should load").len(), 1);
        let connection = database.connection.lock().expect("database should lock");
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("schema version should load");
        let session_table: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'browser_sessions'",
                [],
                |row| row.get(0),
            )
            .expect("session table should query");
        assert_eq!(version, SCHEMA_VERSION);
        assert_eq!(session_table, 1);
    }
}
