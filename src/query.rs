pub use failure::Error;
use arrayvec::ArrayVec;
use serde::ser::{Serialize, Serializer};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use super::enums::term;

pub struct Expr<OutT, AstT> {
    ast: AstT,
    _phantom: PhantomData<*const OutT>,
}

pub fn expr<OutT, OfT: IntoExpr<OutT>>(of: OfT) -> Expr<OutT, OfT::Ast> {
    of.into_expr()
}

pub fn args<OutT, OfT: IntoExpr<ArrayOut<OutT>>>(of: OfT) -> Args<OutT, Term<(OfT::Ast,)>> {
    Args {
        ast: term(term::ARGS, (of.into_ast(),)),
        _phantom: PhantomData,
    }
}

/// Reference a database.
pub fn db<NameT: Serialize>(name: NameT) -> Expr<DbOut, Term<(NameT,)>> {
    Expr::raw(term(term::DB, (name,)))
}

/// Create a database. A RethinkDB database is a collection of tables, similar to relational
/// databases.
pub fn db_create<NameT: Serialize>(name: NameT) -> Expr<ObjectOut, Term<(NameT,)>> {
    Expr::raw(term(term::DB_CREATE, (name,)))
}

/// List all database names in the cluster. The result is a list of strings.
pub fn db_list() -> Expr<ArrayOut<StringOut>, Term<[u8; 0]>> {
    Expr::raw(term(term::DB_LIST, []))
}

/// Drop a database. The database, all its tables, and corresponding data will be deleted.
pub fn db_drop<NameT: Serialize>(name: NameT) -> Expr<ObjectOut, Term<(NameT,)>> {
    Expr::raw(term(term::DB_DROP, (name,)))
}

impl<OutT, AstT> Expr<OutT, AstT> {
    /// Return all documents in a table. Other commands may be chained after table to return a
    /// subset of documents (such as get and filter) or perform further processing.
    pub fn table<NameT: IntoExpr<StringOut>>(
        self,
        name: NameT,
    ) -> Expr<TableOut, Term<(AstT, NameT::Ast)>>
    where
        OutT: IsDb,
    {
        Expr::raw(term(term::TABLE, (self.ast, name.into_ast())))
    }

    /// Create a table. A RethinkDB table is a collection of JSON documents.
    pub fn table_create<NameT: IntoExpr<StringOut>>(
        self,
        name: NameT,
    ) -> Expr<ObjectOut, Term<(AstT, NameT::Ast)>>
    where
        OutT: IsDb,
    {
        Expr::raw(term(term::TABLE_CREATE, (self.ast, name.into_ast())))
    }

    /// Drop a table. The table and all its data will be deleted.
    pub fn table_drop<NameT: IntoExpr<StringOut>>(
        self,
        name: NameT,
    ) -> Expr<ObjectOut, Term<(AstT, NameT::Ast)>>
    where
        OutT: IsDb,
    {
        Expr::raw(term(term::TABLE_DROP, (self.ast, name.into_ast())))
    }

    /// List all table names in a database. The result is a list of strings.
    pub fn table_list(self) -> Expr<ArrayOut<StringOut>, Term<(AstT, [u8; 0])>>
    where
        OutT: IsDb,
    {
        Expr::raw(term(term::TABLE_LIST, (self.ast, [])))
    }

