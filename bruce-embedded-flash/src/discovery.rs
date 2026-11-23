//! Finding the boards on the network by mDNS, see bruce-embedded-std's mdns.rs, which each advertises as a DNS-SD
//! service, `_bruce-device._tcp`, and `_http._tcp` too, as some routers pass only the common types between networks,
//! with facts about it in its TXT record: its ID, board, chip, firmware and version, whether it is adopted, and the
//! firmware's own, e.g. a capture device's site. Shared by the flasher, to say what a board on USB runs now, and apps,
//! e.g. Bruce's Smart Home, to find their devices without their addresses being typed in.
//!
//! A query is sent to the mDNS group from a port of its own, a one off query, which the boards answer straight back,
//! so that it works across networks which a router passes mDNS between, e.g. a computer on a house's main network
//! and the boards on its IoT network, as replies multicast on the boards' network do not always reach the computer's.

use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use simple_dns::rdata::RData;
use simple_dns::{CLASS, Name, Packet, QCLASS, QTYPE, Question, TYPE};

/// The service types asked for, see the module documentation.
const SERVICE_TYPES: [&str; 2] = ["_bruce-device._tcp.local", "_http._tcp.local"];

/// How long replies are gathered for, see browse.
pub const LISTEN: Duration = Duration::from_millis(1500);

/// A board found on the network, from its service's records.
#[derive(Debug, Clone, PartialEq)]
pub struct Found {
    /// Its ID, its Wi-Fi MAC address in hex, e.g. 44bd8d791738.
    pub id: String,
    /// Its address, with its port if it is not 80, e.g. 192.168.4.84, or a simulated device's 192.168.1.190:60496.
    pub address: String,
    /// Its name on the network, e.g. Wave147S3.local.
    pub hostname: String,
    /// The facts of its TXT record, e.g. board, fw, fwv, adopted and site.
    pub facts: BTreeMap<String, String>,
}

impl Found {
    pub fn fact(&self, key: &str) -> Option<&str> {
        self.facts.get(key).map(String::as_str)
    }

    /// What it says it is, e.g. "A Seeed Studio XIAO ESP32C6, running demo 0.3.0, on the network at 192.168.4.50",
    /// which is what it runs now, rather than when it was last identified, see seen.rs.
    pub fn describe(&self) -> String {
        let kind = self.fact("board").unwrap_or_default();
        let name = crate::a(crate::board_type(kind).map_or(kind, |board_type| board_type.name));
        format!(
            "{}{}, running {} {}, on the network at {}",
            name[..1].to_uppercase(),
            &name[1..],
            self.fact("fw").unwrap_or_default(),
            self.fact("fwv").unwrap_or_default(),
            self.address
        )
    }

    /// Whether an app has adopted it.
    pub fn is_adopted(&self) -> bool {
        self.fact("adopted") == Some("1")
    }
}

