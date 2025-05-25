use std::{convert::From, net::SocketAddrV4};

use crate::components::entity::node_roles::Role;
use chrono::{DateTime, Local};
use rusqlite::{params, Connection, Result};
use std::{net::Ipv4Addr, str::FromStr};

// ================================================
// Definitions for DB entry
// ================================================

pub struct FileInfoEntry {
    pub filename: String,
    pub is_local: bool,
    pub path: Option<String>,
    pub node_id: String,
    pub last_updated: Option<DateTime<Local>>,
}

pub struct NodeInfoEntry {
    pub node_id: String,
    pub ip: Option<Ipv4Addr>,
    pub port: u16,
    pub role: Role,
    pub last_updated: Option<DateTime<Local>>,
}

// ================================================
// Implementations
// ================================================

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
    pub fn initialize(ip: Ipv4Addr, port: u16, role: Role) -> NodeInfoEntry {
        NodeInfoEntry {
            node_id: _get_node_id(&ip, port),
            ip: Some(ip),
            role,
            port,
            last_updated: None,
        }
    }
}

// ================================================
// Definitions for DB instance
// ================================================

pub trait InMemDB<T> {
    fn create_db(&mut self) -> Result<()>;
    // fn upsert(&self, ip: Ipv4Addr, port: u16, role: Role) -> Result<()>;
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
        log::info!("Creating db {}", self.db_name);

        let conn = Connection::open_in_memory()?;

        conn.execute(
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                filename        TEXT    PRIMARY KEY
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
}

impl FileInfoDB {
    pub fn intialize(db_name: &'static str) -> FileInfoDB {
        let mut db = FileInfoDB { db_name, db_conn: None };

        let _ = db.create_db();

        db
    }

    pub fn upsert(&self, info: &FileInfoEntry) -> Result<()> {
        log::debug!("Upsert..");

        let current = Local::now();

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
            Ok(size) => log::debug!("Upserted: {}", size),
            Err(err) => log::error!("{}", err),
        };

        Ok(())
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
        log::info!("Creating db {}", self.db_name);

        let conn = Connection::open_in_memory()?;

        conn.execute(
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                node_id         TEXT    PRIMARY KEY
                ,ip             TEXT    NOT NULL
                ,port           INTEGER NOT NULL
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
}

impl NodeInfoDB {
    pub fn intialize(db_name: &'static str) -> NodeInfoDB {
        let mut db = NodeInfoDB { db_name, db_conn: None };

        let _ = db.create_db();

        db
    }

    pub fn upsert(&self, ip: Ipv4Addr, port: u16, role: Role) -> Result<()> {
        log::debug!("Upsert..");

        let current = Local::now();

        match self.db_conn.as_ref().unwrap().execute(
            format!(
                "INSERT INTO {}
                (node_id, ip, port, role, last_updated)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(node_id) DO UPDATE SET
                    ip = ?2,
                    port = ?3,
                    role = ?4,
                    last_updated = ?5
                ;",
                self.db_name
            )
            .as_str(),
            params![
                SocketAddrV4::new(ip, port).to_string(),
                ip.to_string(),
                port,
                u8::from(&role),
                current.to_rfc3339(),
            ],
        ) {
            Ok(size) => log::info!("Upserted: {}", size),
            Err(err) => log::error!("{}", err),
        };

        Ok(())
    }

    pub fn get_node_info(&self, ip: Ipv4Addr, port: u16) -> Result<Vec<NodeInfoEntry>> {
        let node_id = _get_node_id(&ip, port);

        let mut stmt = self
            .db_conn
            .as_ref()
            .unwrap()
            .prepare(format!("SELECT * FROM {} WHERE node_id = ?1;", self.db_name).as_str())?;
        let rows = stmt.query_map([&node_id], |row| {
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
                port: row.get::<usize, u16>(2)?,
                role: Role::from(row.get::<usize, u8>(3)?),
                last_updated: Some(row.get::<usize, String>(4)?.parse().unwrap()),
            })
        })?;

        rows.collect()
    }

    pub fn get_data_nodes(&self) -> Result<Vec<NodeInfoEntry>> {
        let mut stmt = self
            .db_conn
            .as_ref()
            .unwrap()
            .prepare(format!("SELECT * FROM {} WHERE role = ?1;", self.db_name).as_str())?;
        let rows = stmt.query_map([&u8::from(&Role::Data)], |row| {
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
                port: row.get::<usize, u16>(2)?,
                role: Role::from(row.get::<usize, u8>(3)?),
                last_updated: Some(row.get::<usize, String>(4)?.parse().unwrap()),
            })
        })?;

        rows.collect()
    }
}

pub fn _get_node_id(ip: &Ipv4Addr, port: u16) -> String {
    SocketAddrV4::new(ip.clone(), port).to_string()
}