    /// Create a new secondary index on a table. Secondary indexes improve the speed of many read
    /// queries at the slight cost of increased storage space and decreased write performance.
    ///
    /// FIXME: Index functions and options are not supported just yet, bear with me!
    pub fn index_create<NameT: IntoExpr<StringOut>>(
        self,
        name: NameT,
    ) -> Expr<ObjectOut, Term<(AstT, NameT::Ast)>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(term::INDEX_CREATE, (self.ast, name.into_ast())))
    }

    /// Delete a previously created secondary index of this table.
    pub fn index_drop<NameT: IntoExpr<StringOut>>(
        self,
        name: NameT,
    ) -> Expr<ObjectOut, Term<(AstT, NameT::Ast)>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(term::INDEX_DROP, (self.ast, name.into_ast())))
    }

    /// List all the secondary indexes of this table.
    pub fn index_list(self) -> Expr<ArrayOut<StringOut>, Term<(AstT, [u8; 0])>>
    where
        OutT: IsDb,
    {
        Expr::raw(term(term::INDEX_LIST, (self.ast, [])))
    }

    /// Rename an existing secondary index on a table. If the optional argument overwrite is
    /// specified as True, a previously existing index with the new name will be deleted and the
    /// index will be renamed. If overwrite is false (the default) an error will be raised if the
    /// new index name already exists.
    ///
    /// FIXME: overwrite option is not implemented.
    pub fn index_rename<SourceT: IntoExpr<StringOut>, DestinationT: IntoExpr<StringOut>>(
        self,
        source: SourceT,
        destination: DestinationT,
    ) -> Expr<ObjectOut, Term<(AstT, SourceT::Ast, DestinationT::Ast)>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(
            term::INDEX_DROP,
            (self.ast, source.into_ast(), destination.into_ast()),
        ))
    }

    /// Get the status of the specified indexes on this table, or the status of all indexes on this
    /// table if no indexes are specified.
    pub fn index_status<NameT: IsString, ArgsAstT, KeysT: Into<Args<NameT, ArgsAstT>>>(
        self,
        key: KeysT,
    ) -> Expr<SelectionOut<ObjectOut>, Term<(AstT, ArgsAstT), GetAllOptions>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(term::INDEX_STATUS, (self.ast, key.into().ast)))
    }

    /// Wait for the specified indexes on this table to be ready, or for all indexes on this table
    /// to be ready if no indexes are specified.
    pub fn index_wait<NameT: IsString, ArgsAstT, KeysT: Into<Args<NameT, ArgsAstT>>>(
        self,
        key: KeysT,
    ) -> Expr<SelectionOut<ObjectOut>, Term<(AstT, ArgsAstT)>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(term::INDEX_WAIT, (self.ast, key.into().ast)))
    }

    /// Insert documents into a table. Accepts a single document or an array of documents.
    /// FIXME: Missing insert options.
    pub fn insert<ObjectsOutT, ObjectsT>(
        self,
        objects: ObjectsT,
    ) -> Expr<ObjectOut, Term<(AstT, ObjectsT::Ast)>>
    where
        OutT: IsTable,
        ObjectsOutT: IsObjectOrObjectSequence,
        ObjectsT: IntoExpr<ObjectsOutT>,
    {
        Expr::raw(term(term::INSERT, (self.ast, objects.into_ast())))
    }

    /// Update JSON documents in a table. Accepts a JSON document, a ReQL expression, or a
    /// combination of the two.
    /// FIXME: Missing update options.
    pub fn update<ObjectT>(self, object: ObjectT) -> Expr<ObjectOut, Term<(AstT, ObjectT::Ast)>>
    where
        OutT: IsSelection<ObjectOut>,
        ObjectT: IntoExpr<ObjectOut>,
    {
        Expr::raw(term(term::UPDATE, (self.ast, object.into_ast())))
    }

    /// Update JSON documents in a table. Accepts a JSON document, a ReQL expression, or a
    /// combination of the two.
    /// FIXME: Missing update options.
    pub fn update_with<FunctionT, ReturnT>(
        self,
        with: FunctionT,
    ) -> Expr<ObjectOut, Term<(AstT, FunctionT::FunctionAst)>>
    where
        OutT: IsSelection<ObjectOut>,
        ReturnT: IntoExpr<ObjectOut>,
        FunctionT: FnOnce(Var<ObjectOut>) -> ReturnT + IntoFunctionExpr<(ObjectOut,), ObjectOut>,
    {
        Expr::raw(term(
            term::UPDATE,
            (self.ast, with.into_function_expr().ast),
        ))
    }

    // FIXME: Missing replace.

    /// Delete one or more documents from a table.
    /// FIXME: Missing delete options.
    pub fn delete(self) -> Expr<ObjectOut, Term<(AstT,)>>
    where
        OutT: IsSelection<ObjectOut>,
    {
        Expr::raw(term(term::DELETE, (self.ast,)))
    }

    /// Ensures that writes on a given table are written to permanent storage. Queries that specify
    /// soft durability (`durability='soft'`) do not give such guarantees, so sync can be used to
    /// ensure the state of these queries. A call to sync does not return until all previous writes
    /// to the table are persisted.
    pub fn sync(self) -> Expr<ObjectOut, Term<(AstT,)>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(term::SYNC, (self.ast,)))
    }


    /// Get a document by primary key.
    pub fn get<KeyOutT: IsString, KeyT: IntoExpr<KeyOutT>>(
        self,
        key: KeyT,
    ) -> Expr<SingleSelectionOut<ObjectOut>, Term<(AstT, KeyT::Ast)>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(term::GET, (self.ast, key.into_ast())))
    }

    /// Get all documents where the given value matches the value of the requested index.
    pub fn get_all<KeyT: IsIndexKey, ArgsAstT, KeysT: Into<Args<KeyT, ArgsAstT>>>(
        self,
        key: KeysT,
    ) -> Expr<SelectionOut<ObjectOut>, Term<(AstT, ArgsAstT), GetAllOptions>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(term::GET_ALL, (self.ast, key.into().ast)))
    }

    /// Get all documents between two keys. Accepts three options: `index`, `left_bound`, and
    /// `right_bound`. If `index` is set to the name of a secondary index, between will return all
    /// documents where that index's value is in the specified range (it uses the primary key by
    /// default). `left_bound` or `right_bound` may be set to `"open"` or `"closed"` to indicate
    /// whether or not to include that endpoint of the range (by default, `left_bound` is `"closed"`
    /// and `right_bound` is `"open"`).
    pub fn between<MinT: IntoExpr<KeysOutT>, MaxT: IntoExpr<KeysOutT>, KeysOutT: IsIndexKey>(
        self,
        min: MinT,
        max: MaxT,
    ) -> Expr<SelectionOut<ObjectOut>, Term<(AstT, MinT::Ast, MaxT::Ast), BetweenOptions>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(
            term::BETWEEN,
            (self.ast, min.into_ast(), max.into_ast()),
        ))
    }

    /// Sets the `index` option for operations that support it (e.g. `between`, `get_all` etc.),
    /// expects a string, the name of the secondary index.
    pub fn in_index<NameT>(self, index: NameT) -> Expr<OutT, AstT::WithOption>
    where
        AstT: WithOption<IndexOption, NameT>,
    {
        Expr::raw(self.ast.with_option(index))
    }

    /// Sets the `left_bound` option for `between`, expects a string: "closed" or "open".
    pub fn with_left_bound<BoundT>(self, bound: BoundT) -> Expr<OutT, AstT::WithOption>
    where
        AstT: WithOption<LeftBoundOption, BoundT>,
    {
        Expr::raw(self.ast.with_option(bound))
    }

    /// Sets the `right_bound` option for `between`, expects a string: "closed" or "open".
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
        Expr::raw(term(term::GET_FIELD, (self.ast, key.into_ast())))
    }

    pub fn g<KeyT: IntoExpr<StringOut>>(self, key: KeyT) -> Expr<AnyOut, Term<(AstT, KeyT::Ast)>>
    where
        OutT: IsObjectOrObjectSequence,
    {
        self.get_field(key)
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
        Expr::raw(term(term::EQ, (self.ast, other.into_ast())))
    }

    pub fn add<OtherOutT, OtherT>(
        self,
        other: OtherT,
    ) -> Expr<OutT::Output, Term<(AstT, OtherT::Ast)>>
    where
        OutT: CanAdd<OtherOutT>,
        OtherT: IntoExpr<OtherOutT>,
    {
        Expr::raw(term(term::ADD, (self.ast, other.into_ast())))
    }
}

