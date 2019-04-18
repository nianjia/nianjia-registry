#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use serde::{de, Deserialize};

struct Duration(humantime::Duration);
struct DurationVisitor;

impl Default for Duration {
    fn default() -> Self {
        Self(humantime::Duration::from_str("1s").unwrap())
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

#[derive(Deserialize)]
pub struct Configuration {
    version: String,
    log: Log,
    #[serde(rename = "loglevel")]
    log_level: LogLevel,
    storage: Storage,
    auth: Auth,
    middleware: HashMap<String, Vec<Middleware>>,
    reporting: Reporting,
    http: Http,
    notifications: Notifications,
    redis: Redis,
    health: Health,
    proxy: Proxy,

    // `compatibility` is used for configurations of working with older or deprecated features.
    compatibility: Compatibility,

    // `validation` configures validation options for the registry.
    validation: Validation,

    // `policy` configures registry policy options.
    #[serde(default)]
    policy: Policy,
}

#[derive(Deserialize)]
struct Log {
    #[serde(default)]
    access_log: AccessLog,
    #[serde(default)]
    level: LogLevel,
    #[serde(default)]
    formatter: String,
    #[serde(default)]
    fields: HashMap<String, String>,
    hooks: Vec<LogHook>,
}

#[derive(Deserialize, Default)]
struct AccessLog {
    #[serde(default)]
    disabled: bool,
}

#[derive(Deserialize)]
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


#[derive(Deserialize, Default)]
#[serde(default)]
struct Parameters {
    #[serde(flatten)]
    parameters: HashMap<String, Parameter>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Parameter {
    Integer(i64),
    Double(f64),
    String(String),
    Boolean(bool),
}

#[derive(Deserialize)]
struct Storage {
    #[serde(flatten)]
    media: StorageMedia,
    maintenance: Option<Maintenance>,
    cache: Option<Cache>,
    delete: Option<Delete>,
    redirect: Option<Redirect>,
}

#[derive(Deserialize)]
enum StorageMedia {
    #[serde(rename = "filesystem")]
    Filesystem(Parameters),
    InMemory,
}

#[derive(Deserialize)]
struct Maintenance {
    uploadpurging: Parameters,
    readonly: Parameters,
}

#[derive(Deserialize)]
struct Cache(Parameters);

#[derive(Deserialize)]
struct Delete(Parameters);

#[derive(Deserialize)]
struct Redirect(Parameters);

type Auth = HashMap<String, Parameters>;

#[derive(Deserialize)]
struct Middleware {
    name: String,
    #[serde(default)]
    disable: bool,
    options: Parameters,
}

#[derive(Deserialize)]
struct Reporting {
    bugsnag: BugsnagReporting,
    #[serde(rename = "newrelic")]
    new_relic: NewRelicReporting,
}

#[derive(Deserialize, Default)]
struct Http {
    addr: String,
    #[serde(default)]
    net: String,
    host: String,
    prefix: String,
    secret: String,
    #[serde(rename = "relativeurls")]
    relative_urls: bool,
    #[serde(rename = "draintimeout", default)]
    drain_timeout: Duration,
    tls: Tls,
    headers: Header,
    debug: Debug,
    http2: Http2,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Default)]
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

#[derive(Deserialize, Default)]
struct LetsEncrypt {
    #[serde(rename = "cachefile")]
    cache_file: String,
    email: String,
    #[serde(default)]
    hosts: Vec<String>,
}

type Header = HashMap<String, Vec<String>>;

#[derive(Deserialize, Default)]
struct Debug {
    #[serde(default)]
    addr: String,
    #[serde(default)]
    prometheus: Prometheus,
}

#[derive(Deserialize, Default)]
struct Prometheus {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    path: String,
}

#[derive(Deserialize, Default)]
struct Http2 {
    disabled: bool,
}

#[derive(Deserialize)]
struct Notifications {
    #[serde(rename = "events", default)]
    event_config: Events,
    endpoints: Vec<EndPoint>,
}


#[derive(Deserialize)]
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

#[derive(Deserialize)]
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

#[derive(Deserialize)]
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
#[derive(Deserialize)]
struct Proxy {
    #[serde(rename = "remoteurl")]
    remote_url: String,
    username: String,
    // Password of the hub user
    password: String,
}

#[derive(Deserialize)]
struct Compatibility {
    schema1: Schema1, // `schema1` configures how schema1 manifests will be handled
}

#[derive(Deserialize)]
struct Schema1 {
    // `trust_key` is the signing key to use for adding the signature to
    // schema1 manifests.
    #[serde(rename = "signingkeyfile", default)]
    trust_key: String,
    #[serde(default)]
    // `enabled` determines if schema1 manifests should be pullable
    enabled: bool,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
struct Manifest {
    // `urls` configures validation for URLs in pushed manifests.
    urls: Urls,
}

#[derive(Deserialize)]
struct Urls {
    // allow` specifies regular expressions (https://godoc.org/regexp/syntax)
    // that URLs in pushed manifests must match.
    allow: Vec<String>,
    // `deny` specifies regular expressions (https://godoc.org/regexp/syntax)
    // that URLs in pushed manifests must not match.
    deny: Vec<String>,
}

#[derive(Deserialize, Default)]
struct Policy {
    repository: Repository,
}

#[derive(Deserialize, Default)]
struct Repository {
    classes: Vec<String>,
}

#[derive(Deserialize, Default)]
struct MailOptions {
    #[serde(default)]
    smtp: Smtp,
    #[serde(default)]
    from: String,
    #[serde(default)]
    to: Vec<String>,
}

#[derive(Deserialize, Default)]
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

#[derive(Deserialize)]
struct FileChecker {
    #[serde(default)]
    interval: Duration,
    #[serde(default)]
    file: String,
    #[serde(default)]
    threshold: u32,
}

#[derive(Deserialize, Default)]
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

#[derive(Deserialize, Default)]
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

#[derive(Deserialize, Default)]
struct StorageDriver {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    interval: Duration,
    #[serde(default)]

    threshold: u32,
}

#[derive(Deserialize, Default)]
struct Events {
    include_references: bool,
}

#[derive(Deserialize)]
struct EndPoint {
    name: String,
    disabled: bool,
    url: String,
    headers: Header,
    timeout: Duration,
    threshold: u32,
    backoff: Duration,
    #[serde(rename = "ignoredmediatypes")]
    ignore_media_type: Vec<String>,
    #[serde(default)]
    ignore: Ignore,
}

#[derive(Deserialize, Default)]
struct Ignore {
    media_types: Vec<String>,
    actions: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct BugsnagReporting {
    #[serde(rename = "apikey")]
    API_key: String,
    #[serde(rename = "releasestage")]
    release_stage: String,
    endpoint: String,
}

#[derive(Deserialize)]
struct NewRelicReporting {
    #[serde(rename = "licensekey")]
    license_key: String,
    name: String,
    verbose: bool,
}
