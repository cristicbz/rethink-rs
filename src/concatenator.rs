use serde::ser::{Impossible, Serialize, SerializeSeq, SerializeTuple, SerializeTupleStruct,
                 Serializer};

pub struct Concatenator<A, B>(pub A, pub B);

impl<A: Serialize, B: Serialize> Serialize for Concatenator<A, B> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = ConcatenatorSerializerSeq(serializer.serialize_seq(None)?);
        self.0.serialize(ConcatenatorSerializer(&mut seq))?;
        self.1.serialize(ConcatenatorSerializer(&mut seq))?;
        seq.actually_end()
    }
}

struct ConcatenatorSerializer<'a, S: 'a>(&'a mut ConcatenatorSerializerSeq<S>);

impl<'a, S: 'a + SerializeSeq> Serializer for ConcatenatorSerializer<'a, S> {
    type Ok = ();
    type Error = S::Error;
    type SerializeSeq = &'a mut ConcatenatorSerializerSeq<S>;
    type SerializeTuple = &'a mut ConcatenatorSerializerSeq<S>;
    type SerializeTupleStruct = &'a mut ConcatenatorSerializerSeq<S>;
    type SerializeTupleVariant = Impossible<(), S::Error>;
    type SerializeMap = Impossible<(), S::Error>;
    type SerializeStruct = Impossible<(), S::Error>;
    type SerializeStructVariant = Impossible<(), S::Error>;

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self.0)
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self.0)
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self.0)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        unimplemented!();
    }
    fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_str(self, _: &str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_some<T: ?Sized>(self, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unimplemented!()
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unimplemented!()
    }

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unimplemented!()
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unimplemented!()
    }
}

pub struct ConcatenatorSerializerSeq<S>(S);

impl<S: SerializeSeq> ConcatenatorSerializerSeq<S> {
    fn actually_end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<'a, S: SerializeSeq> SerializeSeq for &'a mut ConcatenatorSerializerSeq<S> {
    type Ok = ();
    type Error = S::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, S: SerializeSeq> SerializeTuple for &'a mut ConcatenatorSerializerSeq<S> {
    type Ok = ();
    type Error = S::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, S: SerializeSeq> SerializeTupleStruct for &'a mut ConcatenatorSerializerSeq<S> {
    type Ok = ();
    type Error = S::Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