pub struct Args<OfT, AstT> {
    ast: AstT,
    _phantom: PhantomData<*const OfT>,
}

impl<OfT, ExprT> From<ExprT> for Args<OfT, ExprT::Ast>
where
    ExprT: IntoExpr<ArrayOut<OfT>>,
{
    fn from(expr: ExprT) -> Self {
        Args {
            ast: expr.into_ast(),
            _phantom: PhantomData,
        }
    }
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

impl<OutT, AstT: Serialize> IntoExpr<OutT> for Expr<OutT, AstT> {}


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

pub trait IntoAst: Sized {
    type Ast: Serialize;
    fn into_ast(self) -> Self::Ast;
}

pub trait IntoExpr<OutT>: IntoAst {
    fn into_expr(self) -> Expr<OutT, Self::Ast> {
        Expr::raw(self.into_ast())
    }
}

impl<AstT: Serialize, OutT> IntoAst for Expr<OutT, AstT> {
    type Ast = AstT;
    fn into_ast(self) -> Self::Ast {
        self.ast
    }
}


impl<OutT, AstT: Serialize> IntoExpr<ArrayOut<OutT>> for Expr<AnyOut, AstT> {}

macro_rules! any_into_expr {
    ($($output:ty),+) => {
        $(
            impl<AstT: Serialize> IntoExpr<$output> for Expr<AnyOut, AstT> {}
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
            impl IntoAst for $rust {
                type Ast = Self;
                fn into_ast(self) -> Self {
                    self
                }
            }
            impl<'a> IntoAst for &'a $rust {
                type Ast = Self;
                fn into_ast(self) -> Self {
                    self
                }
            }

            impl IntoExpr<$output> for $rust {}
            impl<'a> IntoExpr<$output> for &'a $rust {}
         )+
    };
    ($output:ty, ref $rust:ty) => {
        impl<'a> IntoAst for &'a $rust {
            type Ast = Self;
            fn into_ast(self) -> Self {
                self
            }
        }
        impl<'a> IntoExpr<$output> for &'a $rust {}
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
            impl<OfT: IntoAst> IntoAst for [OfT; $len] {
                type Ast = Term<ArrayVec<[OfT::Ast; $len]>>;
                fn into_ast(self) -> Self::Ast {
                    term(term::MAKE_ARRAY,
                         ArrayVec::from(self).into_iter().map(|value| {
                             value.into_ast()
                         }).collect())
                }
            }

            impl<OfOutT, OfT: IntoExpr<OfOutT>> IntoExpr<ArrayOut<OfOutT>> for [OfT; $len] {}
         )+
    }
}

