use std::old_io::{Writer, IoError};
use std::error::Error;
use std::num::Int;
use std::fmt;

use rustc_serialize::Encoder;

use super::SizeLimit;

pub type EncodingResult<T> = Result<T, EncodingError>;


/// An error that can be produced during encoding.
#[derive(Debug)]
pub enum EncodingError {
    /// An error originating from the underlying `Writer`.
    IoError(IoError),
    /// An object could not be encoded with the given size limit.
    ///
    /// This error is returned before any bytes are written to the
    /// output `Writer`.
    SizeLimit
}

/// An Encoder that encodes values directly into a Writer.
///
/// This struct should not be used often.
/// For most cases, prefer the `encode_into` function.
pub struct EncoderWriter<'a, W: 'a> {
    writer: &'a mut W,
    _size_limit: SizeLimit
}

pub struct SizeChecker {
    pub size_limit: u64,
    pub written: u64
}

fn wrap_io(err: IoError) -> EncodingError {
    EncodingError::IoError(err)
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            EncodingError::IoError(ref err) => write!(f, "IoError: {}", err),
            EncodingError::SizeLimit => write!(f, "SizeLimit")
        }
    }
}

impl Error for EncodingError {
    fn description(&self) -> &str {
        match *self {
            EncodingError::IoError(ref err)     => err.description(),
            EncodingError::SizeLimit => "the size limit for decoding has been reached"
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            EncodingError::IoError(ref err)     => err.cause(),
            EncodingError::SizeLimit => None
        }
    }
}

impl <'a, W: Writer> EncoderWriter<'a, W> {
    pub fn new(w: &'a mut W, size_limit: SizeLimit) -> EncoderWriter<'a, W> {
        EncoderWriter {
            writer: w,
            _size_limit: size_limit
        }
    }
}

impl SizeChecker {
    pub fn new(limit: u64) -> SizeChecker {
        SizeChecker {
            size_limit: limit,
            written: 0
        }
    }

    fn add_raw(&mut self, size: usize) -> EncodingResult<()> {
        self.written += size as u64;
        if self.written <= self.size_limit {
            Ok(())
        } else {
            Err(EncodingError::SizeLimit)
        }
    }

    fn add_value<T>(&mut self, t: T) -> EncodingResult<()> {
        use std::mem::size_of_val;
        self.add_raw(size_of_val(&t))
    }
}

impl<'a, W: Writer> Encoder for EncoderWriter<'a, W> {
    type Error = EncodingError;

    fn emit_nil(&mut self) -> EncodingResult<()> { Ok(()) }
    fn emit_usize(&mut self, v: usize) -> EncodingResult<()> {
        self.emit_u64(v as u64)
    }
    fn emit_u64(&mut self, v: u64) -> EncodingResult<()> {
        self.writer.write_be_u64(v).map_err(wrap_io)
    }
    fn emit_u32(&mut self, v: u32) -> EncodingResult<()> {
        self.writer.write_be_u32(v).map_err(wrap_io)
    }
    fn emit_u16(&mut self, v: u16) -> EncodingResult<()> {
        self.writer.write_be_u16(v).map_err(wrap_io)
    }
    fn emit_u8(&mut self, v: u8) -> EncodingResult<()> {
        self.writer.write_u8(v).map_err(wrap_io)
    }
    fn emit_isize(&mut self, v: isize) -> EncodingResult<()> {
        self.emit_i64(v as i64)
    }
    fn emit_i64(&mut self, v: i64) -> EncodingResult<()> {
        self.writer.write_be_i64(v).map_err(wrap_io)
    }
    fn emit_i32(&mut self, v: i32) -> EncodingResult<()> {
        self.writer.write_be_i32(v).map_err(wrap_io)
    }
    fn emit_i16(&mut self, v: i16) -> EncodingResult<()> {
        self.writer.write_be_i16(v).map_err(wrap_io)
    }
    fn emit_i8(&mut self, v: i8) -> EncodingResult<()> {
        self.writer.write_i8(v).map_err(wrap_io)
    }
    fn emit_bool(&mut self, v: bool) -> EncodingResult<()> {
        self.writer.write_u8(if v {1} else {0}).map_err(wrap_io)
    }
    fn emit_f64(&mut self, v: f64) -> EncodingResult<()> {
        self.writer.write_be_f64(v).map_err(wrap_io)
    }
    fn emit_f32(&mut self, v: f32) -> EncodingResult<()> {
        self.writer.write_be_f32(v).map_err(wrap_io)
    }
    fn emit_char(&mut self, v: char) -> EncodingResult<()> {
        self.writer.write_char(v).map_err(wrap_io)
    }
    fn emit_str(&mut self, v: &str) -> EncodingResult<()> {
        try!(self.emit_usize(v.len()));
        self.writer.write_str(v).map_err(wrap_io)
    }
    fn emit_enum<F>(&mut self, __: &str, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }
    fn emit_enum_variant<F>(&mut self, _: &str,
                            v_id: usize,
                            _: usize,
                            f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            let max_u32: u32 = Int::max_value();
            if v_id > (max_u32 as usize) {
                panic!("Variant tag doesn't fit in a u32")
            }
            try!(self.emit_u32(v_id as u32));
            f(self)
        }
    fn emit_enum_variant_arg<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }
    fn emit_enum_struct_variant<F>(&mut self, _: &str,
                                   _: usize,
                                   _: usize,
                                   f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }
    fn emit_enum_struct_variant_field<F>(&mut self,
                                         _: &str,
                                         _: usize,
                                         f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }
    fn emit_struct<F>(&mut self, _: &str, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }
    fn emit_struct_field<F>(&mut self, _: &str, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }
    fn emit_tuple<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }
    fn emit_tuple_arg<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }
    fn emit_tuple_struct<F>(&mut self, _: &str, len: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            self.emit_tuple(len, f)
        }
    fn emit_tuple_struct_arg<F>(&mut self, f_idx: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            self.emit_tuple_arg(f_idx, f)
        }
    fn emit_option<F>(&mut self, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }
    fn emit_option_none(&mut self) -> EncodingResult<()> {
        self.writer.write_u8(0).map_err(wrap_io)
    }
    fn emit_option_some<F>(&mut self, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            try!(self.writer.write_u8(1).map_err(wrap_io));
            f(self)
        }
    fn emit_seq<F>(&mut self, len: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            try!(self.emit_usize(len));
            f(self)
        }
    fn emit_seq_elt<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }
    fn emit_map<F>(&mut self, len: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            try!(self.emit_usize(len));
            f(self)
        }
    fn emit_map_elt_key<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }
    fn emit_map_elt_val<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut EncoderWriter<'a, W>) -> EncodingResult<()> {
            f(self)
        }

}

