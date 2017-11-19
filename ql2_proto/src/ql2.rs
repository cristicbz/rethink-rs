//! Automatically generated rust module for 'ql2.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::io::Write;
use std::borrow::Cow;
use quick_protobuf::{MessageWrite, BytesReader, Writer, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct VersionDummy {}

impl VersionDummy {
    pub fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for VersionDummy {}

pub mod mod_VersionDummy {


    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum Version {
        V0_1 = 1063369270,
        V0_2 = 1915781601,
        V0_3 = 1601562686,
        V0_4 = 1074539808,
        V1_0 = 885177795,
    }

    impl Default for Version {
        fn default() -> Self {
            Version::V0_1
        }
    }

    impl From<i32> for Version {
        fn from(i: i32) -> Self {
            match i {
                1063369270 => Version::V0_1,
                1915781601 => Version::V0_2,
                1601562686 => Version::V0_3,
                1074539808 => Version::V0_4,
                885177795 => Version::V1_0,
                _ => Self::default(),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum Protocol {
        PROTOBUF = 656407617,
        JSON = 2120839367,
    }

    impl Default for Protocol {
        fn default() -> Self {
            Protocol::PROTOBUF
        }
    }

    impl From<i32> for Protocol {
        fn from(i: i32) -> Self {
            match i {
                656407617 => Protocol::PROTOBUF,
                2120839367 => Protocol::JSON,
                _ => Self::default(),
            }
        }
    }

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Query<'a> {
    pub type_pb: Option<mod_Query::QueryType>,
    pub query: Option<Term<'a>>,
    pub token: Option<i64>,
    pub OBSOLETE_noreply: bool,
    pub accepts_r_json: bool,
    pub global_optargs: Vec<mod_Query::AssocPair<'a>>,
}

impl<'a> Query<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.type_pb = Some(r.read_enum(bytes)?),
                Ok(18) => msg.query = Some(r.read_message(bytes, Term::from_reader)?),
                Ok(24) => msg.token = Some(r.read_int64(bytes)?),
                Ok(32) => msg.OBSOLETE_noreply = r.read_bool(bytes)?,
                Ok(40) => msg.accepts_r_json = r.read_bool(bytes)?,
                Ok(50) => {
                    msg.global_optargs.push(r.read_message(
                        bytes,
                        mod_Query::AssocPair::from_reader,
                    )?)
                }
                Ok(t) => {
                    r.read_unknown(bytes, t)?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Query<'a> {
    fn get_size(&self) -> usize {
        0 +
            self.type_pb.as_ref().map_or(
                0,
                |m| 1 + sizeof_varint(*(m) as u64),
            ) +
            self.query.as_ref().map_or(
                0,
                |m| 1 + sizeof_len((m).get_size()),
            ) +
            self.token.as_ref().map_or(
                0,
                |m| 1 + sizeof_varint(*(m) as u64),
            ) +
            if self.OBSOLETE_noreply == false {
                0
            } else {
                1 + sizeof_varint(*(&self.OBSOLETE_noreply) as u64)
            } +
            if self.accepts_r_json == false {
                0
            } else {
                1 + sizeof_varint(*(&self.accepts_r_json) as u64)
            } +
            self.global_optargs
                .iter()
                .map(|s| 1 + sizeof_len((s).get_size()))
                .sum::<usize>()
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.type_pb {
            w.write_with_tag(8, |w| w.write_enum(*s as i32))?;
        }
        if let Some(ref s) = self.query {
            w.write_with_tag(18, |w| w.write_message(s))?;
        }
        if let Some(ref s) = self.token {
            w.write_with_tag(24, |w| w.write_int64(*s))?;
        }
        if self.OBSOLETE_noreply != false {
            w.write_with_tag(
                32,
                |w| w.write_bool(*&self.OBSOLETE_noreply),
            )?;
        }
        if self.accepts_r_json != false {
            w.write_with_tag(
                40,
                |w| w.write_bool(*&self.accepts_r_json),
            )?;
        }
        for s in &self.global_optargs {
            w.write_with_tag(50, |w| w.write_message(s))?;
        }
        Ok(())
    }
}

pub mod mod_Query {

    use std::borrow::Cow;
    use super::*;

    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct AssocPair<'a> {
        pub key: Option<Cow<'a, str>>,
        pub val: Option<Box<Term<'a>>>,
    }

    impl<'a> AssocPair<'a> {
        pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(10) => msg.key = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                    Ok(18) => msg.val = Some(Box::new(r.read_message(bytes, Term::from_reader)?)),
                    Ok(t) => {
                        r.read_unknown(bytes, t)?;
                    }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for AssocPair<'a> {
        fn get_size(&self) -> usize {
            0 + self.key.as_ref().map_or(0, |m| 1 + sizeof_len((m).len())) +
                self.val.as_ref().map_or(
                    0,
                    |m| 1 + sizeof_len((m).get_size()),
                )
        }

        fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
            if let Some(ref s) = self.key {
                w.write_with_tag(10, |w| w.write_string(&**s))?;
            }
            if let Some(ref s) = self.val {
                w.write_with_tag(18, |w| w.write_message(&**s))?;
            }
            Ok(())
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum QueryType {
        START = 1,
        CONTINUE = 2,
        STOP = 3,
        NOREPLY_WAIT = 4,
        SERVER_INFO = 5,
    }

    impl Default for QueryType {
        fn default() -> Self {
            QueryType::START
        }
    }

    impl From<i32> for QueryType {
        fn from(i: i32) -> Self {
            match i {
                1 => QueryType::START,
                2 => QueryType::CONTINUE,
                3 => QueryType::STOP,
                4 => QueryType::NOREPLY_WAIT,
                5 => QueryType::SERVER_INFO,
                _ => Self::default(),
            }
        }
    }

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Frame<'a> {
    pub type_pb: Option<mod_Frame::FrameType>,
    pub pos: Option<i64>,
    pub opt: Option<Cow<'a, str>>,
}

impl<'a> Frame<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.type_pb = Some(r.read_enum(bytes)?),
                Ok(16) => msg.pos = Some(r.read_int64(bytes)?),
                Ok(26) => msg.opt = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => {
                    r.read_unknown(bytes, t)?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Frame<'a> {
    fn get_size(&self) -> usize {
        0 +
            self.type_pb.as_ref().map_or(
                0,
                |m| 1 + sizeof_varint(*(m) as u64),
            ) +
            self.pos.as_ref().map_or(
                0,
                |m| 1 + sizeof_varint(*(m) as u64),
            ) + self.opt.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.type_pb {
            w.write_with_tag(8, |w| w.write_enum(*s as i32))?;
        }
        if let Some(ref s) = self.pos {
            w.write_with_tag(16, |w| w.write_int64(*s))?;
        }
        if let Some(ref s) = self.opt {
            w.write_with_tag(26, |w| w.write_string(&**s))?;
        }
        Ok(())
    }
}

pub mod mod_Frame {


    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum FrameType {
        POS = 1,
        OPT = 2,
    }

    impl Default for FrameType {
        fn default() -> Self {
            FrameType::POS
        }
    }

    impl From<i32> for FrameType {
        fn from(i: i32) -> Self {
            match i {
                1 => FrameType::POS,
                2 => FrameType::OPT,
                _ => Self::default(),
            }
        }
    }

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Backtrace<'a> {
    pub frames: Vec<Frame<'a>>,
}

impl<'a> Backtrace<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.frames.push(r.read_message(bytes, Frame::from_reader)?),
                Ok(t) => {
                    r.read_unknown(bytes, t)?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Backtrace<'a> {
    fn get_size(&self) -> usize {
        0 +
            self.frames
                .iter()
                .map(|s| 1 + sizeof_len((s).get_size()))
                .sum::<usize>()
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.frames {
            w.write_with_tag(10, |w| w.write_message(s))?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Response<'a> {
    pub type_pb: Option<mod_Response::ResponseType>,
    pub error_type: Option<mod_Response::ErrorType>,
    pub notes: Vec<mod_Response::ResponseNote>,
    pub token: Option<i64>,
    pub response: Vec<Datum<'a>>,
    pub backtrace: Option<Backtrace<'a>>,
    pub profile: Option<Datum<'a>>,
}

impl<'a> Response<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.type_pb = Some(r.read_enum(bytes)?),
                Ok(56) => msg.error_type = Some(r.read_enum(bytes)?),
                Ok(48) => msg.notes.push(r.read_enum(bytes)?),
                Ok(16) => msg.token = Some(r.read_int64(bytes)?),
                Ok(26) => {
                    msg.response.push(
                        r.read_message(bytes, Datum::from_reader)?,
                    )
                }
                Ok(34) => msg.backtrace = Some(r.read_message(bytes, Backtrace::from_reader)?),
                Ok(42) => msg.profile = Some(r.read_message(bytes, Datum::from_reader)?),
                Ok(t) => {
                    r.read_unknown(bytes, t)?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Response<'a> {
    fn get_size(&self) -> usize {
        0 +
            self.type_pb.as_ref().map_or(
                0,
                |m| 1 + sizeof_varint(*(m) as u64),
            ) +
            self.error_type.as_ref().map_or(
                0,
                |m| 1 + sizeof_varint(*(m) as u64),
            ) +
            self.notes
                .iter()
                .map(|s| 1 + sizeof_varint(*(s) as u64))
                .sum::<usize>() +
            self.token.as_ref().map_or(
                0,
                |m| 1 + sizeof_varint(*(m) as u64),
            ) +
            self.response
                .iter()
                .map(|s| 1 + sizeof_len((s).get_size()))
                .sum::<usize>() +
            self.backtrace.as_ref().map_or(
                0,
                |m| 1 + sizeof_len((m).get_size()),
            ) +
            self.profile.as_ref().map_or(
                0,
                |m| 1 + sizeof_len((m).get_size()),
            )
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.type_pb {
            w.write_with_tag(8, |w| w.write_enum(*s as i32))?;
        }
        if let Some(ref s) = self.error_type {
            w.write_with_tag(56, |w| w.write_enum(*s as i32))?;
        }
        for s in &self.notes {
            w.write_with_tag(48, |w| w.write_enum(*s as i32))?;
        }
        if let Some(ref s) = self.token {
            w.write_with_tag(16, |w| w.write_int64(*s))?;
        }
        for s in &self.response {
            w.write_with_tag(26, |w| w.write_message(s))?;
        }
        if let Some(ref s) = self.backtrace {
            w.write_with_tag(34, |w| w.write_message(s))?;
        }
        if let Some(ref s) = self.profile {
            w.write_with_tag(42, |w| w.write_message(s))?;
        }
        Ok(())
    }
}

pub mod mod_Response {


    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum ResponseType {
        SUCCESS_ATOM = 1,
        SUCCESS_SEQUENCE = 2,
        SUCCESS_PARTIAL = 3,
        WAIT_COMPLETE = 4,
        SERVER_INFO = 5,
        CLIENT_ERROR = 16,
        COMPILE_ERROR = 17,
        RUNTIME_ERROR = 18,
    }

    impl Default for ResponseType {
        fn default() -> Self {
            ResponseType::SUCCESS_ATOM
        }
    }

    impl From<i32> for ResponseType {
        fn from(i: i32) -> Self {
            match i {
                1 => ResponseType::SUCCESS_ATOM,
                2 => ResponseType::SUCCESS_SEQUENCE,
                3 => ResponseType::SUCCESS_PARTIAL,
                4 => ResponseType::WAIT_COMPLETE,
                5 => ResponseType::SERVER_INFO,
                16 => ResponseType::CLIENT_ERROR,
                17 => ResponseType::COMPILE_ERROR,
                18 => ResponseType::RUNTIME_ERROR,
                _ => Self::default(),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum ErrorType {
        INTERNAL = 1000000,
        RESOURCE_LIMIT = 2000000,
        QUERY_LOGIC = 3000000,
        NON_EXISTENCE = 3100000,
        OP_FAILED = 4100000,
        OP_INDETERMINATE = 4200000,
        USER = 5000000,
        PERMISSION_ERROR = 6000000,
    }

    impl Default for ErrorType {
        fn default() -> Self {
            ErrorType::INTERNAL
        }
    }

    impl From<i32> for ErrorType {
        fn from(i: i32) -> Self {
            match i {
                1000000 => ErrorType::INTERNAL,
                2000000 => ErrorType::RESOURCE_LIMIT,
                3000000 => ErrorType::QUERY_LOGIC,
                3100000 => ErrorType::NON_EXISTENCE,
                4100000 => ErrorType::OP_FAILED,
                4200000 => ErrorType::OP_INDETERMINATE,
                5000000 => ErrorType::USER,
                6000000 => ErrorType::PERMISSION_ERROR,
                _ => Self::default(),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum ResponseNote {
        SEQUENCE_FEED = 1,
        ATOM_FEED = 2,
        ORDER_BY_LIMIT_FEED = 3,
        UNIONED_FEED = 4,
        INCLUDES_STATES = 5,
    }

    impl Default for ResponseNote {
        fn default() -> Self {
            ResponseNote::SEQUENCE_FEED
        }
    }

    impl From<i32> for ResponseNote {
        fn from(i: i32) -> Self {
            match i {
                1 => ResponseNote::SEQUENCE_FEED,
                2 => ResponseNote::ATOM_FEED,
                3 => ResponseNote::ORDER_BY_LIMIT_FEED,
                4 => ResponseNote::UNIONED_FEED,
                5 => ResponseNote::INCLUDES_STATES,
                _ => Self::default(),
            }
        }
    }

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Datum<'a> {
    pub type_pb: Option<mod_Datum::DatumType>,
    pub r_bool: Option<bool>,
    pub r_num: Option<f64>,
    pub r_str: Option<Cow<'a, str>>,
    pub r_array: Vec<Datum<'a>>,
    pub r_object: Vec<mod_Query::AssocPair<'a>>,
}

impl<'a> Datum<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.type_pb = Some(r.read_enum(bytes)?),
                Ok(16) => msg.r_bool = Some(r.read_bool(bytes)?),
                Ok(25) => msg.r_num = Some(r.read_double(bytes)?),
                Ok(34) => msg.r_str = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(42) => msg.r_array.push(r.read_message(bytes, Datum::from_reader)?),
                Ok(50) => {
                    msg.r_object.push(r.read_message(
                        bytes,
                        mod_Query::AssocPair::from_reader,
                    )?)
                }
                Ok(t) => {
                    r.read_unknown(bytes, t)?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Datum<'a> {
    fn get_size(&self) -> usize {
        0 +
            self.type_pb.as_ref().map_or(
                0,
                |m| 1 + sizeof_varint(*(m) as u64),
            ) +
            self.r_bool.as_ref().map_or(
                0,
                |m| 1 + sizeof_varint(*(m) as u64),
            ) + self.r_num.as_ref().map_or(0, |_| 1 + 8) +
            self.r_str.as_ref().map_or(0, |m| 1 + sizeof_len((m).len())) +
            self.r_array
                .iter()
                .map(|s| 1 + sizeof_len((s).get_size()))
                .sum::<usize>() +
            self.r_object
                .iter()
                .map(|s| 1 + sizeof_len((s).get_size()))
                .sum::<usize>()
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.type_pb {
            w.write_with_tag(8, |w| w.write_enum(*s as i32))?;
        }
        if let Some(ref s) = self.r_bool {
            w.write_with_tag(16, |w| w.write_bool(*s))?;
        }
        if let Some(ref s) = self.r_num {
            w.write_with_tag(25, |w| w.write_double(*s))?;
        }
        if let Some(ref s) = self.r_str {
            w.write_with_tag(34, |w| w.write_string(&**s))?;
        }
        for s in &self.r_array {
            w.write_with_tag(42, |w| w.write_message(s))?;
        }
        for s in &self.r_object {
            w.write_with_tag(50, |w| w.write_message(s))?;
        }
        Ok(())
    }
}

pub mod mod_Datum {

    use std::borrow::Cow;
    use super::*;

    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct AssocPair<'a> {
        pub key: Option<Cow<'a, str>>,
        pub val: Option<Box<Datum<'a>>>,
    }

    impl<'a> AssocPair<'a> {
        pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(10) => msg.key = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                    Ok(18) => msg.val = Some(Box::new(r.read_message(bytes, Datum::from_reader)?)),
                    Ok(t) => {
                        r.read_unknown(bytes, t)?;
                    }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for AssocPair<'a> {
        fn get_size(&self) -> usize {
            0 + self.key.as_ref().map_or(0, |m| 1 + sizeof_len((m).len())) +
                self.val.as_ref().map_or(
                    0,
                    |m| 1 + sizeof_len((m).get_size()),
                )
        }

        fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
            if let Some(ref s) = self.key {
                w.write_with_tag(10, |w| w.write_string(&**s))?;
            }
            if let Some(ref s) = self.val {
                w.write_with_tag(18, |w| w.write_message(&**s))?;
            }
            Ok(())
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum DatumType {
        R_NULL = 1,
        R_BOOL = 2,
        R_NUM = 3,
        R_STR = 4,
        R_ARRAY = 5,
        R_OBJECT = 6,
        R_JSON = 7,
    }

    impl Default for DatumType {
        fn default() -> Self {
            DatumType::R_NULL
        }
    }

    impl From<i32> for DatumType {
        fn from(i: i32) -> Self {
            match i {
                1 => DatumType::R_NULL,
                2 => DatumType::R_BOOL,
                3 => DatumType::R_NUM,
                4 => DatumType::R_STR,
                5 => DatumType::R_ARRAY,
                6 => DatumType::R_OBJECT,
                7 => DatumType::R_JSON,
                _ => Self::default(),
            }
        }
    }

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Term<'a> {
    pub type_pb: Option<mod_Term::TermType>,
    pub datum: Option<Datum<'a>>,
    pub args: Vec<Term<'a>>,
    pub optargs: Vec<mod_Query::AssocPair<'a>>,
}

impl<'a> Term<'a> {
    pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.type_pb = Some(r.read_enum(bytes)?),
                Ok(18) => msg.datum = Some(r.read_message(bytes, Datum::from_reader)?),
                Ok(26) => msg.args.push(r.read_message(bytes, Term::from_reader)?),
                Ok(34) => {
                    msg.optargs.push(r.read_message(
                        bytes,
                        mod_Query::AssocPair::from_reader,
                    )?)
                }
                Ok(t) => {
                    r.read_unknown(bytes, t)?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Term<'a> {
    fn get_size(&self) -> usize {
        0 +
            self.type_pb.as_ref().map_or(
                0,
                |m| 1 + sizeof_varint(*(m) as u64),
            ) +
            self.datum.as_ref().map_or(
                0,
                |m| 1 + sizeof_len((m).get_size()),
            ) +
            self.args
                .iter()
                .map(|s| 1 + sizeof_len((s).get_size()))
                .sum::<usize>() +
            self.optargs
                .iter()
                .map(|s| 1 + sizeof_len((s).get_size()))
                .sum::<usize>()
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.type_pb {
            w.write_with_tag(8, |w| w.write_enum(*s as i32))?;
        }
        if let Some(ref s) = self.datum {
            w.write_with_tag(18, |w| w.write_message(s))?;
        }
        for s in &self.args {
            w.write_with_tag(26, |w| w.write_message(s))?;
        }
        for s in &self.optargs {
            w.write_with_tag(34, |w| w.write_message(s))?;
        }
        Ok(())
    }
}

pub mod mod_Term {

    use std::borrow::Cow;
    use super::*;

    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct AssocPair<'a> {
        pub key: Option<Cow<'a, str>>,
        pub val: Option<Box<Term<'a>>>,
    }

    impl<'a> AssocPair<'a> {
        pub fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(10) => msg.key = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                    Ok(18) => msg.val = Some(Box::new(r.read_message(bytes, Term::from_reader)?)),
                    Ok(t) => {
                        r.read_unknown(bytes, t)?;
                    }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for AssocPair<'a> {
        fn get_size(&self) -> usize {
            0 + self.key.as_ref().map_or(0, |m| 1 + sizeof_len((m).len())) +
                self.val.as_ref().map_or(
                    0,
                    |m| 1 + sizeof_len((m).get_size()),
                )
        }

        fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
            if let Some(ref s) = self.key {
                w.write_with_tag(10, |w| w.write_string(&**s))?;
            }
            if let Some(ref s) = self.val {
                w.write_with_tag(18, |w| w.write_message(&**s))?;
            }
            Ok(())
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum TermType {
        DATUM = 1,
        MAKE_ARRAY = 2,
        MAKE_OBJ = 3,
        VAR = 10,
        JAVASCRIPT = 11,
        UUID = 169,
        HTTP = 153,
        ERROR = 12,
        IMPLICIT_VAR = 13,
        DB = 14,
        TABLE = 15,
        GET = 16,
        GET_ALL = 78,
        EQ = 17,
        NE = 18,
        LT = 19,
        LE = 20,
        GT = 21,
        GE = 22,
        NOT = 23,
        ADD = 24,
        SUB = 25,
        MUL = 26,
        DIV = 27,
        MOD = 28,
        FLOOR = 183,
        CEIL = 184,
        ROUND = 185,
        APPEND = 29,
        PREPEND = 80,
        DIFFERENCE = 95,
        SET_INSERT = 88,
        SET_INTERSECTION = 89,
        SET_UNION = 90,
        SET_DIFFERENCE = 91,
        SLICE = 30,
        SKIP = 70,
        LIMIT = 71,
        OFFSETS_OF = 87,
        CONTAINS = 93,
        GET_FIELD = 31,
        KEYS = 94,
        VALUES = 186,
        OBJECT = 143,
        HAS_FIELDS = 32,
        WITH_FIELDS = 96,
        PLUCK = 33,
        WITHOUT = 34,
        MERGE = 35,
        BETWEEN_DEPRECATED = 36,
        BETWEEN = 182,
        REDUCE = 37,
        MAP = 38,
        FOLD = 187,
        FILTER = 39,
        CONCAT_MAP = 40,
        ORDER_BY = 41,
        DISTINCT = 42,
        COUNT = 43,
        IS_EMPTY = 86,
        UNION = 44,
        NTH = 45,
        BRACKET = 170,
        INNER_JOIN = 48,
        OUTER_JOIN = 49,
        EQ_JOIN = 50,
        ZIP = 72,
        RANGE = 173,
        INSERT_AT = 82,
        DELETE_AT = 83,
        CHANGE_AT = 84,
        SPLICE_AT = 85,
        COERCE_TO = 51,
        TYPE_OF = 52,
        UPDATE = 53,
        DELETE = 54,
        REPLACE = 55,
        INSERT = 56,
        DB_CREATE = 57,
        DB_DROP = 58,
        DB_LIST = 59,
        TABLE_CREATE = 60,
        TABLE_DROP = 61,
        TABLE_LIST = 62,
        CONFIG = 174,
        STATUS = 175,
        WAIT = 177,
        RECONFIGURE = 176,
        REBALANCE = 179,
        SYNC = 138,
        GRANT = 188,
        INDEX_CREATE = 75,
        INDEX_DROP = 76,
        INDEX_LIST = 77,
        INDEX_STATUS = 139,
        INDEX_WAIT = 140,
        INDEX_RENAME = 156,
        FUNCALL = 64,
        BRANCH = 65,
        OR = 66,
        AND = 67,
        FOR_EACH = 68,
        FUNC = 69,
        ASC = 73,
        DESC = 74,
        INFO = 79,
        MATCH = 97,
        UPCASE = 141,
        DOWNCASE = 142,
        SAMPLE = 81,
        DEFAULT = 92,
        JSON = 98,
        TO_JSON_STRING = 172,
        ISO8601 = 99,
        TO_ISO8601 = 100,
        EPOCH_TIME = 101,
        TO_EPOCH_TIME = 102,
        NOW = 103,
        IN_TIMEZONE = 104,
        DURING = 105,
        DATE = 106,
        TIME_OF_DAY = 126,
        TIMEZONE = 127,
        YEAR = 128,
        MONTH = 129,
        DAY = 130,
        DAY_OF_WEEK = 131,
        DAY_OF_YEAR = 132,
        HOURS = 133,
        MINUTES = 134,
        SECONDS = 135,
        TIME = 136,
        MONDAY = 107,
        TUESDAY = 108,
        WEDNESDAY = 109,
        THURSDAY = 110,
        FRIDAY = 111,
        SATURDAY = 112,
        SUNDAY = 113,
        JANUARY = 114,
        FEBRUARY = 115,
        MARCH = 116,
        APRIL = 117,
        MAY = 118,
        JUNE = 119,
        JULY = 120,
        AUGUST = 121,
        SEPTEMBER = 122,
        OCTOBER = 123,
        NOVEMBER = 124,
        DECEMBER = 125,
        LITERAL = 137,
        GROUP = 144,
        SUM = 145,
        AVG = 146,
        MIN = 147,
        MAX = 148,
        SPLIT = 149,
        UNGROUP = 150,
        RANDOM = 151,
        CHANGES = 152,
        ARGS = 154,
        BINARY = 155,
        GEOJSON = 157,
        TO_GEOJSON = 158,
        POINT = 159,
        LINE = 160,
        POLYGON = 161,
        DISTANCE = 162,
        INTERSECTS = 163,
        INCLUDES = 164,
        CIRCLE = 165,
        GET_INTERSECTING = 166,
        FILL = 167,
        GET_NEAREST = 168,
        POLYGON_SUB = 171,
        MINVAL = 180,
        MAXVAL = 181,
    }

    impl Default for TermType {
        fn default() -> Self {
            TermType::DATUM
        }
    }

    impl From<i32> for TermType {
        fn from(i: i32) -> Self {
            match i {
                1 => TermType::DATUM,
                2 => TermType::MAKE_ARRAY,
                3 => TermType::MAKE_OBJ,
                10 => TermType::VAR,
                11 => TermType::JAVASCRIPT,
                169 => TermType::UUID,
                153 => TermType::HTTP,
                12 => TermType::ERROR,
                13 => TermType::IMPLICIT_VAR,
                14 => TermType::DB,
                15 => TermType::TABLE,
                16 => TermType::GET,
                78 => TermType::GET_ALL,
                17 => TermType::EQ,
                18 => TermType::NE,
                19 => TermType::LT,
                20 => TermType::LE,
                21 => TermType::GT,
                22 => TermType::GE,
                23 => TermType::NOT,
                24 => TermType::ADD,
                25 => TermType::SUB,
                26 => TermType::MUL,
                27 => TermType::DIV,
                28 => TermType::MOD,
                183 => TermType::FLOOR,
                184 => TermType::CEIL,
                185 => TermType::ROUND,
                29 => TermType::APPEND,
                80 => TermType::PREPEND,
                95 => TermType::DIFFERENCE,
                88 => TermType::SET_INSERT,
                89 => TermType::SET_INTERSECTION,
                90 => TermType::SET_UNION,
                91 => TermType::SET_DIFFERENCE,
                30 => TermType::SLICE,
                70 => TermType::SKIP,
                71 => TermType::LIMIT,
                87 => TermType::OFFSETS_OF,
                93 => TermType::CONTAINS,
                31 => TermType::GET_FIELD,
                94 => TermType::KEYS,
                186 => TermType::VALUES,
                143 => TermType::OBJECT,
                32 => TermType::HAS_FIELDS,
                96 => TermType::WITH_FIELDS,
                33 => TermType::PLUCK,
                34 => TermType::WITHOUT,
                35 => TermType::MERGE,
                36 => TermType::BETWEEN_DEPRECATED,
                182 => TermType::BETWEEN,
                37 => TermType::REDUCE,
                38 => TermType::MAP,
                187 => TermType::FOLD,
                39 => TermType::FILTER,
                40 => TermType::CONCAT_MAP,
                41 => TermType::ORDER_BY,
                42 => TermType::DISTINCT,
                43 => TermType::COUNT,
                86 => TermType::IS_EMPTY,
                44 => TermType::UNION,
                45 => TermType::NTH,
                170 => TermType::BRACKET,
                48 => TermType::INNER_JOIN,
                49 => TermType::OUTER_JOIN,
                50 => TermType::EQ_JOIN,
                72 => TermType::ZIP,
                173 => TermType::RANGE,
                82 => TermType::INSERT_AT,
                83 => TermType::DELETE_AT,
                84 => TermType::CHANGE_AT,
                85 => TermType::SPLICE_AT,
                51 => TermType::COERCE_TO,
                52 => TermType::TYPE_OF,
                53 => TermType::UPDATE,
                54 => TermType::DELETE,
                55 => TermType::REPLACE,
                56 => TermType::INSERT,
                57 => TermType::DB_CREATE,
                58 => TermType::DB_DROP,
                59 => TermType::DB_LIST,
                60 => TermType::TABLE_CREATE,
                61 => TermType::TABLE_DROP,
                62 => TermType::TABLE_LIST,
                174 => TermType::CONFIG,
                175 => TermType::STATUS,
                177 => TermType::WAIT,
                176 => TermType::RECONFIGURE,
                179 => TermType::REBALANCE,
                138 => TermType::SYNC,
                188 => TermType::GRANT,
                75 => TermType::INDEX_CREATE,
                76 => TermType::INDEX_DROP,
                77 => TermType::INDEX_LIST,
                139 => TermType::INDEX_STATUS,
                140 => TermType::INDEX_WAIT,
                156 => TermType::INDEX_RENAME,
                64 => TermType::FUNCALL,
                65 => TermType::BRANCH,
                66 => TermType::OR,
                67 => TermType::AND,
                68 => TermType::FOR_EACH,
                69 => TermType::FUNC,
                73 => TermType::ASC,
                74 => TermType::DESC,
                79 => TermType::INFO,
                97 => TermType::MATCH,
                141 => TermType::UPCASE,
                142 => TermType::DOWNCASE,
                81 => TermType::SAMPLE,
                92 => TermType::DEFAULT,
                98 => TermType::JSON,
                172 => TermType::TO_JSON_STRING,
                99 => TermType::ISO8601,
                100 => TermType::TO_ISO8601,
                101 => TermType::EPOCH_TIME,
                102 => TermType::TO_EPOCH_TIME,
                103 => TermType::NOW,
                104 => TermType::IN_TIMEZONE,
                105 => TermType::DURING,
                106 => TermType::DATE,
                126 => TermType::TIME_OF_DAY,
                127 => TermType::TIMEZONE,
                128 => TermType::YEAR,
                129 => TermType::MONTH,
                130 => TermType::DAY,
                131 => TermType::DAY_OF_WEEK,
                132 => TermType::DAY_OF_YEAR,
                133 => TermType::HOURS,
                134 => TermType::MINUTES,
                135 => TermType::SECONDS,
                136 => TermType::TIME,
                107 => TermType::MONDAY,
                108 => TermType::TUESDAY,
                109 => TermType::WEDNESDAY,
                110 => TermType::THURSDAY,
                111 => TermType::FRIDAY,
                112 => TermType::SATURDAY,
                113 => TermType::SUNDAY,
                114 => TermType::JANUARY,
                115 => TermType::FEBRUARY,
                116 => TermType::MARCH,
                117 => TermType::APRIL,
                118 => TermType::MAY,
                119 => TermType::JUNE,
                120 => TermType::JULY,
                121 => TermType::AUGUST,
                122 => TermType::SEPTEMBER,
                123 => TermType::OCTOBER,
                124 => TermType::NOVEMBER,
                125 => TermType::DECEMBER,
                137 => TermType::LITERAL,
                144 => TermType::GROUP,
                145 => TermType::SUM,
                146 => TermType::AVG,
                147 => TermType::MIN,
                148 => TermType::MAX,
                149 => TermType::SPLIT,
                150 => TermType::UNGROUP,
                151 => TermType::RANDOM,
                152 => TermType::CHANGES,
                154 => TermType::ARGS,
                155 => TermType::BINARY,
                157 => TermType::GEOJSON,
                158 => TermType::TO_GEOJSON,
                159 => TermType::POINT,
                160 => TermType::LINE,
                161 => TermType::POLYGON,
                162 => TermType::DISTANCE,
                163 => TermType::INTERSECTS,
                164 => TermType::INCLUDES,
                165 => TermType::CIRCLE,
                166 => TermType::GET_INTERSECTING,
                167 => TermType::FILL,
                168 => TermType::GET_NEAREST,
                171 => TermType::POLYGON_SUB,
                180 => TermType::MINVAL,
                181 => TermType::MAXVAL,
                _ => Self::default(),
            }
        }
    }

}
