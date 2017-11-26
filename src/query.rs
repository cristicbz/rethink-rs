pub use failure::Error;
use arrayvec::ArrayVec;
use serde::ser::{Serialize, Serializer};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use super::concatenator::Concatenator;
use super::enums::term;

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

pub fn db<NameT: Serialize>(name: NameT) -> Expr<DbOut, Term<(NameT,)>> {
    Expr::raw(term(term::DB, (name,)))
}

impl<OutT, AstT: Serialize> IntoExpr<OutT> for Expr<OutT, AstT> {
    type Ast = AstT;
    fn into_expr(self) -> Self {
        self
    }
}

impl<OutT, AstT> Expr<OutT, AstT> {
    pub fn table<NameT: IntoExpr<StringOut>>(
        self,
        name: NameT,
    ) -> Expr<TableOut, Term<(AstT, NameT::Ast)>>
    where
        OutT: IsDb,
    {
        Expr::raw(term(term::TABLE, (self.ast, name.into_expr().ast)))
    }

    pub fn get<KeyOutT: IsKey, KeyT: IntoExpr<KeyOutT>>(
        self,
        key: KeyT,
    ) -> Expr<SingleSelectionOut<ObjectOut>, Term<(AstT, KeyT::Ast)>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(term::GET, (self.ast, key.into_expr().ast)))
    }

    pub fn get_all<KeyT: IsKey, KeysT: IntoArgs<KeyT>>(
        self,
        key: KeysT,
    ) -> Expr<SelectionOut<ObjectOut>, Term<Concatenator<(AstT,), KeysT::ArgsAst>, GetAllOptions>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(
            term::GET_ALL,
            Concatenator((self.ast,), key.into_args()),
        ))
    }

    pub fn g<KeyT: IntoExpr<StringOut>>(self, key: KeyT) -> Expr<AnyOut, Term<(AstT, KeyT::Ast)>>
    where
        OutT: IsObjectOrObjectSequence,
    {
        self.get_field(key)
    }

    pub fn between<MinT: IntoExpr<KeysOutT>, MaxT: IntoExpr<KeysOutT>, KeysOutT: IsKey>(
        self,
        min: MinT,
        max: MaxT,
    ) -> Expr<SelectionOut<ObjectOut>, Term<(AstT, MinT::Ast, MaxT::Ast), BetweenOptions>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(
            term::BETWEEN,
            (self.ast, min.into_expr().ast, max.into_expr().ast),
        ))
    }

    pub fn in_index<NameT>(self, index: NameT) -> Expr<OutT, AstT::WithOption>
    where
        AstT: WithOption<IndexOption, NameT>,
    {
        Expr::raw(self.ast.with_option(index))
    }

    pub fn with_left_bound<BoundT>(self, bound: BoundT) -> Expr<OutT, AstT::WithOption>
    where
        AstT: WithOption<LeftBoundOption, BoundT>,
    {
        Expr::raw(self.ast.with_option(bound))
    }


    pub fn with_right_bound<BoundT>(self, bound: BoundT) -> Expr<OutT, AstT::WithOption>
    where
        AstT: WithOption<RightBoundOption, BoundT>,
    {
        Expr::raw(self.ast.with_option(bound))
    }

    pub fn get_field<KeyT: IntoExpr<StringOut>>(
        self,
        key: KeyT,
    ) -> Expr<AnyOut, Term<(AstT, KeyT::Ast)>>
    where
        OutT: IsObjectOrObjectSequence,
    {
        Expr::raw(term(term::GET_FIELD, (self.ast, key.into_expr().ast)))
    }

    pub fn filter<ReturnT, FilterT>(
        self,
        filter: FilterT,
    ) -> Expr<OutT, Term<(AstT, FilterT::FunctionAst)>>
    where
        OutT: IsSequence,
        FilterT: FnOnce(Var<OutT::SequenceItem>) -> ReturnT
            + IntoFunctionExpr<(OutT::SequenceItem,), BoolOut>,
    {
        Expr::raw(term(
            term::FILTER,
            (self.ast, filter.into_function_expr().ast),
        ))
    }

    pub fn eq<OtherOutT, OtherT>(self, other: OtherT) -> Expr<BoolOut, Term<(AstT, OtherT::Ast)>>
    where
        OutT: IsEqualComparable<OtherOutT>,
        OtherT: IntoExpr<OtherOutT>,
    {
        Expr::raw(term(term::EQ, (self.ast, other.into_expr().ast)))
    }

    pub fn add<OtherOutT, OtherT>(
        self,
        other: OtherT,
    ) -> Expr<OutT::Output, Term<(AstT, OtherT::Ast)>>
    where
        OutT: CanAdd<OtherOutT>,
        OtherT: IntoExpr<OtherOutT>,
    {
        Expr::raw(term(term::ADD, (self.ast, other.into_expr().ast)))
    }
}