macro_rules! impl_datum_tuple {
    () => {};
    ($head:ident, $($tail:ident,)*) => {
        impl<$head, $($tail),*> IntoAst for ($head, $($tail),*)
            where $head: IntoAst, $($tail: IntoAst),*
        {
            type Ast = Term<($head::Ast, $($tail::Ast),*)>;
            #[allow(non_snake_case)]
            fn into_ast(self) -> Self::Ast {
                let ($head, $($tail),*) = self;
                term(term::MAKE_ARRAY,
                     ($head.into_ast(), $($tail.into_ast()),*))
            }
        }

        impl<$head, $($tail),*> IntoExpr<ArrayOut<AnyOut>> for ($head, $($tail),*)
            where $head: IntoAst, $($tail: IntoAst),*
        {}

        impl_datum_tuple!($($tail,)*);
    }
}

impl_datum_tuple! { A, B, C, D, E, F, G, H, I, J, }

impl_datum_fixed_array! {
    0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MinVal;

impl IntoAst for MinVal {
    type Ast = (u32,);

    fn into_ast(self) -> Self::Ast {
        (term::MINVAL,)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MaxVal;

impl IntoAst for MaxVal {
    type Ast = (u32,);

    fn into_ast(self) -> Self::Ast {
        (term::MAXVAL,)
    }
}

impl<OutT> IntoExpr<OutT> for MinVal {}
impl<OutT> IntoExpr<OutT> for MaxVal {}

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

pub trait IsIndexKey {}
impl<OfT> IsIndexKey for ArrayOut<OfT> {}
impl<OfT> IsIndexKey for SelectionOut<OfT> {}
impl<OfT> IsIndexKey for SingleSelectionOut<OfT> {}
impl<OfT> IsIndexKey for SequenceOut<OfT> {}
impl IsIndexKey for BoolOut {}
impl IsIndexKey for NumberOut {}
impl IsIndexKey for StringOut {}
impl IsIndexKey for NullOut {}
impl IsIndexKey for AnyOut {}

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


pub trait IsSelection<OfT> {}
impl<OfT> IsSelection<OfT> for SelectionOut<OfT> {}
impl<OfT> IsSelection<OfT> for SingleSelectionOut<OfT> {}
impl<ObjectT> IsSelection<ObjectT> for TableOut {}
impl<OfT> IsSelection<OfT> for AnyOut {}

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
            (term(term::MAKE_ARRAY, (var_id,)), (self)(var).into_ast()),
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