impl Encoder for SizeChecker {
    type Error = EncodingError;

    fn emit_nil(&mut self) -> EncodingResult<()> { Ok(()) }
    fn emit_usize(&mut self, v: usize) -> EncodingResult<()> {
        self.add_value(v as u64)
    }
    fn emit_u64(&mut self, v: u64) -> EncodingResult<()> {
        self.add_value(v)
    }
    fn emit_u32(&mut self, v: u32) -> EncodingResult<()> {
        self.add_value(v)
    }
    fn emit_u16(&mut self, v: u16) -> EncodingResult<()> {
        self.add_value(v)
    }
    fn emit_u8(&mut self, v: u8) -> EncodingResult<()> {
        self.add_value(v)
    }
    fn emit_isize(&mut self, v: isize) -> EncodingResult<()> {
        self.add_value(v as i64)
    }
    fn emit_i64(&mut self, v: i64) -> EncodingResult<()> {
        self.add_value(v)
    }
    fn emit_i32(&mut self, v: i32) -> EncodingResult<()> {
        self.add_value(v)
    }
    fn emit_i16(&mut self, v: i16) -> EncodingResult<()> {
        self.add_value(v)
    }
    fn emit_i8(&mut self, v: i8) -> EncodingResult<()> {
        self.add_value(v)
    }
    fn emit_bool(&mut self, _: bool) -> EncodingResult<()> {
        self.add_value(0 as u8)
    }
    fn emit_f64(&mut self, v: f64) -> EncodingResult<()> {
        self.add_value(v)
    }
    fn emit_f32(&mut self, v: f32) -> EncodingResult<()> {
        self.add_value(v)
    }
    fn emit_char(&mut self, v: char) -> EncodingResult<()> {
        self.add_raw(v.len_utf8())
    }
    fn emit_str(&mut self, v: &str) -> EncodingResult<()> {
        self.add_raw(v.len())
    }
    fn emit_enum<F>(&mut self, __: &str, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }
    fn emit_enum_variant<F>(&mut self, _: &str,
                            v_id: usize,
                            _: usize,
                            f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            try!(self.add_value(v_id as u32));
            f(self)
        }
    fn emit_enum_variant_arg<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }
    fn emit_enum_struct_variant<F>(&mut self, _: &str,
                                   _: usize,
                                   _: usize,
                                   f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }
    fn emit_enum_struct_variant_field<F>(&mut self,
                                         _: &str,
                                         _: usize,
                                         f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }
    fn emit_struct<F>(&mut self, _: &str, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }
    fn emit_struct_field<F>(&mut self, _: &str, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }
    fn emit_tuple<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }
    fn emit_tuple_arg<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }
    fn emit_tuple_struct<F>(&mut self, _: &str, len: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            self.emit_tuple(len, f)
        }
    fn emit_tuple_struct_arg<F>(&mut self, f_idx: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            self.emit_tuple_arg(f_idx, f)
        }
    fn emit_option<F>(&mut self, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }
    fn emit_option_none(&mut self) -> EncodingResult<()> {
        self.add_value(0 as u8)
    }
    fn emit_option_some<F>(&mut self, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            try!(self.add_value(1 as u8));
            f(self)
        }
    fn emit_seq<F>(&mut self, len: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            try!(self.emit_usize(len));
            f(self)
        }
    fn emit_seq_elt<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }
    fn emit_map<F>(&mut self, len: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            try!(self.emit_usize(len));
            f(self)
        }
    fn emit_map_elt_key<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }
    fn emit_map_elt_val<F>(&mut self, _: usize, f: F) -> EncodingResult<()> where
        F: FnOnce(&mut SizeChecker) -> EncodingResult<()> {
            f(self)
        }

}
