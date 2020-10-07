use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{
    fmt,
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

mod default_computer {
    pub fn verbose() -> i32 {
        0
    }
    pub fn no_confirm() -> bool {
        false
    }
    pub fn syslog() -> bool {
        false
    }
    pub fn color() -> bool {
        true
    }
    pub fn download_timeout() -> bool {
        true
    }
    pub fn arch() -> String {
        env!("ARCH").to_string()
    }
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_computer::verbose")]
    pub verbose: i32,
    #[serde(default = "default_computer::no_confirm")]
    pub no_confirm: bool,
    #[serde(default = "default_computer::syslog")]
    pub syslog: bool,
    #[serde(default = "default_computer::color")]
    pub color: bool,
    #[serde(default = "default_computer::download_timeout")]
    pub download_timeout: bool,
    #[serde(default = "default_computer::arch")]
    pub arch: String,
    #[serde(default)]
    pub paths: PathConfig,
    #[serde(default)]
    pub databases: Vec<Database>,
}

#[derive(Deserialize)]
pub struct PathConfig {
    pub root: PathBuf,
    pub database: PathBuf,
    pub gpg: PathBuf,
    pub logfile: PathBuf,
    pub hook_dirs: Vec<PathBuf>,
    pub cache_dirs: Vec<PathBuf>,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("/"),
            database: PathBuf::from("/var/lib/pacman"),
            hook_dirs: vec![PathBuf::from("/etc/pacman.d/hooks")],
            gpg: PathBuf::from("/etc/pacman.d/gnupg"),
            logfile: PathBuf::from("/var/log/rpac.log"),
            cache_dirs: vec![PathBuf::from("/var/cache/pacman/pkg")],
        }
    }
}

pub struct Database {
    pub name: String,
    pub servers: Vec<String>,
    pub usage: u32,
    pub siglevel: u32,
    pub siglevel_mask: u32,
}

impl Database {
    pub fn new(
        name: String,
        servers: Vec<String>,
        usage: u32,
        siglevel: u32,
        siglevel_mask: u32,
    ) -> Database {
        Self {
            name,
            servers: {
                let mut ret = Vec::new();
                for server in servers {
                    match File::open(PathBuf::from(server.to_owned())) {
                        // Parse Mirrorlist
                        Ok(file) => {
                            let bufread = BufReader::new(file);
                            for line in bufread.lines() {
                                let line = line.unwrap();
                                let server = line.trim_start_matches("Server = ");
                                if server.is_empty() || server.starts_with('#') {
                                    continue;
                                }
                                ret.push(server.to_string());
                            }
                        }
                        // File was not found. Probably isn't a mirrorlist, adding it directly
                        Err(ref e) if e.kind() == io::ErrorKind::NotFound => ret.push(server),
                        // A error that wasn't expected
                        Err(e) => panic!("Could not open file: {}", e),
                    }
                }
                ret
            },
            usage,
            siglevel,
            siglevel_mask,
        }
    }
}

impl<'de> Deserialize<'de> for Database {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Name,
            Servers,
            Usage,
            SigLevel,
            SigLevelMask,
        };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter
                            .write_str("`name`, `servers`, `usage`, `siglevel`, `siglevel_mask`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "name" => Ok(Field::Name),
                            "servers" => Ok(Field::Servers),
                            "usage" => Ok(Field::Usage),
                            "siglevel" => Ok(Field::SigLevel),
                            "siglevel_mask" => Ok(Field::SigLevelMask),

                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct DatabaseVisitor;

        impl<'de> Visitor<'de> for DatabaseVisitor {
            type Value = Database;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Database")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Database, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut servers = None;
                let mut usage = None;
                let mut siglevel = None;
                let mut siglevel_mask = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Servers => {
                            if servers.is_some() {
                                return Err(de::Error::duplicate_field("servers"));
                            }
                            servers = Some(map.next_value()?);
                        }
                        Field::Usage => {
                            if usage.is_some() {
                                return Err(de::Error::duplicate_field("usage"));
                            }
                            usage = Some(map.next_value()?);
                        }
                        Field::SigLevel => {
                            if siglevel.is_some() {
                                return Err(de::Error::duplicate_field("siglevel"));
                            }
                            siglevel = Some(map.next_value()?);
                        }
                        Field::SigLevelMask => {
                            if siglevel_mask.is_some() {
                                return Err(de::Error::duplicate_field("siglevel_mask"));
                            }
                            siglevel_mask = Some(map.next_value()?);
                        }
                    }
                }
                let name = name.ok_or_else(|| de::Error::missing_field("secs"))?;
                let servers = servers.ok_or_else(|| de::Error::missing_field("servers"))?;
                let usage = usage.ok_or_else(|| de::Error::missing_field("usage"))?;
                let siglevel = siglevel.ok_or_else(|| de::Error::missing_field("siglevel"))?;
                let siglevel_mask =
                    siglevel_mask.ok_or_else(|| de::Error::missing_field("siglevel_mask"))?;

                Ok(Database::new(name, servers, usage, siglevel, siglevel_mask))
            }
        }

        const FIELDS: &[&str] = &["name", "servers", "usage", "siglevel", "siglevel_mask"];
        deserializer.deserialize_struct("Duration", FIELDS, DatabaseVisitor)
    }
}
