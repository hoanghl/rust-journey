use std::convert::From;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use std::{net::Ipv4Addr, str::FromStr};
// ================================================
// Definitions for DB entry
// ================================================
#[rustfmt::skip]
#[repr(u8)]
pub enum Role {
    Default = 0,
    Master  = 1,
    Data    = 2,
    DNS     = 3,
}

pub struct FileInfoEntry {
    pub filename: String,
    pub is_local: bool,
    pub path: Option<String>,
    pub node_id: String,
    pub last_updated: Option<DateTime<Utc>>,
}

struct NodeInfoEntry {
    pub node_id: String,
    pub ip: Option<Ipv4Addr>,
    pub role: Role,
    pub last_updated: Option<DateTime<Utc>>,
}

// ================================================
// Implementations
// ================================================
impl From<u8> for Role {
    fn from(value: u8) -> Self {
        match value {
            1 => Role::Master,
            2 => Role::Data,
            3 => Role::DNS,
            _ => panic!("Error as parsing to enum Role: value = {}", value),
        }
    }
}

impl From<&Role> for u8 {
    fn from(value: &Role) -> Self {
        match value {
            Role::Master => 1,
            Role::Data => 2,
            Role::DNS => 3,
            _ => panic!("Error as parsing from enum Role"),
        }
    }
}

impl FileInfoEntry {
    pub fn initialize(filename: String, is_local: bool, path: Option<String>, node_id: String) -> FileInfoEntry {
        FileInfoEntry {
            filename,
            is_local,
            path,
            node_id,
            last_updated: None,
        }
    }
}

impl NodeInfoEntry {
    pub fn initialize(node_id: String, ip: Ipv4Addr, role: Role) -> NodeInfoEntry {
        NodeInfoEntry {
            node_id,
            ip: Some(ip),
            role,
            last_updated: None,
        }
    }
}

// ================================================
// Definitions for DB instance
// ================================================

pub trait InMemDB<T> {
    fn create_db(&mut self) -> Result<()>;
    fn upsert(&self, info: &T) -> Result<()>;
}

pub struct FileInfoDB {
    db_name: &'static str,
    db_conn: Option<Connection>,
}

pub struct NodeInfoDB {
    db_name: &'static str,
    db_conn: Option<Connection>,
}

// ================================================
// Implementations
// ================================================

impl InMemDB<FileInfoEntry> for FileInfoDB {
    fn create_db(&mut self) -> Result<()> {
        log::debug!("Creating db {}", self.db_name);

        let conn = Connection::open_in_memory()?;

        conn.execute(
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                filename        TEXT    UNIQUE
                ,is_local       BOOLEAN NOT NULL
                ,path           TEXT
                ,node           TEXT    NOT NULL
                ,last_updated   TEXT    NOT NULL
            );",
                &self.db_name
            )
            .as_str(),
            // [&self.db_name],
            [],
        )?;

        self.db_conn = Some(conn);

        Ok(())
    }

    fn upsert(&self, info: &FileInfoEntry) -> Result<()> {
        log::debug!("Upsert..");

        let current = Utc::now();

        match self.db_conn.as_ref().unwrap().execute(
            format!(
                "INSERT INTO {}
                (filename, is_local, path, node, last_updated)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(filename) DO UPDATE SET
                    is_local = ?2,
                    path = ?3,
                    node = ?4,
                    last_updated = ?5
                ;",
                self.db_name
            )
            .as_str(),
            params![
                info.filename,
                info.is_local,
                info.path,
                info.node_id,
                current.to_rfc3339(),
            ],
        ) {
            Ok(size) => log::info!("Upserted: {}", size),
            Err(err) => log::error!("{}", err),
        };

        Ok(())
    }
}

impl FileInfoDB {
    pub fn intialize(db_name: &'static str) -> FileInfoDB {
        FileInfoDB { db_name, db_conn: None }
    }

