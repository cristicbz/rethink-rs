pub use failure::Error;
use serde::ser::{Serialize, Serializer, SerializeSeq, Impossible, SerializeTuple,
SerializeTupleStruct};
use std::marker::PhantomData;
use super::enums::term;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

pub struct Expr<OutT, AstT> {
    ast: AstT,
    _phantom: PhantomData<*const OutT>,
}

pub fn expr<OutT, OfT: IntoExpr<OutT>>(of: OfT) -> Expr<OutT, OfT::Ast> {
    of.into_expr()
}
impl<OutT, AstT> Expr<OutT, AstT> {
    fn raw(ast: AstT) -> Self {
        Expr {
            ast,
            _phantom: PhantomData,
        }
    }
}

impl<OutT, AstT: Serialize> Serialize for Expr<OutT, AstT> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.ast.serialize(serializer)
    }
}

pub fn db<NameT: Serialize>(name: NameT) -> Expr<DbOut, Term<(NameT,), NoOptions>> {
    Expr::raw(Term(term::DB, (name,), NoOptions {}))
}

impl<OutT, AstT> Expr<OutT, AstT> {
    pub fn table<NameT: IntoExpr<StringOut>>(
        self,
        name: NameT,
        ) -> Expr<TableOut, Term<(AstT, NameT::Ast), NoOptions>>
        where
        OutT: IsDb,
        {
            Expr::raw(Term(
                    term::TABLE,
                    (self.ast, name.into_expr().ast),
                    NoOptions {},
                    ))
        }

    pub fn get<KeyOutT: IsKey, KeyT: IntoExpr<KeyOutT>>(
        self,
        key: KeyT,
        ) -> Expr<SingleSelectionOut<ObjectOut>, Term<(AstT, KeyT::Ast), NoOptions>>
        where
        OutT: IsTable,
        {
            Expr::raw(Term(term::GET, (self.ast, key.into_expr().ast), NoOptions {}))
        }

    pub fn get_all<KeysOutT: IsSequence, KeysT: IntoExpr<KeysOutT>>(
        self,
        key: KeysT,
        ) -> Expr<SelectionOut<ObjectOut>, Term<Concatenator<(AstT,), KeysT::Ast>, GetAllOptions>>
        where
        OutT: IsTable,
        KeysOutT::SequenceItem: IsKey,
        {
            Expr::raw(Term(
                    term::GET_ALL,
                    Concatenator((self.ast,), key.into_expr().ast),
                    GetAllOptions { index: (), }
                    ))
        }

    pub fn between<
        MinT: IntoExpr<KeysOutT>,
        MaxT: IntoExpr<KeysOutT>,
        KeysOutT: IsKey,
        >(
            self,
            min: MinT,
            max: MaxT,
            ) -> Expr<SelectionOut<ObjectOut>, Term<(AstT, MinT::Ast, MaxT::Ast), BetweenOptions>>
            where
            OutT: IsTable,
            {
                Expr::raw(Term(
                        term::BETWEEN,
                        (self.ast, min.into_expr().ast, max.into_expr().ast),
                        BetweenOptions { index: (), left_bound: (), right_bound: () }
                        ))
            }

    pub fn in_index<NameT>(self, index: NameT) -> Expr<OutT, AstT::WithOption>
        where
        AstT: WithOption<IndexOption, NameT>,
        {
            Expr::raw(self.ast.with_option(index))
        }

    pub fn left_bound<BoundT>(self, bound: BoundT) -> Expr<OutT, AstT::WithOption>
        where
        AstT: WithOption<LeftBoundOption, BoundT>,
        {
            Expr::raw(self.ast.with_option(bound))
        }


    pub fn right_bound<BoundT>(self, bound: BoundT) -> Expr<OutT, AstT::WithOption>
        where
        AstT: WithOption<RightBoundOption, BoundT>,
        {
            Expr::raw(self.ast.with_option(bound))
        }

