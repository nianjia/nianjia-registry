#![allow(dead_code)]
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::str::FromStr;

use serde::{de, ser, Deserialize, Serialize};

use nianjia::util::errors::NianjiaResult;

#[derive(PartialEq)]
struct Duration(humantime::Duration);
struct DurationVisitor;

impl fmt::Debug for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Duration {:?}", self.0)
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self(humantime::Duration::from_str("1s").unwrap())
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let string = format!("{}s {}ns", self.0.as_secs(), self.0.subsec_nanos());
        serializer.serialize_str(&string)
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(DurationVisitor)
    }
}

impl<'de> de::Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string to represent the time duration.")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match humantime::Duration::from_str(v) {
            Err(_) => Err(E::custom(format!("can't parse the duration"))),
            Ok(d) => Ok(Duration(d)),
        }
    }
}

type LogLevel = String;

// Configuration is a versioned registry configuration, intended to be provided by a yaml file, and
// optionally modified by environment variables.
//
// Note that yaml field names should never include _ characters, since this is the separator used
// in environment variable names.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Configuration {
    // `version` is the version which defines the format of the rest of the configuration
    version: String,

    // `log` supports setting various parameters related to the logging
    // subsystem.
    log: Log,

    // `storage` is the configuration for the registry's storage driver
    #[serde(default)]
    storage: Storage,
    // `auth` allows configuration of various authorization methods that may be
    // used to gate requests.
    #[serde(default)]
    auth: Auth,

    // `middleware` lists all middlewares to be used by the registry.
    #[serde(default)]
    middleware: BTreeMap<String, Vec<Middleware>>,

    #[serde(default)]
    reporting: Reporting,
    #[serde(default)]
    http: Http,

    #[serde(default)]
    notifications: Notifications,

    #[serde(default)]
    redis: Redis,

    #[serde(default)]
    health: Health,
    #[serde(default)]
    proxy: Proxy,

    // `compatibility` is used for configurations of working with older or deprecated features.
    #[serde(default)]
    compatibility: Compatibility,

    // `validation` configures validation options for the registry.
    #[serde(default)]
    validation: Validation,

    // `policy` configures registry policy options.
    #[serde(default)]
    policy: Policy,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Log {
    #[serde(default)]
    access_log: AccessLog,
    #[serde(default)]
    level: LogLevel,
    #[serde(default)]
    formatter: String,
    #[serde(default)]
    fields: BTreeMap<String, String>,
    #[serde(default)]
    hooks: Vec<LogHook>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct AccessLog {
    #[serde(default)]
    disabled: bool,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct LogHook {
    #[serde(default)]
    disabled: bool,
    #[serde(default)]
    #[serde(rename = "type")]
    _type: String,
    #[serde(default)]
    levels: Vec<String>,
    #[serde(default)]
    mail_options: MailOptions,
}


#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
#[serde(default)]
struct Parameters {
    #[serde(flatten)]
    parameters: BTreeMap<String, Parameter>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum Parameter {
    Integer(i64),
    Double(f64),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Storage {
    #[serde(flatten, default)]
    media: StorageMedia,
    maintenance: Option<Maintenance>,
    cache: Option<Cache>,
    delete: Option<Delete>,
    redirect: Option<Redirect>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum StorageMedia {
    #[serde(rename = "filesystem")]
    Filesystem(BTreeMap<String, Parameter>),
    #[serde(rename = "s3")]
    S3(BTreeMap<String, Parameter>),
    #[serde(rename = "inmemory")]
    InMemory,
}

impl Default for StorageMedia {
    fn default() -> Self {
        StorageMedia::Filesystem(BTreeMap::new())
    }
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Maintenance {
    uploadpurging: Parameters,
    readonly: Parameters,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Cache(Parameters);

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Delete(Parameters);

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Redirect(Parameters);

type Auth = BTreeMap<String, Parameters>;

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Middleware {
    name: String,
    #[serde(default)]
    disable: bool,
    options: Parameters,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Reporting {
    bugsnag: BugsnagReporting,
    #[serde(rename = "newrelic", default)]
    new_relic: NewRelicReporting,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Http {
    #[serde(default)]
    addr: String,
    #[serde(default)]
    net: String,
    #[serde(default)]
    host: String,
    #[serde(default)]
    prefix: String,
    #[serde(default)]
    secret: String,
    #[serde(rename = "relativeurls", default)]
    relative_urls: bool,
    #[serde(rename = "draintimeout", default)]
    drain_timeout: Duration,
    #[serde(default)]
    tls: Tls,
    headers: Header,
    #[serde(default)]
    debug: Debug,
    #[serde(default)]
    http2: Http2,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Tls {
    certificate: String,
    key: String,
    #[serde(rename = "clientcas")]
    client_CAs: Vec<String>,
    #[serde(rename = "minimumtls", default)]
    minimum_tls: String,
    #[serde(rename = "letsencrypt", default)]
    lets_encrypt: LetsEncrypt,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct LetsEncrypt {
    #[serde(rename = "cachefile")]
    cache_file: String,
    email: String,
    #[serde(default)]
    hosts: Vec<String>,
}

type Header = BTreeMap<String, Vec<String>>;

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Debug {
    #[serde(default)]
    addr: String,
    #[serde(default)]
    prometheus: Prometheus,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Prometheus {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    path: String,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Http2 {
    disabled: bool,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Notifications {
    #[serde(rename = "events", default)]
    event_config: Events,
    endpoints: Vec<EndPoint>,
}


#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Redis {
    // `addr` specifies the the redis instance available to the application.
    addr: String,
    // `password` string to use when making a connection.
    password: String,
    // `db` specifies the database to connect to on the redis instance.
    db: u32,

    #[serde(rename = "dialtimeout", default)]
    dial_timeout: Duration, // timeout for connect
    #[serde(rename = "readtimeout", default)]
    read_timeout: Duration, // timeout for reads of data
    #[serde(rename = "writetimeout", default)]
    write_timeout: Duration, // timeout for writes of data

    // `pool` configures the behavior of the redis connection pool.
    pool: Pool,

}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Pool {
    // `max_idle` sets the maximum number of idle connections.
    #[serde(rename = "maxidle")]
    max_idle: u32,
    // `max_active` sets the maximum number of connections that should be
    // opened before blocking a connection request.
    #[serde(rename = "maxactive")]
    max_active: u32,
    // `idle_timeout` sets the amount time to wait before closing
    // inactive connections.
    #[serde(rename = "idletimeout")]
    idle_timeout: Duration,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Health {
    #[serde(rename = "file", default)]
    file_checkers: Vec<FileChecker>,
    #[serde(rename = "http", default)]
    http_checkers: Vec<HttpChecker>,
    #[serde(rename = "tcp", default)]
    tcp_checkers: Vec<TcpChecker>,
    #[serde(rename = "storagedriver")]
    stroage_driver: StorageDriver,
}

// Proxy configures the registry as a pull through cache
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Proxy {
    #[serde(rename = "remoteurl")]
    remote_url: String,
    username: String,
    // Password of the hub user
    password: String,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Compatibility {
    schema1: Schema1, // `schema1` configures how schema1 manifests will be handled
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Schema1 {
    // `trust_key` is the signing key to use for adding the signature to
    // schema1 manifests.
    #[serde(rename = "signingkeyfile", default)]
    trust_key: String,
    #[serde(default)]
    // `enabled` determines if schema1 manifests should be pullable
    enabled: bool,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Validation {
    // Enabled enables the other options in this section. This field is
    // deprecated in favor of Disabled.
    enabled: bool,
    #[serde(rename = "signingkeyfile", default)]
    // Disabled disables the other options in this section.
    disabled: bool,
    // Manifests configures manifest validation.
    manifests: Manifest,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Manifest {
    // `urls` configures validation for URLs in pushed manifests.
    urls: Urls,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Urls {
    // allow` specifies regular expressions (https://godoc.org/regexp/syntax)
    // that URLs in pushed manifests must match.
    allow: Vec<String>,
    // `deny` specifies regular expressions (https://godoc.org/regexp/syntax)
    // that URLs in pushed manifests must not match.
    deny: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Policy {
    repository: Repository,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Repository {
    classes: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct MailOptions {
    #[serde(default)]
    smtp: Smtp,
    #[serde(default)]
    from: String,
    #[serde(default)]
    to: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Smtp {
    #[serde(default)]
    addr: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    insecure: bool,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct FileChecker {
    #[serde(default)]
    interval: Duration,
    #[serde(default)]
    file: String,
    #[serde(default)]
    threshold: u32,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct HttpChecker {
    #[serde(default)]
    timeout: Duration,
    #[serde(rename = "statuscode")]
    status_code: i32,
    #[serde(default)]
    interval: Duration,
    #[serde(default)]
    url: String,
    headers: Header,
    #[serde(default)]
    threshold: u32,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct TcpChecker {
    #[serde(default)]
    timeout: Duration,
    #[serde(default)]
    interval: Duration,
    #[serde(default)]
    add: String,
    #[serde(default)]
    threshold: u32,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct StorageDriver {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    interval: Duration,
    #[serde(default)]

    threshold: u32,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Events {
    include_references: bool,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct EndPoint {
    name: String,
    #[serde(default)]
    disabled: bool,
    url: String,
    headers: Header,
    #[serde(default)]
    timeout: Duration,
    #[serde(default)]
    threshold: u32,
    #[serde(default)]
    backoff: Duration,
    #[serde(rename = "ignoredmediatypes")]
    ignore_media_type: Vec<String>,
    #[serde(default)]
    ignore: Ignore,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Ignore {
    #[serde(default, rename = "mediatypes")]
    media_types: Vec<String>,
    actions: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct BugsnagReporting {
    #[serde(rename = "apikey")]
    API_key: String,
    #[serde(rename = "releasestage", default)]
    release_stage: String,
    #[serde(default)]
    endpoint: String,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct NewRelicReporting {
    #[serde(rename = "licensekey")]
    license_key: String,
    name: String,
    verbose: bool,
}

pub fn parse_str<T: AsRef<str>>(content: &T) -> NianjiaResult<Configuration> {
    let config = serde_yaml::from_str(&content.as_ref())?;
    Ok(config)
}

pub fn parse_file(file: &str) -> NianjiaResult<Configuration> {
    parse_str(&fs::read_to_string(file)?)
}


#[cfg(test)]
mod tests {
    use crate::configuration::*;
    // CONFIG_YAML_V0_1 is a Version 0.1 yaml document representing configStruct
    const CONFIG_YAML_V0_1: &'static str = "
version: 0.1
log:
  level: info
  fields:
    environment: test
storage:
  s3:
    region: us-east-1
    bucket: my-bucket
    rootdirectory: /registry
    encrypt: true
    secure: false
    accesskey: SAMPLEACCESSKEY
    secretkey: SUPERSECRET
    host: ~
    port: 42
auth:
  silly:
    realm: silly
    service: silly
notifications:
  endpoints:
    - name: endpoint-1
      url:  http://example.com
      headers:
        Authorization: [Bearer <example>]
      ignoredmediatypes:
        - application/octet-stream
      ignore:
        mediatypes:
           - application/octet-streamsto
        actions:
           - pull
reporting:
  bugsnag:
    apikey: BugsnagApiKey
http:
  clientcas:
    - /path/to/ca.pem
  headers:
    X-Content-Type-Options: [nosniff]
";

    #[test]
    fn test_parse_roundtrip() {
        let config = parse_str(&CONFIG_YAML_V0_1).unwrap();
        let content = serde_yaml::to_string(&config).unwrap();
        let config_repeat = parse_str(&content).unwrap();
        assert_eq!(config, config_repeat);
        assert_eq!(content, serde_yaml::to_string(&config_repeat).unwrap());
    }
}