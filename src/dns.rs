use std::io;
use std::net::UdpSocket;
use std::time::Duration;

const DNS_TIMEOUT_SECS: u64 = 3;

fn encode_domain_name(domain: &str) -> Vec<u8> {
    let mut result = Vec::new();
    for label in domain.split('.') {
        let label_bytes = label.as_bytes();
        result.push(label_bytes.len() as u8);
        result.extend_from_slice(label_bytes);
    }
    result.push(0);
    result
}

fn parse_domain_name(data: &[u8], start: usize) -> (String, usize) {
    let mut name = String::new();
    let mut pos = start;
    let mut jumped = false;
    let mut jump_pos = 0;
    let mut seen_pointers = 0;

    loop {
        if pos >= data.len() {
            break;
        }
        let len = data[pos] as usize;

        if len == 0 {
            if jumped {
                pos = jump_pos;
            } else {
                pos += 1;
            }
            break;
        }

        if (len & 0xC0) == 0xC0 {
            if pos + 1 >= data.len() {
                break;
            }
            let offset = ((data[pos] as usize & 0x3F) << 8) | (data[pos + 1] as usize);
            if !jumped {
                jump_pos = pos + 2;
                jumped = true;
            }
            pos = offset;
            seen_pointers += 1;
            if seen_pointers > 10 {
                break;
            }
            continue;
        }

        pos += 1;
        if pos + len > data.len() {
            break;
        }
        if !name.is_empty() {
            name.push('.');
        }
        if let Ok(label) = std::str::from_utf8(&data[pos..pos + len]) {
            name.push_str(label);
        }
        pos += len;
    }

    (name, pos)
}

pub fn resolve_via_dns(domain: &str, dns_server: &str) -> io::Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(DNS_TIMEOUT_SECS)))?;
    socket.set_write_timeout(Some(Duration::from_secs(DNS_TIMEOUT_SECS)))?;

    let mut query = Vec::new();
    query.push(0x12);
    query.push(0x34);
    query.push(0x01);
    query.push(0x00);
    query.push(0x00);
    query.push(0x01);
    query.push(0x00);
    query.push(0x00);
    query.push(0x00);
    query.push(0x00);
    query.push(0x00);
    query.push(0x00);
    query.extend(encode_domain_name(domain));
    query.push(0x00);
    query.push(0x01);
    query.push(0x00);
    query.push(0x01);

    let dns_addr = if dns_server.contains(':') {
        dns_server.to_string()
    } else {
        format!("{}:53", dns_server)
    };

    socket.send_to(&query, dns_addr)?;

    let mut response = [0u8; 512];
    let (len, _) = socket.recv_from(&mut response)?;
    let data = &response[..len];

    if len < 12 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "DNS response too short",
        ));
    }

    let flags = ((data[2] as u16) << 8) | (data[3] as u16);
    let rcode = flags & 0x000F;
    if rcode != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("DNS error code: {}", rcode),
        ));
    }

    let question_count = ((data[4] as u16) << 8) | (data[5] as u16);
    let answer_count = ((data[6] as u16) << 8) | (data[7] as u16);

    if answer_count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No DNS answers",
        ));
    }

    let mut pos = 12;
    for _ in 0..question_count {
        let (_, q_end) = parse_domain_name(data, pos);
        pos = q_end + 4;
    }

    for _ in 0..answer_count {
        if pos >= data.len() {
            break;
        }
        let (_, rr_name_end) = parse_domain_name(data, pos);
        pos = rr_name_end;

        if pos + 10 > data.len() {
            break;
        }
        let rtype = ((data[pos] as u16) << 8) | (data[pos + 1] as u16);
        let rdlength = ((data[pos + 8] as u16) << 8) | (data[pos + 9] as u16);
        pos += 10;

        if rtype == 1 && rdlength == 4 && pos + 4 <= data.len() {
            let ip = format!(
                "{}.{}.{}.{}",
                data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
            );
            return Ok(ip);
        }
        pos += rdlength as usize;
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No A record found",
    ))
}