#[derive(Serialize)]
pub struct Term<ArgsT, OptionsT: Options = NoOptions>(
    u32,
    ArgsT,
    #[serde(skip_serializing_if = "Options::all_unset")] OptionsT,
);


#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Null;
impl Serialize for Null {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

pub trait IntoExpr<OutT> {
    type Ast: Serialize;
    fn into_expr(self) -> Expr<OutT, Self::Ast>;
}

pub trait IntoArgs<ArgT>: IntoExpr<ArrayOut<ArgT>> {
    type ArgsAst: Serialize;
    fn into_args(self) -> Self::ArgsAst;
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

pub trait Datum: Serialize {
    type Out;
}

macro_rules! impl_datum {
    ($output:ty, $($rust:ty),+) => {
        $(
            impl IntoExpr<$output> for $rust {
                type Ast = Self;
                fn into_expr(self) -> Expr<$output, Self> {
                    Expr::raw(self)
                }
            }
            impl<'a> IntoExpr<$output> for &'a $rust {
                type Ast = Self;
                fn into_expr(self) -> Expr<$output, Self> {
                    Expr::raw(self)
                }
            }
         )+
    };
    ($output:ty, ref $rust:ty) => {
        impl<'a> IntoExpr<$output> for &'a $rust {
            type Ast = Self;
            fn into_expr(self) -> Expr<$output, Self> {
                Expr::raw(self)
            }
        }
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
            impl<OfOutT, OfT: IntoExpr<OfOutT>> IntoExpr<ArrayOut<OfOutT>> for [OfT; $len] {
                type Ast = Term<ArrayVec<[OfT::Ast; $len]>>;
                fn into_expr(self) -> Expr<ArrayOut<OfOutT>, Self::Ast> {
                    Expr::raw(term(term::MAKE_ARRAY, self.into_args()))
                }
            }

            impl<OfOutT, OfT: IntoExpr<OfOutT>> IntoArgs<OfOutT> for [OfT; $len] {
                type ArgsAst = ArrayVec<[OfT::Ast; $len]>;
                fn into_args(self) -> Self::ArgsAst {
                    ArrayVec::from(self).into_iter().map(|value| value.into_expr().ast).collect()
                }
            }
         )+
    }
}

impl_datum_fixed_array! {
    0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32
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


impl<OutT> IntoExpr<OutT> for MinVal {
    type Ast = Self;
    fn into_expr(self) -> Expr<OutT, Self> {
        Expr::raw(self)
    }
}
impl<OutT> IntoExpr<OutT> for MaxVal {
    type Ast = Self;
    fn into_expr(self) -> Expr<OutT, Self> {
        Expr::raw(self)
    }
}

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

pub trait IsEqualComparable<WithT> {}
impl IsEqualComparable<BoolOut> for BoolOut {}
impl IsEqualComparable<NumberOut> for NumberOut {}
impl IsEqualComparable<StringOut> for StringOut {}
impl IsEqualComparable<ObjectOut> for ObjectOut {}
impl<WithT> IsEqualComparable<WithT> for AnyOut {}
impl<WithT, OfT> IsEqualComparable<ArrayOut<WithT>> for ArrayOut<OfT>
where
    OfT: IsEqualComparable<WithT>,
{
}

impl IsEqualComparable<AnyOut> for BoolOut {}
impl IsEqualComparable<AnyOut> for NumberOut {}
impl IsEqualComparable<AnyOut> for StringOut {}
impl IsEqualComparable<AnyOut> for ObjectOut {}
impl<OfT> IsEqualComparable<AnyOut> for ArrayOut<OfT> {}


pub trait CanAdd<WithT> {
    type Output;
}
impl CanAdd<NumberOut> for NumberOut {
    type Output = NumberOut;
}
impl CanAdd<StringOut> for StringOut {
    type Output = StringOut;
}
impl<WithT> CanAdd<WithT> for AnyOut {
    type Output = AnyOut;
}
impl<WithT, OfT> CanAdd<ArrayOut<WithT>> for ArrayOut<OfT> {
    type Output = ArrayOut<AnyOut>;
}

impl CanAdd<AnyOut> for NumberOut {
    type Output = AnyOut;
}
impl CanAdd<AnyOut> for StringOut {
    type Output = AnyOut;
}
impl<OfT> CanAdd<AnyOut> for ArrayOut<OfT> {
    type Output = AnyOut;
}

pub type Var<OutT> = Expr<OutT, Term<(usize,)>>;

const NEXT_VAR_ID: AtomicUsize = ATOMIC_USIZE_INIT;
fn fresh_var<OutT>() -> Var<OutT> {
    Expr::raw(term(
        term::VAR,
        (NEXT_VAR_ID.fetch_add(1, Ordering::SeqCst),),
    ))
}

pub trait IntoFunctionExpr<ArgsT, ReturnT> {
    type FunctionAst;

    fn into_function_expr(self) -> Expr<FunctionOut<ArgsT, ReturnT>, Self::FunctionAst>;
}

impl<Arg1T, ReturnRawT, ReturnT, FunctionT> IntoFunctionExpr<(Arg1T,), ReturnT> for FunctionT
where
    FunctionT: FnOnce(Var<Arg1T>) -> ReturnRawT,
    ReturnRawT: IntoExpr<ReturnT>,
{
    type FunctionAst = Term<(Term<(usize,)>, ReturnRawT::Ast)>;

    fn into_function_expr(self) -> Expr<FunctionOut<(Arg1T,), ReturnT>, Self::FunctionAst> {
        let var = fresh_var();
        let var_id = (var.ast.1).0;
        Expr::raw(term(
            term::FUNC,
            (
                term(term::MAKE_ARRAY, (var_id,)),
                (self)(var).into_expr().ast,
            ),
        ))
    }
}

fn term<ArgsT, OptionsT: Default + Options>(term_type: u32, args: ArgsT) -> Term<ArgsT, OptionsT> {
    Term(term_type, args, OptionsT::default())
}

///// OPTIONS /////

pub trait Options: Serialize {
    fn all_unset(&self) -> bool;
}

pub trait OptionValue: Serialize {
    fn is_unset(&self) -> bool;
}

impl OptionValue for () {
    fn is_unset(&self) -> bool {
        true
    }
}

impl<OutT, AstT: Serialize> OptionValue for Expr<OutT, AstT> {
    fn is_unset(&self) -> bool {
        false
    }
}

pub trait WithOption<OptionT, ValueT> {
    type WithOption;
    fn with_option(self, value: ValueT) -> Self::WithOption;
}

impl<ArgsT, OptionsT, OptionT, ValueT> WithOption<OptionT, ValueT> for Term<ArgsT, OptionsT>
where
    OptionsT: Options + WithOption<OptionT, ValueT>,
    OptionsT::WithOption: Options,
{
    type WithOption = Term<ArgsT, OptionsT::WithOption>;

    fn with_option(self, value: ValueT) -> Self::WithOption {
        Term(self.0, self.1, self.2.with_option(value))
    }
}

pub enum IndexOption {}
pub enum LeftBoundOption {}
pub enum RightBoundOption {}

#[derive(Serialize, Default)]
pub struct NoOptions {}

impl Options for NoOptions {
    fn all_unset(&self) -> bool {
        true
    }
}

#[derive(Serialize, Default)]
pub struct GetAllOptions<IndexT: OptionValue = ()> {
    #[serde(skip_serializing_if = "OptionValue::is_unset")] index: IndexT,
}

impl<IndexT: OptionValue> Options for GetAllOptions<IndexT> {
    fn all_unset(&self) -> bool {
        self.index.is_unset()
    }
}

impl<NameT: IntoExpr<StringOut>> WithOption<IndexOption, NameT> for GetAllOptions<()> {
    type WithOption = GetAllOptions<Expr<StringOut, NameT::Ast>>;

    fn with_option(self, value: NameT) -> Self::WithOption {
        GetAllOptions {
            index: value.into_expr(),
        }
    }
}

#[derive(Serialize, Default)]
pub struct BetweenOptions<
    IndexT: OptionValue = (),
    LeftBoundT: OptionValue = (),
    RightBoundT: OptionValue = (),
> {
    #[serde(skip_serializing_if = "OptionValue::is_unset")] index: IndexT,
    #[serde(skip_serializing_if = "OptionValue::is_unset")] left_bound: LeftBoundT,
    #[serde(skip_serializing_if = "OptionValue::is_unset")] right_bound: RightBoundT,
}

impl<IndexT: OptionValue, LeftBoundT: OptionValue, RightBoundT: OptionValue> Options
    for BetweenOptions<IndexT, LeftBoundT, RightBoundT> {
    fn all_unset(&self) -> bool {
        self.index.is_unset() && self.left_bound.is_unset() && self.right_bound.is_unset()
    }
}

impl<
    NameT: IntoExpr<StringOut>,
    LeftBoundT: OptionValue,
    RightBoundT: OptionValue,
> WithOption<IndexOption, NameT> for BetweenOptions<(), LeftBoundT, RightBoundT> {
    type WithOption = BetweenOptions<Expr<StringOut, NameT::Ast>, LeftBoundT, RightBoundT>;

    fn with_option(self, value: NameT) -> Self::WithOption {
        BetweenOptions {
            index: value.into_expr(),
            left_bound: self.left_bound,
            right_bound: self.right_bound,
        }
    }
}

impl<
    IndexT: OptionValue,
    LeftBoundT: IntoExpr<StringOut>,
    RightBoundT: OptionValue,
> WithOption<LeftBoundOption, LeftBoundT> for BetweenOptions<IndexT, (), RightBoundT> {
    type WithOption = BetweenOptions<IndexT, Expr<StringOut, LeftBoundT::Ast>, RightBoundT>;

    fn with_option(self, value: LeftBoundT) -> Self::WithOption {
        BetweenOptions {
            index: self.index,
            left_bound: value.into_expr(),
            right_bound: self.right_bound,
        }
    }
}

impl<
    IndexT: OptionValue,
    LeftBoundT: OptionValue,
    RightBoundT: IntoExpr<StringOut>,
> WithOption<RightBoundOption, RightBoundT> for BetweenOptions<IndexT, LeftBoundT, ()> {
    type WithOption = BetweenOptions<IndexT, LeftBoundT, Expr<StringOut, RightBoundT::Ast>>;

    fn with_option(self, value: RightBoundT) -> Self::WithOption {
        BetweenOptions {
            index: self.index,
            left_bound: self.left_bound,
            right_bound: value.into_expr(),
        }
    }
}