    pub fn get_field<KeyT: IntoExpr<StringOut>>(
        self,
        key: KeyT,
        ) -> Expr<AnyOut, Term<(AstT, KeyT::Ast), NoOptions>>
        where
        OutT: IsObjectOrObjectSequence,
        {
            Expr::raw(Term(
                    term::GET_FIELD,
                    (self.ast, key.into_expr().ast),
                    NoOptions {},
                    ))
        }

    pub fn filter<FilterT: IntoExpr<FunctionOut<(OutT::SequenceItem,), BoolOut>>>(
        self, filter: FilterT) -> Expr<OutT::SequenceItem, Term<(AstT, FilterT::Ast), NoOptions>>
        where OutT: IsSequence {
            Expr::raw(Term(
                    term::GET_FIELD,
                    (self.ast, filter.into_expr().ast),
                    NoOptions {},
                    ))

        }

    pub fn g<KeyT: IntoExpr<StringOut>>(
        self,
        key: KeyT,
        ) -> Expr<AnyOut, Term<(AstT, KeyT::Ast), NoOptions>>
        where
        OutT: IsObjectOrObjectSequence,
        {
            self.get_field(key)
        }
}

#[derive(Serialize)]
pub struct Term<ArgsT, OptionsT: Options>(
    u32,
    ArgsT,
    #[serde(skip_serializing_if = "Options::all_unset")]
    OptionsT
    );


