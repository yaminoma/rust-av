#![allow(dead_code)]

use std::io::SeekFrom;

use buffer::Buffered;
use data::packet::Packet;
use stream::Stream;
use demuxer::context::GlobalInfo;
use error::*;

#[derive(Clone,Debug,PartialEq)]
pub enum Event {
    NewPacket(Packet),
    NewStream(Stream),
    MoreDataNeeded
}

pub trait Demuxer {
    fn open(&mut self);
    fn read_headers(&mut self, buf: &Box<Buffered>, info: &mut GlobalInfo) -> Result<SeekFrom>;
    fn read_packet(&mut self, buf: &Box<Buffered>) -> Result<(SeekFrom, Event)>;
}

pub struct DemuxerDescription {
    pub name:        &'static str,
    pub description: &'static str,
    pub extensions:  &'static [&'static str],
    pub mime:        &'static [&'static str],
}

/// Least amount of data needed to check the bytestream structure
/// to match some known format.
pub const PROBE_DATA: usize = 4 * 1024;

/// Probe threshold values
pub enum Score {
    /// Minimum acceptable value, a file matched just by the extension
    EXTENSION = 50,
    /// The underlying layer provides the information, trust it up to a point
    MIME = 75,
    /// The data actually match a format structure
    MAX = 100,
}

pub trait DemuxerBuilder {
    fn describe(&self) -> &'static DemuxerDescription;
    fn probe(&self, data: &[u8]) -> u8;
    // cannot use impl Demuxer as return type of a trait method yet
    fn alloc(&self) -> Box<Demuxer>;
}

pub fn probe<'a>(demuxers: &[&'static DemuxerBuilder],
                 data: &[u8])
                 -> Option<&'a DemuxerBuilder> {
    let mut max = u8::min_value();
    let mut candidate: Option<&DemuxerBuilder> = None;
    for builder in demuxers {
        let score = builder.probe(data);

        if score > max {
            max = score;
            candidate = Some(*builder);
        }
    }

    if max > Score::EXTENSION as u8 {
        candidate
    } else {
        None
    }
}

#[macro_export]
macro_rules! module {
    {
        ($name:ident) {
            open($os:ident) => $ob:block
            read_headers($rhs:ident, $rhctx:ident, $rhi:ident) => $rhb:block
            read_packet($rps:ident, $rpctx:ident) => $rpb:block

            describe($ds:ident) => $db:block
            probe($ps:ident, $pd:ident) => $pb:block
            alloc($asel:ident) => $ab:block
        }
    } => {
        interpolate_idents! {
            struct [$name Demuxer];
            struct [$name DemuxerBuilder];

            impl Demuxer for [$name Demuxer] {
                fn open(&mut $os) $ob
                fn read_headers(&mut $rhs, $rhctx: &Box<Buffered>, $rhi: &mut GlobalInfo) -> Result<SeekFrom> $rhb
                fn read_packet(&mut $rps, $rpctx: &Box<Buffered>) -> Result<(SeekFrom, Event)> $rpb
            }

            impl DemuxerBuilder for [$name DemuxerBuilder] {
                fn describe(&$ds) -> &'static DemuxerDescription $db
                fn probe(&$ps, $pd: &[[u8]]) -> u8 $pb
                fn alloc(&$asel) -> Box<Demuxer> $ab
            }
        }
    }
}


#[cfg(test)]
mod test {
    #![allow(dead_code)]
    #![allow(unused_variables)]
    use super::*;
    use std::io::Error;
    use data::packet::Packet;
    module! {
        (Test) {
            open(self) => { () }
            read_headers(self, buf, info) => { Ok(SeekFrom::Current(0)) }
            read_packet(self, buf) => { unimplemented!() }

            describe(self) => {
                const D: &'static DemuxerDescription = &DemuxerDescription {
                    name: "Test",
                    description: "Test demuxer",
                    extensions: &["test", "t"],
                    mime: &["x-application/test"],
                };

                D
            }

            probe(self, data) => {
                if data[0] == 0 {
                    Score::MAX as u8
                } else {
                    0
                }
            }

            alloc(self) => {
                let demux = TestDemuxer {};

                box demux
            }
        }
    }

    const DEMUXER_BUILDERS: [&'static DemuxerBuilder; 1] = [&TestDemuxerBuilder {}];

    #[test]
    fn probe_demuxer() {
        let mut buf = [1; PROBE_DATA];

        match probe(&DEMUXER_BUILDERS, &buf) {
            Some(_) => panic!(),
            None => (),
        };

        buf[0] = 0;

        match probe(&DEMUXER_BUILDERS, &buf) {
            Some(_) => (),
            None => panic!(),
        };
    }
}
