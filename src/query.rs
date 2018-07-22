pub use failure::Error;
use arrayvec::ArrayVec;
use serde::ser::{Serialize, Serializer};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use super::enums::term;


#[derive(Copy, Clone, Debug)]
pub struct Expr<OutT, AstT> {
    ast: AstT,
    _phantom: PhantomData<*const OutT>,
}

/// Construct a ReQL object.
pub fn expr<OfT: IntoExpr>(of: OfT) -> Expr<OfT::Out, OfT::Ast> {
    of.into_expr()
}

/// `args` is a special term that’s used to splice an array of arguments into another term. This is
/// useful when you want to call a variadic term such as getAll with a set of arguments produced at
/// runtime.
pub fn args<OutT, OfT: IntoExpr<Out=ArrayOut<OutT>>>(of: OfT) -> Args<OutT, Term<(OfT::Ast,)>> {
    Args {
        ast: term(term::ARGS, (of.into_ast(),)),
        _phantom: PhantomData,
    }
}

// FIXME: Implement binary.

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
    pub fn table<NameT: IntoExpr<Out=StringOut>>(
        self,
        name: NameT,
    ) -> Expr<TableOut, Term<(AstT, NameT::Ast)>>
    where
        OutT: IsDb,
    {
        Expr::raw(term(term::TABLE, (self.ast, name.into_ast())))
    }

    /// Create a table. A RethinkDB table is a collection of JSON documents.
    pub fn table_create<NameT: IntoExpr<Out=StringOut>>(
        self,
        name: NameT,
    ) -> Expr<ObjectOut, Term<(AstT, NameT::Ast)>>
    where
        OutT: IsDb,
    {
        Expr::raw(term(term::TABLE_CREATE, (self.ast, name.into_ast())))
    }

    /// Drop a table. The table and all its data will be deleted.
    pub fn table_drop<NameT: IntoExpr<Out=StringOut>>(
        self,
        name: NameT,
    ) -> Expr<ObjectOut, Term<(AstT, NameT::Ast)>>
    where
        OutT: IsDb,
    {
        Expr::raw(term(term::TABLE_DROP, (self.ast, name.into_ast())))
    }

    /// List all table names in a database. The result is a list of strings.
    pub fn table_list(self) -> Expr<ArrayOut<StringOut>, Term<(AstT,)>>
    where
        OutT: IsDb,
    {
        Expr::raw(term(term::TABLE_LIST, (self.ast,)))
    }

    /// Create a new secondary index on a table. Secondary indexes improve the speed of many read
    /// queries at the slight cost of increased storage space and decreased write performance.
    ///
    /// FIXME: Index functions and options are not supported just yet, bear with me!
    pub fn index_create<NameT: IntoExpr<Out=StringOut>>(
        self,
        name: NameT,
    ) -> Expr<ObjectOut, Term<(AstT, NameT::Ast)>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(term::INDEX_CREATE, (self.ast, name.into_ast())))
    }

    /// Delete a previously created secondary index of this table.
    pub fn index_drop<NameT: IntoExpr<Out=StringOut>>(
        self,
        name: NameT,
    ) -> Expr<ObjectOut, Term<(AstT, NameT::Ast)>>
    where
        OutT: IsTable,
    {
        Expr::raw(term(term::INDEX_DROP, (self.ast, name.into_ast())))
    }

    /// List all the secondary indexes of this table.
    pub fn index_list(self) -> Expr<ArrayOut<StringOut>, Term<(AstT,)>>
    where
        OutT: IsDb,
    {
        Expr::raw(term(term::INDEX_LIST, (self.ast,)))
    }

    /// Rename an existing secondary index on a table. If the optional argument overwrite is
    /// specified as True, a previously existing index with the new name will be deleted and the
    /// index will be renamed. If overwrite is false (the default) an error will be raised if the
    /// new index name already exists.
    ///
    /// FIXME: overwrite option is not implemented.
    pub fn index_rename<SourceT: IntoExpr<Out=StringOut>, DestinationT: IntoExpr<Out=StringOut>>(
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
    pub fn insert<ObjectsT>(
        self,
        objects: ObjectsT,
    ) -> Expr<ObjectOut, Term<(AstT, ObjectsT::Ast)>>
    where
        ObjectsT: IntoExpr,
        ObjectsT::Out: IsObjectOrObjectSequence,
    {
        Expr::raw(term(term::INSERT, (self.ast, objects.into_ast())))
    }

    /// Update JSON documents in a table. Accepts a JSON document, a ReQL expression, or a
    /// combination of the two.
    /// FIXME: Missing update options.
    pub fn update<ObjectT>(self, object: ObjectT) -> Expr<ObjectOut, Term<(AstT, ObjectT::Ast)>>
    where
        OutT: IsSelection<ObjectOut>,
        ObjectT: IntoExpr<Out=ObjectOut>,
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
        ReturnT: IntoExpr<Out=ObjectOut>,
        FunctionT: FnOnce(Var<ObjectOut>) -> ReturnT + IntoFunctionExpr<(ObjectOut,), ObjectOut>,
    {
        Expr::raw(term(
            term::UPDATE,
            (self.ast, with.into_function_expr().ast),
        ))
    }

    // FIXME: Implement replace.

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
    pub fn get<KeyT: IntoExpr>(
        self,
        key: KeyT,
    ) -> Expr<SingleSelectionOut<ObjectOut>, Term<(AstT, KeyT::Ast)>>
    where
        OutT: IsTable,
        KeyT::Out: IsKey,
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
    pub fn between<MinT: MinLimitFor<KeysOutT>, MaxT: MaxLimitFor<KeysOutT>, KeysOutT: IsIndexKey>(
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

    /// Return all the elements in a sequence for which the given predicate is true. The return
    /// value of filter will be the same as the input (sequence, stream, or array). Documents can be
    /// filtered in a variety of ways—ranges, nested values, boolean conditions, and the results of
    /// anonymous functions.
    ///
    /// FIXME: Missing `default` option.
    /// FIXME: Missing support for filtering with an object.
    pub fn filter<ReturnT, FilterT>(
        self,
        filter: FilterT,
    ) -> Expr<OutT, Term<(AstT, FilterT::FunctionAst)>>
    where
        OutT: IsSequence,
        ReturnT: IntoExpr,
        ReturnT::Out: IsBool,
        FilterT: FnOnce(Var<OutT::SequenceItem>) -> ReturnT
            + IntoFunctionExpr<(OutT::SequenceItem,), BoolOut>,
    {
        Expr::raw(term(
            term::FILTER,
            (self.ast, filter.into_function_expr().ast),
        ))
    }

    // FIXME: Implement innerJoin
    // FIXME: Implement outerJoin
    // FIXME: Implement eqJoin

    /// Used to 'zip' up the result of a join by merging the 'right' fields into 'left' fields of
    /// each member of the sequence.
    pub fn zip(self) -> Expr<ObjectOut, Term<(AstT,)>>
    where
        OutT: IsSequence,
        OutT::SequenceItem: IsObject,
    {
        Expr::raw(term(term::ZIP, (self.ast,)))
    }

    /// Transform each element of one or more sequences by applying a mapping function to them. If
    /// map is run with two or more sequences, it will iterate for as many items as there are in the
    /// shortest sequence.
    ///
    /// FIXME: Implement support for multiple sequences.
    pub fn map<ReturnT, MapT>(
        self,
        map: MapT,
    ) -> Expr<OutT::Rebound, Term<(AstT, MapT::FunctionAst)>>
    where
        OutT: Rebind<ReturnT::Out>,
        ReturnT: IntoExpr,
        MapT: FnOnce(Var<OutT::SequenceItem>) -> ReturnT
            + IntoFunctionExpr<(OutT::SequenceItem,), ReturnT::Out>,
    {
        Expr::raw(term(term::MAP, (self.ast, map.into_function_expr().ast)))
    }

    /// Plucks one or more attributes from a sequence of objects, filtering out any objects in the
    /// sequence that do not have the specified fields. Functionally, this is identical to
    /// `has_fields` followed by `pluck` on a sequence.
    pub fn with_fields<SelectorT, ArgsAstT, SelectorsT>(
        self,
        selectors: SelectorsT,
    ) -> Expr<OutT::Rebound, Term<(AstT, ArgsAstT)>>
    where
        OutT: Rebind<ObjectOut>,
        SelectorT: IsSelector,
        SelectorsT: Into<Args<SelectorT, ArgsAstT>>,
    {
        Expr::raw(term(term::WITH_FIELDS, (self.ast, selectors.into().ast)))
    }

    /// Concatenate one or more elements into a single sequence using a mapping function.
    pub fn concat_map<ReturnT, MapT>(
        self,
        concat_map: MapT,
    ) -> Expr<OutT::Rebound, Term<(AstT, MapT::FunctionAst)>>
    where
        OutT: Rebind<<ReturnT::Out as IsSequence>::SequenceItem>,
        ReturnT: IntoExpr,
        ReturnT::Out: IsSequence,
        MapT: FnOnce(Var<OutT::SequenceItem>) -> ReturnT
            + IntoFunctionExpr<(OutT::SequenceItem,), ReturnT::Out>,
    {
        Expr::raw(term(
            term::CONCAT_MAP,
            (self.ast, concat_map.into_function_expr().ast),
        ))
    }

    // FIXME: implement order_by

    /// Skip a number of elements from the head of the sequence.
    pub fn skip<NumT>(self, n: NumT) -> Expr<OutT, Term<(AstT, NumT::Ast)>>
    where
        NumT: IntoExpr<Out=NumberOut>,
        OutT: IsSequence,
    {
        Expr::raw(term(term::SKIP, (self.ast, n.into_ast())))
    }

    /// End the sequence after the given number of elements.
    pub fn limit<NumT>(self, n: NumT) -> Expr<OutT, Term<(AstT, NumT::Ast)>>
    where
        NumT: IntoExpr<Out=NumberOut>,
        OutT: IsSequence,
    {
        Expr::raw(term(term::LIMIT, (self.ast, n.into_ast())))
    }

    /// Return the elements of a sequence within the specified range.
    pub fn slice_after<NumT>(self, start: NumT) -> Expr<OutT, Term<(AstT, NumT::Ast)>>
    where
        NumT: IntoExpr<Out=NumberOut>,
        OutT: IsSequence,
    {
        Expr::raw(term(term::SLICE, (self.ast, start.into_ast())))
    }

    /// Return the elements of a sequence within the specified range.
    /// FIXME: Add left_bound and right_bound options.
    pub fn slice<StartT, EndT>(
        self,
        start: StartT,
        end: EndT,
    ) -> Expr<OutT, Term<(AstT, StartT::Ast, EndT::Ast)>>
    where
        StartT: IntoExpr<Out=NumberOut>,
        EndT: IntoExpr<Out=NumberOut>,
        OutT: IsSequence,
    {
        Expr::raw(term(
            term::SLICE,
            (self.ast, start.into_ast(), end.into_ast()),
        ))
    }

    /// Get the nth element of a sequence, counting from zero. If the argument is negative, count
    /// from the last element.
    pub fn nth<NumT>(self, n: NumT) -> Expr<OutT::Select, Term<(AstT, NumT::Ast)>>
    where
        NumT: IntoExpr<Out=NumberOut>,
        OutT: IsSequence,
    {
        Expr::raw(term(term::NTH, (self.ast, n.into_ast())))
    }

    // FIXME: Implement offsets_of.

    // FIXME: Implement is_empty.
    pub fn is_empty(self) -> Expr<BoolOut, Term<(AstT,)>>
    where
        OutT: IsSequence,
    {
        Expr::raw(term(term::IS_EMPTY, (self.ast,)))
    }

    // FIXME: Implement union.
    // FIXME: Implement sample.

    // FIXME: Implement group.
    // FIXME: Implement ungroup.
    // FIXME: Implement reduce.
    // FIXME: Implement fold.

    // FIXME: Implement sum.
    pub fn sum(self) -> Expr<NumberOut, Term<(AstT,)>>
    where
        OutT: IsSequence,
        OutT::SequenceItem: IsNumber,
    {
        Expr::raw(term(term::SUM, (self.ast,)))
    }

    // FIXME: Implement avg.
    pub fn avg(self) -> Expr<NumberOut, Term<(AstT,)>>
    where
        OutT: IsSequence,
        OutT::SequenceItem: IsNumber,
    {
        Expr::raw(term(term::AVG, (self.ast,)))
    }

    // FIXME: Implement min.
    pub fn min(self) -> Expr<OutT::SequenceItem, Term<(AstT,)>>
    where
        OutT: IsSequence,
    {
        Expr::raw(term(term::MIN, (self.ast,)))
    }

    // FIXME: Implement max.
    pub fn max(self) -> Expr<OutT::SequenceItem, Term<(AstT,)>>
    where
        OutT: IsSequence,
    {
        Expr::raw(term(term::MAX, (self.ast,)))
    }

    // FIXME: Implement distinct.
    pub fn distinct(self) -> Expr<OutT, Term<(AstT,)>>
    where
        OutT: IsSequence,
    {
        Expr::raw(term(term::DISTINCT, (self.ast,)))
    }

    // FIXME: Implement contains.
    pub fn contains<ValueT>(self, value: ValueT) -> Expr<BoolOut, Term<(AstT, ValueT::Ast)>>
    where
        OutT: IsSequence,
        OutT::SequenceItem: IsEqualComparable<ValueT::Out>,
        ValueT: IntoExpr,
    {
        Expr::raw(term(term::CONTAINS, (self.ast, value.into_ast())))
    }

    // FIXME: Implement r.row implicit var functions.

    // FIXME: Implement pluck.
    // FIXME: Implement without.
    // FIXME: Implement merge.
    // FIXME: Implement append.
    // FIXME: Implement prepend.
    // FIXME: Implement difference.
    // FIXME: Implement setInsert.
    // FIXME: Implement setUnion.
    // FIXME: Implement setIntersection.
    // FIXME: Implement setDifference.

    /// Get a single field from an object. If called on a sequence, gets that field from every
    /// object in the sequence, skipping objects that lack it.
    pub fn get_field<KeyT: IntoExpr<Out=StringOut>>(
        self,
        key: KeyT,
    ) -> Expr<OutT::AnyOrAnySequence, Term<(AstT, KeyT::Ast)>>
    where
        OutT: IsObjectOrObjectSequence,
    {
        Expr::raw(term(term::GET_FIELD, (self.ast, key.into_ast())))
    }

    /// Alias for `get_field`.
    pub fn g<KeyT: IntoExpr<Out=StringOut>>(
        self,
        key: KeyT,
    ) -> Expr<OutT::AnyOrAnySequence, Term<(AstT, KeyT::Ast)>>
    where
        OutT: IsObjectOrObjectSequence,
    {
        Expr::raw(term(term::GET_FIELD, (self.ast, key.into_ast())))
    }

    /// Alias for `get_field` that asserts the result is a object.
    pub fn get_object<KeyT: IntoExpr<Out=StringOut>>(self, key: KeyT) -> Expr<ObjectOut, Term<(AstT, KeyT::Ast)>>
    where
        OutT: IsObjectOrObjectSequence,
    {
        self.get_field(key).as_object()
    }

    // FIXME: Implement hasFields
    // FIXME: Implement insertAt
    // FIXME: Implement spliceAt
    // FIXME: Implement deleteAt
    // FIXME: Implement changeAt
    // FIXME: Implement keys
    // FIXME: Implement values
    // FIXME: Implement literal
    // FIXME: Implement object

    // FIXME: Implement match
    // FIXME: Implement split
    // FIXME: Implement upcase
    // FIXME: Implement downcase

    /// Sum two or more numbers, or concatenate two or more strings or arrays.
    /// FIXME: Support more args.
    pub fn add<OtherT>(
        self,
        other: OtherT,
    ) -> Expr<OutT::Output, Term<(AstT, OtherT::Ast)>>
    where
        OutT: CanAdd<OtherT::Out>,
        OtherT: IntoExpr,
    {
        Expr::raw(term(term::ADD, (self.ast, other.into_ast())))
    }

    // FIXME: Implement sub
    pub fn sub<OtherT>(
        self,
        other: OtherT,
    ) -> Expr<NumberOut, Term<(AstT, OtherT::Ast)>>
    where
        OutT: IsNumber,
        OtherT::Out: IsNumber,
        OtherT: IntoExpr,
    {
        Expr::raw(term(term::SUB, (self.ast, other.into_ast())))
    }

    // FIXME: Implement mul
    pub fn mul<OtherT>(
        self,
        other: OtherT,
    ) -> Expr<NumberOut, Term<(AstT, OtherT::Ast)>>
    where
        OutT: IsNumber,
        OtherT::Out: IsNumber,
        OtherT: IntoExpr,
    {
        Expr::raw(term(term::MUL, (self.ast, other.into_ast())))
    }

    // FIXME: Implement div
    pub fn div<OtherT>(
        self,
        other: OtherT,
    ) -> Expr<NumberOut, Term<(AstT, OtherT::Ast)>>
    where
        OutT: IsNumber,
        OtherT::Out: IsNumber,
        OtherT: IntoExpr,
    {
        Expr::raw(term(term::DIV, (self.ast, other.into_ast())))
    }

    // FIXME: Implement mod
    pub fn modulo<OtherT>(
        self,
        other: OtherT,
    ) -> Expr<NumberOut, Term<(AstT, OtherT::Ast)>>
    where
        OutT: IsNumber,
        OtherT::Out: IsNumber,
        OtherT: IntoExpr,
    {
        Expr::raw(term(term::MOD, (self.ast, other.into_ast())))
    }

    // FIXME: Implement and
    pub fn and<OtherT>(
        self,
        other: OtherT,
    ) -> Expr<BoolOut, Term<(AstT, OtherT::Ast)>>
    where
        OutT: IsBool,
        OtherT::Out: IsBool,
        OtherT: IntoExpr,
    {
        Expr::raw(term(term::AND, (self.ast, other.into_ast())))
    }

    // FIXME: Implement or
    pub fn or<OtherT>(
        self,
        other: OtherT,
    ) -> Expr<BoolOut, Term<(AstT, OtherT::Ast)>>
    where
        OutT: IsBool,
        OtherT::Out: IsBool,
        OtherT: IntoExpr,
    {
        Expr::raw(term(term::OR, (self.ast, other.into_ast())))
    }

    /// Test if two or more values are equal.
    /// FIXME: Support more args.
    pub fn eq<OtherT>(self, other: OtherT) -> Expr<BoolOut, Term<(AstT, OtherT::Ast)>>
    where
        OutT: IsEqualComparable<OtherT::Out>,
        OtherT: IntoExpr,
    {
        Expr::raw(term(term::EQ, (self.ast, other.into_ast())))
    }

    /// Test if two or more values are not equal.
    /// FIXME: Support more args.
    pub fn ne<OtherT>(self, other: OtherT) -> Expr<BoolOut, Term<(AstT, OtherT::Ast)>>
    where
        OutT: IsEqualComparable<OtherT::Out>,
        OtherT: IntoExpr,
    {
        Expr::raw(term(term::NE, (self.ast, other.into_ast())))
    }

    // FIXME: Implement gt
    // FIXME: Implement ge
    // FIXME: Implement lt
    // FIXME: Implement le
    // FIXME: Implement not
    // FIXME: Implement random
    // FIXME: Implement round
    // FIXME: Implement ceil
    // FIXME: Implement floor

    // FIXME: Implement now
    // FIXME: Implement time
    // FIXME: Implement epochTime
    // FIXME: Implement iso8601
    // FIXME: Implement inTimezone
    // FIXME: Implement timezone
    // FIXME: Implement during
    // FIXME: Implement date
    // FIXME: Implement timeofday
    // FIXME: Implement year
    // FIXME: Implement month
    // FIXME: Implement day
    // FIXME: Implement dayOfWeek
    // FIXME: Implement dayOfYear
    // FIXME: Implement hours
    // FIXME: Implement minutes
    // FIXME: Implement seconds
    // FIXME: Implement to_iso8601
    // FIXME: Implement toEpochTime

    // FIXME: Implement do
    // FIXME: Implement branch
    // FIXME: Implement forEach
    // FIXME: Implement range
    // FIXME: Implement error
    // FIXME: Implement default
    // FIXME: Implement js
    // FIXME: Implement coerce_to (potentially as multiple functions)
    // FIXME: Implement type_of
    // FIXME: Implement json
    // FIXME: Implement to_json

    // FIXME: Implement http
    // FIXME: Implement uuid

    // FIXME: Implement grant
    // FIXME: Implement config
    // FIXME: Implement rebalance
    // FIXME: Implement reconfigure
    // FIXME: Implement status
    // FIXME: Implement wait

    pub fn as_number(self) -> Expr<NumberOut, AstT> {
        Expr::raw(self.ast)
    }

    pub fn as_string(self) -> Expr<StringOut, AstT> {
        Expr::raw(self.ast)
    }

    pub fn as_bool(self) -> Expr<BoolOut, AstT> {
        Expr::raw(self.ast)
    }

    pub fn as_object(self) -> Expr<ObjectOut, AstT> {
        Expr::raw(self.ast)
    }

    pub fn items_as<OtherT>(self) -> Expr<OutT::Rebound, AstT> where OutT: Rebind<OtherT> {
        Expr::raw(self.ast)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Args<OfT, AstT> {
    ast: AstT,
    _phantom: PhantomData<*const OfT>,
}

impl<ExprT> From<ExprT> for Args<ExprT::Out, ExprT::Ast>
where
    ExprT: IntoExpr,
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

impl<OutT, AstT: Serialize> IntoExpr for Expr<OutT, AstT> {
    type Out = OutT;
}


#[derive(Copy, Clone, Debug, Serialize)]
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

pub trait IntoExpr: IntoAst {
    type Out;

    fn into_expr(self) -> Expr<Self::Out, Self::Ast> {
        Expr::raw(self.into_ast())
    }
}

impl<AstT: Serialize, OutT> IntoAst for Expr<OutT, AstT> {
    type Ast = AstT;
    fn into_ast(self) -> Self::Ast {
        self.ast
    }
}

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

            impl IntoExpr for $rust {
                type Out = $output;
            }
            impl<'a> IntoExpr for &'a $rust {
                type Out = $output;
            }
         )+
    };
    ($output:ty, ref $rust:ty) => {
        impl<'a> IntoAst for &'a $rust {
            type Ast = Self;
            fn into_ast(self) -> Self {
                self
            }
        }
        impl<'a> IntoExpr for &'a $rust {
            type Out = $output;
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
            impl<OfT: IntoAst> IntoAst for [OfT; $len] {
                type Ast = Term<ArrayVec<[OfT::Ast; $len]>>;
                fn into_ast(self) -> Self::Ast {
                    term(term::MAKE_ARRAY,
                         ArrayVec::from(self).into_iter().map(|value| {
                             value.into_ast()
                         }).collect())
                }
            }

            impl<OfT: IntoExpr> IntoExpr for [OfT; $len] {
                type Out = ArrayOut<OfT::Out>;
            }
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

        impl<$head, $($tail),*> IntoExpr for ($head, $($tail),*)
            where $head: IntoAst, $($tail: IntoAst),*
        {
            type Out = ArrayOut<AnyOut>;
        }

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

pub trait MinLimitFor<OutT>: IntoAst {}
pub trait MaxLimitFor<OutT>: IntoAst {}

impl<ExprT: IntoExpr> MinLimitFor<ExprT::Out> for ExprT {}
impl<ExprT: IntoExpr> MaxLimitFor<ExprT::Out> for ExprT {}

impl<OutT> MinLimitFor<OutT> for MinVal {}
impl<OutT> MaxLimitFor<OutT> for MaxVal {}

#[derive(Copy, Clone, Debug)]
pub enum StringOut {}
#[derive(Copy, Clone, Debug)]
pub struct ArrayOut<OfT>(PhantomData<*const OfT>);
#[derive(Copy, Clone, Debug)]
pub struct SelectionOut<OfT>(PhantomData<*const OfT>);
#[derive(Copy, Clone, Debug)]
pub struct SingleSelectionOut<OfT>(PhantomData<*const OfT>);
#[derive(Copy, Clone, Debug)]
pub struct StreamOut<OfT>(PhantomData<*const OfT>);
#[derive(Copy, Clone, Debug)]
pub struct FunctionOut<ArgsT, ReturnT>(PhantomData<*const (ArgsT, ReturnT)>);
#[derive(Copy, Clone, Debug)]
pub enum ObjectOut {}
#[derive(Copy, Clone, Debug)]
pub enum BoolOut {}
#[derive(Copy, Clone, Debug)]
pub enum NumberOut {}
#[derive(Copy, Clone, Debug)]
pub enum NullOut {}
#[derive(Copy, Clone, Debug)]
pub enum AnyOut {}

#[derive(Copy, Clone, Debug)]
pub enum TableOut {}
#[derive(Copy, Clone, Debug)]
pub enum DbOut {}

#[derive(Copy, Clone, Debug)]
pub struct NullOr<OfT>(PhantomData<*const OfT>);

pub trait IsDb {}
impl IsDb for DbOut {}

pub trait IsTable {}
impl IsTable for TableOut {}

pub trait IsObject {}
impl IsObject for ObjectOut {}
impl IsObject for SingleSelectionOut<ObjectOut> {}

pub trait IsString {}
impl IsString for StringOut {}
impl IsString for SingleSelectionOut<StringOut> {}

pub trait IsNumber {}
impl IsNumber for NumberOut {}
impl IsNumber for SingleSelectionOut<NumberOut> {}

pub trait IsBool {}
impl IsBool for BoolOut {}
impl IsBool for SingleSelectionOut<BoolOut> {}

pub trait IsKey {}
impl IsKey for StringOut {}
impl IsKey for NumberOut {}
impl IsKey for SingleSelectionOut<StringOut> {}
impl IsKey for SingleSelectionOut<NumberOut> {}

pub trait IsSelector {}
impl IsSelector for StringOut {}
impl IsSelector for ObjectOut {}

pub trait IsIndexKey {}
impl<OfT> IsIndexKey for ArrayOut<OfT> {}
impl<OfT> IsIndexKey for SelectionOut<OfT> {}
impl<OfT> IsIndexKey for SingleSelectionOut<OfT> {}
impl<OfT> IsIndexKey for StreamOut<OfT> {}
impl IsIndexKey for BoolOut {}
impl IsIndexKey for NumberOut {}
impl IsIndexKey for StringOut {}

pub trait IsObjectOrObjectSequence {
    type AnyOrAnySequence;
}
impl IsObjectOrObjectSequence for ObjectOut {
    type AnyOrAnySequence = AnyOut;
}
impl<OfT: IsObject> IsObjectOrObjectSequence for SingleSelectionOut<OfT> {
    type AnyOrAnySequence = AnyOut;
}
impl<OfT: IsObject> IsObjectOrObjectSequence for SelectionOut<OfT> {
    type AnyOrAnySequence = SelectionOut<AnyOut>;
}
impl<OfT: IsObject> IsObjectOrObjectSequence for StreamOut<OfT> {
    type AnyOrAnySequence = StreamOut<AnyOut>;
}
impl<OfT: IsObject> IsObjectOrObjectSequence for ArrayOut<OfT> {
    type AnyOrAnySequence = ArrayOut<AnyOut>;
}
impl IsObjectOrObjectSequence for TableOut {
    type AnyOrAnySequence = SelectionOut<AnyOut>;
}

pub trait IsSequence {
    type SequenceItem;
    type Select;
}

pub trait Rebind<ToT>: IsSequence {
    type Rebound: IsSequence<SequenceItem = ToT>;
}

impl IsSequence for TableOut {
    type SequenceItem = ObjectOut;
    type Select = SingleSelectionOut<ObjectOut>;
}

impl<ToT> Rebind<ToT> for TableOut {
    type Rebound = StreamOut<ToT>;
}

impl<OfT> IsSequence for ArrayOut<OfT> {
    type SequenceItem = OfT;
    type Select = OfT;
}

impl<ToT, OfT> Rebind<ToT> for ArrayOut<OfT> {
    type Rebound = ArrayOut<ToT>;
}

impl<OfT> IsSequence for StreamOut<OfT> {
    type SequenceItem = OfT;
    type Select = OfT;
}

impl<ToT, OfT> Rebind<ToT> for StreamOut<OfT> {
    type Rebound = StreamOut<ToT>;
}

impl<OfT> IsSequence for SelectionOut<OfT> {
    type SequenceItem = OfT;
    type Select = SingleSelectionOut<OfT>;
}

impl<ToT, OfT> Rebind<ToT> for SelectionOut<OfT> {
    type Rebound = StreamOut<ToT>;
}

pub trait IsEqualComparable<WithT> {}
impl IsEqualComparable<BoolOut> for BoolOut {}
impl IsEqualComparable<NumberOut> for NumberOut {}
impl IsEqualComparable<StringOut> for StringOut {}
impl IsEqualComparable<ObjectOut> for ObjectOut {}
impl<WithT, OfT> IsEqualComparable<ArrayOut<WithT>> for ArrayOut<OfT>
where
    OfT: IsEqualComparable<WithT>,
{
}

pub trait IsSelection<OfT> {}
impl<OfT> IsSelection<OfT> for SelectionOut<OfT> {}
impl<OfT> IsSelection<OfT> for SingleSelectionOut<OfT> {}
impl<ObjectT> IsSelection<ObjectT> for TableOut {}

pub trait CanAdd<WithT> {
    type Output;
}
impl CanAdd<NumberOut> for NumberOut {
    type Output = NumberOut;
}
impl CanAdd<StringOut> for StringOut {
    type Output = StringOut;
}
impl<WithT, OfT> CanAdd<ArrayOut<WithT>> for ArrayOut<OfT> {
    type Output = ArrayOut<AnyOut>;
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

impl<Arg1T, ReturnRawT, FunctionT> IntoFunctionExpr<(Arg1T,), ReturnRawT::Out> for FunctionT
where
    FunctionT: FnOnce(Var<Arg1T>) -> ReturnRawT,
    ReturnRawT: IntoExpr,
{
    type FunctionAst = Term<(Term<(usize,)>, ReturnRawT::Ast)>;

    fn into_function_expr(self) -> Expr<FunctionOut<(Arg1T,), ReturnRawT::Out>, Self::FunctionAst> {
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

#[derive(Copy, Clone, Debug, Serialize, Default)]
pub struct NoOptions {}

impl Options for NoOptions {
    fn all_unset(&self) -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug, Serialize, Default)]
pub struct GetAllOptions<IndexT: OptionValue = ()> {
    #[serde(skip_serializing_if = "OptionValue::is_unset")] index: IndexT,
}

impl<IndexT: OptionValue> Options for GetAllOptions<IndexT> {
    fn all_unset(&self) -> bool {
        self.index.is_unset()
    }
}

impl<NameT: IntoExpr<Out=StringOut>> WithOption<IndexOption, NameT> for GetAllOptions<()> {
    type WithOption = GetAllOptions<Expr<StringOut, NameT::Ast>>;

    fn with_option(self, value: NameT) -> Self::WithOption {
        GetAllOptions {
            index: value.into_expr(),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Default)]
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
    NameT: IntoExpr<Out=StringOut>,
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

// FIXME: Should use `IsString` rather than StringOut
impl<
    IndexT: OptionValue,
    LeftBoundT: IntoExpr<Out=StringOut>,
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
    RightBoundT: IntoExpr<Out=StringOut>,
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