#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Null;
impl Serialize for Null {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

pub trait IntoExpr<OutT>: Serialize {
    type Ast: Serialize;
    fn into_expr(self) -> Expr<OutT, Self::Ast>;
}

impl<OutT, AstT: Serialize> IntoExpr<ArrayOut<OutT>> for Expr<AnyOut, AstT> {
    type Ast = AstT;
    fn into_expr(self) -> Expr<ArrayOut<OutT>, AstT> {
        Expr::raw(self.ast)
    }
}

macro_rules! any_into_expr {
    ($($output:ty),+) => {
        $(
            impl<AstT: Serialize> IntoExpr<$output> for Expr<AnyOut, AstT> {
                type Ast = AstT;
                fn into_expr(self) -> Expr<$output, AstT> {
                    Expr::raw(self.ast)
                }
            }
         )+
    }
}

any_into_expr!(StringOut, NumberOut, BoolOut, NullOut, ObjectOut);

pub trait Datum<Out>: Serialize {}

impl<'a, OutT, DatumT: Datum<OutT>> Datum<OutT> for &'a DatumT {}

impl<OutT, DatumT: Datum<OutT>> IntoExpr<OutT> for DatumT {
    type Ast = Self;
    fn into_expr(self) -> Expr<OutT, Self> {
        Expr::raw(self)
    }
}


macro_rules! impl_datum {
    ($output:ty, $($rust:ty),+) => {
        $(
            impl Datum<$output> for $rust {}
         )+
    };
    ($output:ty, ref $rust:ty) => {
        impl<'a> Datum<$output> for &'a $rust {}
    };
}

impl_datum!(StringOut, String);
impl_datum!(StringOut, ref str);

impl_datum!(NumberOut, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
impl_datum!(BoolOut, bool);
impl_datum!(NullOut, Null);

macro_rules! impl_datum_fixed_array {
    ($($len:expr),+) => {
        $(
            impl<OfOutT, OfT: IntoExpr<OfOutT>> Datum<ArrayOut<OfOutT>> for [OfT; $len] {}
         )+
    }
}

impl_datum_fixed_array! {
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    25, 26, 27, 28, 29, 30, 31, 32
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinVal;

impl Serialize for MinVal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (term::MINVAL,).serialize(serializer)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MaxVal;

impl Serialize for MaxVal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (term::MAXVAL,).serialize(serializer)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Var<OutT> {
    id: usize,
    _phantom: PhantomData<OutT>,
}

impl<OutT> Serialize for Var<OutT> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (term::VAR, self.id).serialize(serializer)
    }
}

impl<OutT> Var<OutT> {
    fn fresh() -> Self {
        Var {
            id: NEXT_VAR_ID.fetch_add(1, Ordering::SeqCst),
            _phantom: PhantomData,
        }
    }
}

impl<OutT> Datum<OutT> for Var<OutT> {}

const NEXT_VAR_ID: AtomicUsize = ATOMIC_USIZE_INIT;

impl<OfT> Datum<OfT> for MinVal {}
impl<OfT> Datum<OfT> for MaxVal {}

pub enum StringOut {}
pub struct ArrayOut<OfT>(PhantomData<*const OfT>);
pub struct SelectionOut<OfT>(PhantomData<*const OfT>);
pub struct SingleSelectionOut<OfT>(PhantomData<*const OfT>);
pub struct SequenceOut<OfT>(PhantomData<*const OfT>);
pub struct FunctionOut<ArgsT, ReturnT>(PhantomData<*const (ArgsT, ReturnT)>);
pub enum ObjectOut {}
pub enum BoolOut {}
pub enum NumberOut {}
pub enum NullOut {}
pub enum AnyOut {}

pub enum TableOut {}
pub enum DbOut {}

pub struct NullOr<OfT>(PhantomData<*const OfT>);

pub trait IsDb {}
impl IsDb for DbOut {}
impl IsDb for AnyOut {}

pub trait IsTable {}
impl IsTable for TableOut {}
impl IsTable for AnyOut {}

pub trait IsObject {}
impl IsObject for ObjectOut {}
impl IsObject for AnyOut {}
impl<OfT: IsObject> IsObject for SingleSelectionOut<OfT> {}

pub trait IsString {}
impl IsString for StringOut {}
impl IsString for AnyOut {}

pub trait IsKey {}
impl IsKey for StringOut {}
impl IsKey for NumberOut {}
impl IsKey for AnyOut {}

pub trait IsObjectOrObjectSequence {}
impl IsObjectOrObjectSequence for ObjectOut {}
impl<OfT: IsObject> IsObjectOrObjectSequence for SingleSelectionOut<OfT> {}
impl<OfT: IsObject> IsObjectOrObjectSequence for SelectionOut<OfT> {}
impl<OfT: IsObject> IsObjectOrObjectSequence for SequenceOut<OfT> {}
impl<OfT: IsObject> IsObjectOrObjectSequence for ArrayOut<OfT> {}
impl IsObjectOrObjectSequence for TableOut {}
impl IsObjectOrObjectSequence for AnyOut {}

pub trait IsSequence {
    type SequenceItem;
}

impl IsSequence for TableOut {
    type SequenceItem = ObjectOut;
}

impl<OfT> IsSequence for ArrayOut<OfT> {
    type SequenceItem = OfT;
}

impl<OfT> IsSequence for SequenceOut<OfT> {
    type SequenceItem = OfT;
}

impl IsSequence for AnyOut {
    type SequenceItem = AnyOut;
}

impl<Arg1T, ReturnT, ReturnOutT, FunctionT> IntoExpr<FunctionOut<(Arg1T,), ReturnOutT>> for FunctionT
where FunctionT: FnOnce(Var<Arg1T>) -> ReturnT,
      ReturnT: IntoExpr<ReturnOutT> {
          type Ast = ReturnT::Ast;
          fn into_expr(self) -> Expr<FunctionOut<(Arg1T,), ReturnOutT>, Self::Ast> {
              Expr::raw((self)(Var::fresh()).into_expr().ast)
          }
      }

///// OPTIONS /////

pub trait Options: Serialize {
    fn all_unset(&self) -> bool;
}

pub trait OptionValue: Serialize {
    fn is_unset(&self) -> bool;
}

impl OptionValue for () {
    fn is_unset(&self) -> bool { true }
}

impl<OutT, AstT: Serialize> OptionValue for Expr<OutT, AstT> {
    fn is_unset(&self) -> bool { false }
}

pub trait WithOption<OptionT, ValueT> {
    type WithOption;
    fn with_option(self, value: ValueT) -> Self::WithOption;
}

impl<ArgsT, OptionsT, OptionT, ValueT> WithOption<OptionT, ValueT> for Term<ArgsT, OptionsT>
where OptionsT: Options + WithOption<OptionT, ValueT>,
      OptionsT::WithOption: Options
{
    type WithOption = Term<ArgsT, OptionsT::WithOption>;

    fn with_option(self, value: ValueT) -> Self::WithOption {
        Term(self.0, self.1, self.2.with_option(value))
    }
}

pub enum IndexOption {}
pub enum LeftBoundOption {}
pub enum RightBoundOption {}

#[derive(Serialize)]
pub struct NoOptions {}

impl Options for NoOptions {
    fn all_unset(&self) -> bool {
        true
    }
}

#[derive(Serialize)]
pub struct GetAllOptions<IndexT: OptionValue=()> {
    #[serde(skip_serializing_if="OptionValue::is_unset")]
    index: IndexT,
}

impl<IndexT: OptionValue> Options for GetAllOptions<IndexT> {
    fn all_unset(&self) -> bool {
        self.index.is_unset()
    }
}

impl<NameT: IntoExpr<StringOut>> WithOption<IndexOption, NameT> for GetAllOptions<()> {
    type WithOption = GetAllOptions<Expr<StringOut, NameT::Ast>>;

