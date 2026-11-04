//! Captive DNS: answer every A query with the portal's address.
//!
//! Pure logic, no hardware: `host-tests/` runs the unit tests on the Mac.

const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;
/// Short TTL so the phone forgets the fake answers soon after setup.
const TTL_SECS: u32 = 10;

/// Build the reply to `query` in `out`. Returns the reply length, or `None`
/// for packets that are not a plain one-question query (they are dropped).
///
/// A queries for `IN` get one answer, `ip`. Every other type (AAAA, HTTPS,
/// ...) gets an empty NOERROR reply, so the phone falls back to IPv4.
pub fn answer(query: &[u8], ip: [u8; 4], out: &mut [u8]) -> Option<usize> {
    if query.len() < 12 {
        return None;
    }
    let flags = u16::from_be_bytes([query[2], query[3]]);
    let is_response = flags & 0x8000 != 0;
    let opcode = (flags >> 11) & 0xF;
    let qdcount = u16::from_be_bytes([query[4], query[5]]);
    if is_response || opcode != 0 || qdcount != 1 {
        return None;
    }

    // Walk the QNAME labels (no compression in a question).
    let mut i = 12;
    loop {
        let len = *query.get(i)? as usize;
        if len & 0xC0 != 0 {
            return None;
        }
        i += 1;
        if len == 0 {
            break;
        }
        i += len;
    }
    let qtype = u16::from_be_bytes([*query.get(i)?, *query.get(i + 1)?]);
    let qclass = u16::from_be_bytes([*query.get(i + 2)?, *query.get(i + 3)?]);
    let question_end = i + 4;

    let answer_a = qtype == TYPE_A && qclass == CLASS_IN;
    let len = question_end + if answer_a { 16 } else { 0 };
    if out.len() < len {
        return None;
    }

    out[..question_end].copy_from_slice(&query[..question_end]);
    // QR=1, keep opcode 0 and RD, set RA, RCODE=0 (NOERROR).
    let rd = flags & 0x0100;
    out[2..4].copy_from_slice(&(0x8000 | 0x0080 | rd).to_be_bytes());
    out[6..8].copy_from_slice(&(answer_a as u16).to_be_bytes()); // ANCOUNT
    out[8..12].fill(0); // NSCOUNT, ARCOUNT: drop any EDNS OPT record

    if answer_a {
        let a = &mut out[question_end..len];
        a[0..2].copy_from_slice(&0xC00Cu16.to_be_bytes()); // name: pointer to the question
        a[2..4].copy_from_slice(&TYPE_A.to_be_bytes());
        a[4..6].copy_from_slice(&CLASS_IN.to_be_bytes());
        a[6..10].copy_from_slice(&TTL_SECS.to_be_bytes());
        a[10..12].copy_from_slice(&4u16.to_be_bytes());
        a[12..16].copy_from_slice(&ip);
    }
    Some(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(name: &str, qtype: u16, edns: bool) -> alloc::vec::Vec<u8> {
        let mut q = alloc::vec![0x12, 0x34, 0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, edns as u8];
        for label in name.split('.') {
            q.push(label.len() as u8);
            q.extend_from_slice(label.as_bytes());
        }
        q.push(0);
        q.extend_from_slice(&qtype.to_be_bytes());
        q.extend_from_slice(&1u16.to_be_bytes());
        if edns {
            q.extend_from_slice(&[0, 0, 41, 0x05, 0xC0, 0, 0, 0, 0, 0, 0]);
        }
        q
    }

    #[test]
    fn answers_a_with_portal_ip() {
        let q = query("captive.apple.com", 1, true);
        let mut out = [0u8; 512];
        let n = answer(&q, [192, 168, 4, 1], &mut out).unwrap();
        let r = &out[..n];
        assert_eq!(&r[0..2], &[0x12, 0x34]); // same ID
        assert_eq!(r[2] & 0x80, 0x80); // response
        assert_eq!(r[3] & 0x0F, 0); // NOERROR
        assert_eq!(&r[4..12], &[0, 1, 0, 1, 0, 0, 0, 0]);
        let question_end = q.len() - 11;
        assert_eq!(&r[12..question_end], &q[12..question_end]);
        assert_eq!(&r[n - 4..], &[192, 168, 4, 1]);
        assert_eq!(n, question_end + 16);
    }

    #[test]
    fn empty_reply_for_aaaa() {
        let q = query("connectivitycheck.gstatic.com", 28, false);
        let mut out = [0u8; 512];
        let n = answer(&q, [192, 168, 4, 1], &mut out).unwrap();
        assert_eq!(n, q.len());
        assert_eq!(&out[6..8], &[0, 0]);
    }

    #[test]
    fn drops_garbage() {
        let mut out = [0u8; 512];
        assert_eq!(answer(&[0; 5], [1, 2, 3, 4], &mut out), None);
        let mut q = query("a.b", 1, false);
        q[2] |= 0x80; // a response, not a query
        assert_eq!(answer(&q, [1, 2, 3, 4], &mut out), None);
        let q = query("a.b", 1, false);
        assert_eq!(answer(&q[..q.len() - 2], [1, 2, 3, 4], &mut out), None);
        assert_eq!(answer(&q, [1, 2, 3, 4], &mut out[..10]), None);
    }
}