    pub fn get_file_info(&self, filename: &String) -> Result<Vec<FileInfoEntry>> {
        let mut stmt = self
            .db_conn
            .as_ref()
            .unwrap()
            .prepare(format!("SELECT * FROM {} WHERE filename = ?1;", self.db_name).as_str())?;
        let rows = stmt.query_map([&filename], |row| {
            log::debug!("inside: {:?}", row.get::<usize, String>(3));

            Ok(FileInfoEntry {
                filename: row.get(0)?,
                is_local: row.get::<usize, i32>(1)? == 1,
                path: row.get(2)?,
                node_id: row.get(3)?,
                last_updated: Some(row.get::<usize, String>(4)?.parse().unwrap()),
            })
        })?;

        rows.collect()
    }
}

impl InMemDB<NodeInfoEntry> for NodeInfoDB {
    fn create_db(&mut self) -> Result<()> {
        log::debug!("Creating db {}", self.db_name);

        let conn = Connection::open_in_memory()?;

        conn.execute(
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                node_id         TEXT    NOT NULL
                ,ip             TEXT    NOT NULL
                ,role           INTEGER NOT NULL
                ,last_updated   TEXT    NOT NULL
            );",
                &self.db_name
            )
            .as_str(),
            [],
        )?;

        self.db_conn = Some(conn);

        Ok(())
    }

    fn upsert(&self, info: &NodeInfoEntry) -> Result<()> {
        log::debug!("Upsert..");

        let current = Utc::now();

        match self.db_conn.as_ref().unwrap().execute(
            format!(
                "INSERT INTO {}
                (node_id, ip, role, last_updated)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(filename) DO UPDATE SET
                    ip = ?2,
                    role = ?3,
                    last_updated = ?4
                ;",
                self.db_name
            )
            .as_str(),
            params![
                info.node_id,
                info.ip.unwrap().to_string(),
                u8::from(&info.role),
                current.to_rfc3339(),
            ],
        ) {
            Ok(size) => log::info!("Upserted: {}", size),
            Err(err) => log::error!("{}", err),
        };

        Ok(())
    }
}

impl NodeInfoDB {
    pub fn intialize(db_name: &'static str) -> NodeInfoDB {
        NodeInfoDB { db_name, db_conn: None }
    }

    pub fn get_node_info(&self, filename: &String) -> Result<Vec<NodeInfoEntry>> {
        let mut stmt = self
            .db_conn
            .as_ref()
            .unwrap()
            .prepare(format!("SELECT * FROM {} WHERE node_id = ?1;", self.db_name).as_str())?;
        let rows = stmt.query_map([&filename], |row| {
            log::debug!("inside: {:?}", row.get::<usize, String>(3));

            let ip_str: String = row.get(1)?;
            let ip = match Ipv4Addr::from_str(ip_str.as_str()) {
                Ok(ip) => Some(ip),
                Err(e) => {
                    log::error!("Cannot parse following to Ipv4: {}: {}", ip_str, e);
                    None
                }
            };

            Ok(NodeInfoEntry {
                node_id: row.get(0)?,
                ip: ip,
                role: Role::from(row.get::<usize, u8>(2)?),
                last_updated: Some(row.get::<usize, String>(3)?.parse().unwrap()),
            })
        })?;

        rows.collect()
    }

    pub fn get_master_info(&self, filename: &String) -> Result<Vec<NodeInfoEntry>> {
        let mut stmt = self
            .db_conn
            .as_ref()
            .unwrap()
            .prepare(format!("SELECT * FROM {} WHERE node_id = ?1;", self.db_name).as_str())?;
        let rows = stmt.query_map([&filename], |row| {
            log::debug!("inside: {:?}", row.get::<usize, String>(3));

            let ip_str: String = row.get(1)?;
            let ip = match Ipv4Addr::from_str(ip_str.as_str()) {
                Ok(ip) => Some(ip),
                Err(e) => {
                    log::error!("Cannot parse following to Ipv4: {}: {}", ip_str, e);
                    None
                }
            };

            Ok(NodeInfoEntry {
                node_id: row.get(0)?,
                ip: ip,
                role: Role::from(row.get::<usize, u8>(2)?),
                last_updated: Some(row.get::<usize, String>(3)?.parse().unwrap()),
            })
        })?;

        rows.collect()
    }
}