    fn with_option(self, value: NameT) -> Self::WithOption {
        GetAllOptions { index: value.into_expr() }
    }
}

#[derive(Serialize)]
pub struct BetweenOptions<
IndexT: OptionValue=(), LeftBoundT: OptionValue=(), RightBoundT: OptionValue=()> {
    #[serde(skip_serializing_if="OptionValue::is_unset")]
    index: IndexT,
    #[serde(skip_serializing_if="OptionValue::is_unset")]
    left_bound: LeftBoundT,
    #[serde(skip_serializing_if="OptionValue::is_unset")]
    right_bound: RightBoundT,
}

impl<IndexT:OptionValue, LeftBoundT:OptionValue, RightBoundT:OptionValue> Options
for BetweenOptions<IndexT, LeftBoundT, RightBoundT> {
    fn all_unset(&self) -> bool {
        self.index.is_unset() && self.left_bound.is_unset() && self.right_bound.is_unset()
    }
}

impl<NameT: IntoExpr<StringOut>, LeftBoundT: OptionValue, RightBoundT:OptionValue>
WithOption<IndexOption, NameT> for BetweenOptions<(), LeftBoundT, RightBoundT> {
    type WithOption = BetweenOptions<Expr<StringOut, NameT::Ast>, LeftBoundT, RightBoundT>;

    fn with_option(self, value: NameT) -> Self::WithOption {
        BetweenOptions {
            index: value.into_expr(), left_bound: self.left_bound, right_bound: self.right_bound
        }
    }
}

impl<IndexT: OptionValue, LeftBoundT: IntoExpr<StringOut>, RightBoundT: OptionValue>
WithOption<LeftBoundOption, LeftBoundT> for BetweenOptions<IndexT, (), RightBoundT> {
    type WithOption = BetweenOptions<IndexT, Expr<StringOut, LeftBoundT::Ast>, RightBoundT>;

    fn with_option(self, value: LeftBoundT) -> Self::WithOption {
        BetweenOptions {
            index: self.index,
            left_bound: value.into_expr(),
            right_bound: self.right_bound
        }
    }
}

impl<IndexT: OptionValue, LeftBoundT: OptionValue, RightBoundT: IntoExpr<StringOut>>
WithOption<RightBoundOption, RightBoundT> for BetweenOptions<IndexT, LeftBoundT, ()> {
    type WithOption = BetweenOptions<IndexT, LeftBoundT, Expr<StringOut, RightBoundT::Ast>>;

    fn with_option(self, value: RightBoundT) -> Self::WithOption {
        BetweenOptions {
            index: self.index,
            left_bound: self.left_bound,
            right_bound: value.into_expr(),
        }
    }
}

pub struct Concatenator<A, B>(A, B);

impl<A: Serialize, B: Serialize> Serialize for Concatenator<A, B> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = ConcatenatorSerializerSeq(serializer.serialize_seq(None)?);
        self.0.serialize(ConcatenatorSerializer(&mut seq))?;
        self.1.serialize(ConcatenatorSerializer(&mut seq))?;
        seq.actually_end()
    }
}

pub struct ConcatenatorSerializer<'a, S: 'a>(&'a mut ConcatenatorSerializerSeq<S>);

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
