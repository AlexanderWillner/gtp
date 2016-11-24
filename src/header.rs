use parser::{Parser, ParseResult};
use std::collections::hash_set::{HashSet};

// TODO: Add message type too.
#[derive(Debug)]
pub struct Gtp {
    pub version: Version,
    pub protocol: Protocol,
    pub flags: Flags,
    pub length: Length,
    pub teid: TunnelEid,
    pub seq_num: Option<SequenceNumber>,
    pub npdu_num: Option<NPduNumber>,
    pub next_ext_type: Option<NextExtHeaderType>,
    // TODO: Implement support for extension headers.
}

impl Gtp {
    pub fn parse(p: &mut Parser) -> ParseResult<Gtp> {
        let top   = p.parse_u8()?;
        let ver   = Version::parse(top)?;
        let proto = Protocol::parse(top)?;
        let flags = Flags::parse(top)?;
        let len   = Length::parse(p)?;
        let teid  = TunnelEid::parse(p)?;
        let seq_num = if flags.contains(&Flag::SequenceNumber) {
            SequenceNumber::parse(p).map(Some)?
        } else {
            None
        };
        let npdu_num = flags.parse_npdu(p)?;
        Ok(Gtp {
            version: ver,
            protocol: proto,
            flags: flags,
            length: len,
            teid: teid,
            seq_num: seq_num,
            npdu_num: npdu_num,
            next_ext_type: None
        })
    }
}

#[derive(Eq, Debug, PartialEq)]
pub struct Version(u8);

impl Version {
    pub fn parse(b: u8) -> ParseResult<Version>{
        Ok(Version(b >> 5))
    }
}

#[derive(Debug)]
pub enum Protocol {
    Gtp,
    GtpPrime,
}

impl Protocol {
    pub fn parse(b: u8) -> ParseResult<Protocol> {
        match b & 0b00100000 {
            0 => Ok(Protocol::GtpPrime),
            _ => Ok(Protocol::Gtp),
        }
    }
}

#[derive(Debug)]
pub struct Flags(HashSet<Flag>);

impl Flags {
    pub fn parse(b: u8) -> ParseResult<Self> {
        let mut res = HashSet::new();
        if Flag::has_npdu_number(b) { res.insert(Flag::NPduNumber); }
        if Flag::has_sequence_number(b) { res.insert(Flag::SequenceNumber); }
        if Flag::has_extension_header(b) { res.insert(Flag::ExtensionHeader); }
        Ok(Flags(res))
    }

    pub fn parse_npdu(&self, p: &mut Parser) -> ParseResult<Option<NPduNumber>> {
        if self.contains(&Flag::NPduNumber) {
            NPduNumber::parse(p).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn contains(&self, flag: &Flag) -> bool {
        self.0.contains(flag)
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Flag {
    NPduNumber,
    SequenceNumber,
    ExtensionHeader,
}

impl Flag {
    pub fn has_npdu_number(b: u8) -> bool {
        b & 0b00000001 != 0
    }

    pub fn has_sequence_number(b: u8) -> bool {
        b & 0b00000010 != 0
    }

    pub fn has_extension_header(b: u8) -> bool {
        b & 0b00000100 != 0
    }
}

pub enum MessageType {
    EchoRequest,   // TS29281, 7.2.1
    EchoResponse,  // TS29281, 7.2.2

}

#[derive(Debug, Eq, PartialEq)]
pub struct Length(u16);

impl Length {
    pub fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.parse_u16().map(Length)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TunnelEid(u32);

impl TunnelEid {
    pub fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.parse_u32().map(TunnelEid)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SequenceNumber(u16);

impl SequenceNumber {
    pub fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.parse_u16().map(SequenceNumber)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct NPduNumber(u8);

impl NPduNumber {
    pub fn parse(p: &mut Parser) -> ParseResult<Self> {
        p.parse_u8().map(NPduNumber)
    }
}

#[derive(Debug)]
pub enum NextExtHeaderType {
    EndReached,               // 00000000
    MbmsSupport,              // 00000001, Control
    MsInfoChangeReporting,    // 00000010, Control
    UdpPort,                  // 01000000, User
    PdcpPdu,                  // 11000000
    SuspendRequest,           // 11000001, Control
    SuspendResponse,          // 11000010, Control
}

#[derive(Debug)]
pub struct NextExtensionHeader {
    pub length: u8,
    pub content: Vec<u8>,
    pub next_ext_type: NextExtHeaderType
}


#[cfg(test)]
mod tests {
    use parser::Parser;
    use super::*;

    #[test]
    fn parse_minimal_header() {
        let raw = [0b00110000, 0, 0, 1, 0, 0, 0, 0];
        let mut p = Parser::new(&raw);
        let parsed = Gtp::parse(&mut p).unwrap();
        assert!(parsed.flags.0.is_empty());
        assert_eq!(parsed.version, Version(1));
        assert_eq!(parsed.length, Length(0));
        assert_eq!(parsed.teid, TunnelEid(1));
    }

    #[test]
    fn parse_basic_header() {
        let raw = [0b00110011, 0, 0, 1, 0, 0, 0, 14, 0, 5, 0];
        let mut p = Parser::new(&raw);
        let parsed = Gtp::parse(&mut p).unwrap();
        assert!(!parsed.flags.0.is_empty());
        assert_eq!(parsed.seq_num, Some(SequenceNumber(14)));
        assert_eq!(parsed.npdu_num, Some(NPduNumber(5)));
    }
}
