use std::net::IpAddr;
use std::time::Duration;

use dns_lookup::lookup_addr;
use maxminddb::geoip2;

#[derive(Default)]
pub struct GeoResult {
    pub country: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
}

pub fn resolve_hostname(ip: IpAddr) -> String {
    lookup_addr(&ip).unwrap_or_default()
}

pub fn lookup_geo(geoip: &maxminddb::Reader<Vec<u8>>, ip: IpAddr) -> GeoResult {
    let geo: Option<geoip2::City> = geoip.lookup(ip).ok();

    let country = geo
        .as_ref()
        .and_then(|g| g.country.as_ref())
        .and_then(|c| c.iso_code)
        .unwrap_or_default()
        .to_string();

    let city = geo
        .as_ref()
        .and_then(|g| g.city.as_ref())
        .and_then(|c| c.names.as_ref())
        .and_then(|n| n.get("en").copied())
        .unwrap_or_default()
        .to_string();

    let (latitude, longitude) = geo
        .as_ref()
        .and_then(|g| g.location.as_ref())
        .map(|loc| (loc.latitude.unwrap_or(0.0), loc.longitude.unwrap_or(0.0)))
        .unwrap_or((0.0, 0.0));

    GeoResult {
        country,
        city,
        latitude,
        longitude,
    }
}

pub fn probe_http(ip: IpAddr) -> (bool, bool) {
    let agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_millis(500)))
        .timeout_recv_response(Some(Duration::from_millis(500)))
        .build()
        .new_agent();

    match agent.head(&format!("http://{ip}/")).call() {
        Ok(resp) => {
            let had_date = resp.headers().get("date").is_some();
            (true, had_date)
        }
        Err(ureq::Error::StatusCode(_)) => (true, false),
        Err(_) => (false, false),
    }
}

pub fn ping_rtt(ip: IpAddr) -> Option<f64> {
    let timeout = Duration::from_secs(2);
    let data = [0u8; 8];
    let options = ping_rs::PingOptions {
        ttl: 128,
        dont_fragment: false,
    };
    let reply = ping_rs::send_ping(&ip, timeout, &data, Some(&options)).ok()?;
    Some(reply.rtt as f64)
}