/// The boards which answer within LISTEN, which blocks for that long, so an app calls it on a thread of its own.
pub fn browse() -> anyhow::Result<Vec<Found>> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    socket.set_multicast_ttl_v4(255)?;
    socket.set_read_timeout(Some(Duration::from_millis(200)))?;
    let mut query = Packet::new_query(0x6272);
    for service_type in SERVICE_TYPES {
        query.questions.push(Question::new(
            Name::new_unchecked(service_type),
            QTYPE::TYPE(TYPE::PTR),
            QCLASS::CLASS(CLASS::IN),
            false,
        ));
    }
    socket.send_to(&query.build_bytes_vec()?, SocketAddr::from(([224, 0, 0, 251], 5353)))?;
    let mut replies = Vec::new();
    let mut buffer = [0u8; 9000];
    let end = Instant::now() + LISTEN;
    while Instant::now() < end {
        match socket.recv_from(&mut buffer) {
            Ok((length, from)) => replies.push((buffer[..length].to_vec(), from)),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(read_replies(&replies))
}

/// A service instance's host and port, from its SRV record, and its facts, from its TXT record, see read_replies.
type Instance = (Option<(String, u16)>, BTreeMap<String, String>);

/// The boards in replies, each a packet and where it came from, by putting together each service instance's SRV, TXT
/// and its host's A record, one board for each ID, as it advertises one instance of each type.
fn read_replies(replies: &[(Vec<u8>, SocketAddr)]) -> Vec<Found> {
    // By instance name: its host and port, and its facts; and each host's address.
    let mut services: BTreeMap<String, Instance> = BTreeMap::new();
    let mut addresses: BTreeMap<String, Ipv4Addr> = BTreeMap::new();
    for (bytes, _) in replies {
        let Ok(packet) = Packet::parse(bytes) else {
            continue;
        };
        for record in packet.answers.iter().chain(packet.additional_records.iter()) {
            let name = record.name.to_string().to_lowercase();
            match &record.rdata {
                RData::SRV(srv) => {
                    services.entry(name).or_default().0 = Some((srv.target.to_string().to_lowercase(), srv.port));
                }
                RData::TXT(txt) => {
                    let facts = &mut services.entry(name).or_default().1;
                    for (key, value) in txt.attributes() {
                        facts.insert(key, value.unwrap_or_default());
                    }
                }
                RData::A(a) => {
                    addresses.insert(name, Ipv4Addr::from(a.address));
                }
                _ => {}
            }
        }
    }
    let mut found: BTreeMap<String, Found> = BTreeMap::new();
    for (_, (target, facts)) in services {
        let (Some((hostname, port)), Some(id)) = (target, facts.get("id").cloned()) else {
            continue;
        };
        let Some(ip) = addresses.get(&hostname) else {
            continue;
        };
        let address = if port == 80 {
            ip.to_string()
        } else {
            format!("{ip}:{port}")
        };
        found.entry(id.clone()).or_insert(Found {
            id,
            address,
            hostname,
            facts,
        });
    }
    found.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_dns::ResourceRecord;
    use simple_dns::rdata::{A, PTR, SRV, TXT};

    /// A board's reply, as bruce-embedded-std's responder makes it.
    fn reply(service_type: &str, id: &str, host: &str, ip: [u8; 4], port: u16) -> Vec<u8> {
        let instance = format!("{id}.{service_type}");
        let mut packet = Packet::new_reply(0);
        packet.answers.push(ResourceRecord::new(
            Name::new_unchecked(service_type),
            CLASS::IN,
            4500,
            RData::PTR(PTR(Name::new_unchecked(&instance))),
        ));
        packet.additional_records.push(ResourceRecord::new(
            Name::new_unchecked(&instance),
            CLASS::IN,
            120,
            RData::SRV(SRV {
                priority: 0,
                weight: 0,
                port,
                target: Name::new_unchecked(host),
            }),
        ));
        let id_fact = format!("id={id}");
        let txt = TXT::new()
            .with_string(&id_fact)
            .unwrap()
            .with_string("fw=bruce-smart-home-capture")
            .unwrap()
            .with_string("adopted=1")
            .unwrap();
        packet.additional_records.push(ResourceRecord::new(
            Name::new_unchecked(&instance),
            CLASS::IN,
            4500,
            RData::TXT(txt),
        ));
        packet.additional_records.push(ResourceRecord::new(
            Name::new_unchecked(host),
            CLASS::IN,
            120,
            RData::A(A {
                address: u32::from(Ipv4Addr::from(ip)),
            }),
        ));
        packet.build_bytes_vec().unwrap()
    }

    /// Browses the network for real, run by hand with --ignored, to see what answers.
    #[test]
    #[ignore]
    fn browse_the_network() {
        for found in browse().unwrap() {
            println!(
                "{} at {} ({}): {:?}",
                found.id,
                found.address,
                found.hostname,
                found.facts.get("fw")
            );
        }
    }

    #[test]
    fn a_board_answering_for_both_types_is_found_once_with_its_address_and_facts() {
        let from = SocketAddr::from(([192, 168, 4, 84], 5353));
        let replies = vec![
            (
                reply(
                    "_bruce-device._tcp.local",
                    "44bd8d791738",
                    "Wave147S3.local",
                    [192, 168, 4, 84],
                    80,
                ),
                from,
            ),
            (
                reply(
                    "_http._tcp.local",
                    "44bd8d791738",
                    "Wave147S3.local",
                    [192, 168, 4, 84],
                    80,
                ),
                from,
            ),
            (
                reply(
                    "_http._tcp.local",
                    "f85b1b909f50",
                    "Guitron.local",
                    [192, 168, 4, 83],
                    80,
                ),
                from,
            ),
            (
                reply(
                    "_bruce-device._tcp.local",
                    "aabbccddeeff",
                    "Sim.local",
                    [192, 168, 1, 190],
                    60496,
                ),
                from,
            ),
        ];
        let found = read_replies(&replies);
        assert_eq!(found.len(), 3);
        assert_eq!(found[0].id, "44bd8d791738");
        assert_eq!(found[0].address, "192.168.4.84");
        assert_eq!(found[0].fact("fw"), Some("bruce-smart-home-capture"));
        assert!(found[0].is_adopted());
        assert_eq!(found[1].address, "192.168.1.190:60496", "a port other than 80 is kept");
        assert_eq!(found[2].id, "f85b1b909f50");
    }
}
