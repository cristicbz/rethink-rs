use ql2_proto;
pub use failure::Error;
use serde::{Serialize, Serializer};
use std::ops::Add;
use std::marker::PhantomData;

pub struct Expr<OutputT, AstT: Serialize> {
    ast: AstT,
    _phantom: PhantomData<*const OutputT>,
}

pub fn expr<OutputT, OfT: AsExpr<OutputT>>(of: OfT) -> Expr<OutputT, OfT::Ast> {
    of.as_expr()
}

impl<OutputT, AstT: Serialize> Expr<OutputT, AstT> {
    fn raw(ast: AstT) -> Self {
        Expr {
            ast,
            _phantom: PhantomData,
        }
    }
}

impl<OutputT, AstT: Serialize> Serialize for Expr<OutputT, AstT> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.ast.serialize(serializer)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Null;

impl Serialize for Null {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

pub enum StringOutput {}
pub struct ArrayOutput<OfT>(PhantomData<*const OfT>);
pub enum ObjectOutput {}
pub enum BoolOutput {}
pub enum NumberOutput {}
pub enum NullOutput {}
pub enum AnyOutput {}

pub enum TableOutput {}
pub enum DbOutput {}

pub struct NullOr<OfT>(PhantomData<*const OfT>);

pub trait IsKeyOutput {}
impl IsKeyOutput for StringOutput {}
impl IsKeyOutput for NumberOutput {}

pub trait IsAddOutput {}
impl IsAddOutput for StringOutput {}
impl IsAddOutput for NumberOutput {}
impl<OfT> IsAddOutput for ArrayOutput<OfT> {}
impl IsAddOutput for AnyOutput {}

pub trait IsIndexFor<WhatT> {
    type Item;
}

impl IsIndexFor<ObjectOutput> for StringOutput {
    type Item = AnyOutput;
}

impl IsIndexFor<TableOutput> for StringOutput {
    type Item = AnyOutput;
}

impl<OfT> IsIndexFor<ArrayOutput<OfT>> for NumberOutput {
    type Item = OfT;
}

impl IsIndexFor<StringOutput> for NumberOutput {
    type Item = StringOutput;
}

impl IsIndexFor<AnyOutput> for NumberOutput {
    type Item = AnyOutput;
}

impl IsIndexFor<AnyOutput> for StringOutput {
    type Item = AnyOutput;
}

pub trait AsExpr<OutputT> {
    type Ast: Serialize;
    fn as_expr(self) -> Expr<OutputT, Self::Ast>;
}

impl<OutputT, AstT: Serialize> AsExpr<OutputT> for Expr<OutputT, AstT> {
    type Ast = AstT;
    fn as_expr(self) -> Self {
        self
    }
}

impl<OutputT, AstT: Serialize> AsExpr<ArrayOutput<OutputT>> for Expr<AnyOutput, AstT> {
    type Ast = AstT;
    fn as_expr(self) -> Expr<ArrayOutput<OutputT>, AstT> {
        Expr::raw(self.ast)
    }
}

macro_rules! any_as_expr {
    ($($output:ty),+) => {
        $(
            impl<AstT: Serialize> AsExpr<$output> for Expr<AnyOutput, AstT> {
                type Ast = AstT;
                fn as_expr(self) -> Expr<$output, AstT> {
                    Expr::raw(self.ast)
                }
            }
        )+
    }
}

any_as_expr!(
    StringOutput,
    NumberOutput,
    BoolOutput,
    NullOutput,
    ObjectOutput
);

macro_rules! datum_as_expr {
    ($output:ty, $($rust:ty),+) => {
        $(
            impl AsExpr<$output> for $rust {
                type Ast = Self;
                fn as_expr(self) -> Expr<$output, Self> {
                    Expr::raw(self)
                }
            }
            impl<'a> AsExpr<$output> for &'a $rust {
                type Ast = Self;
                fn as_expr(self) -> Expr<$output, Self> {
                    Expr::raw(self)
                }
            }
         )+
    };
    ($output:ty, ref $rust:ty) => {
        impl<'a> AsExpr<$output> for &'a $rust {
            type Ast = Self;
            fn as_expr(self) -> Expr<$output, Self> {
                Expr::raw(self)
            }
        }
    };
}

datum_as_expr!(StringOutput, String);
datum_as_expr!(StringOutput, ref str);

datum_as_expr!(NumberOutput, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
datum_as_expr!(BoolOutput, bool);
datum_as_expr!(NullOutput, Null);


impl<AstT: Serialize> Expr<DbOutput, AstT> {
    pub fn table<NameT: AsExpr<StringOutput>>(
        self,
        name: NameT,
    ) -> Expr<TableOutput, Term<(AstT, NameT::Ast)>> {
        Expr::raw(Term(ql2_proto::mod_Term::TermType::TABLE as u32, (
            self.ast,
            name.as_expr()
                .ast,
        )))
    }
}

impl<OutputT, AstT: Serialize> Expr<NullOr<OutputT>, AstT> {
    pub fn assert_not_null(self) -> Expr<OutputT, AstT> {
        Expr::raw(self.ast)
    }
}

impl<AstT: Serialize> Expr<TableOutput, AstT> {
    pub fn get<OutputT: IsKeyOutput, KeyT: AsExpr<OutputT>>(
        self,
        key: KeyT,
    ) -> Expr<NullOr<ObjectOutput>, Term<(AstT, KeyT::Ast)>> {
        Expr::raw(Term(ql2_proto::mod_Term::TermType::GET as u32, (
            self.ast,
            key.as_expr().ast,
        )))
    }
}

impl<OutputT, AstT: Serialize> Expr<OutputT, AstT> {
    pub fn i<IndexT: IsIndexFor<OutputT>, AsIndexT: AsExpr<IndexT>>(
        self,
        index: AsIndexT,
    ) -> Expr<IndexT::Item, Term<(AstT, AsIndexT::Ast)>> {
        Expr::raw(Term(ql2_proto::mod_Term::TermType::BRACKET as u32, (
            self.ast,
            index
                .as_expr()
                .ast,
        )))
    }

    pub fn get_field<IndexT: IsIndexFor<OutputT>, AsIndexT: AsExpr<IndexT>>(
        self,
        index: AsIndexT,
    ) -> Expr<IndexT::Item, Term<(AstT, AsIndexT::Ast)>> {
        Expr::raw(Term(ql2_proto::mod_Term::TermType::GET_FIELD as u32, (
            self.ast,
            index
                .as_expr()
                .ast,
        )))
    }
}


pub fn db<NameT: AsExpr<StringOutput>>(name: NameT) -> Expr<DbOutput, Term<(NameT::Ast,)>> {
    Expr::raw(Term(
        ql2_proto::mod_Term::TermType::DB as u32,
        (name.as_expr().ast,),
    ))
}

pub fn table<NameT: AsExpr<StringOutput>>(name: NameT) -> Expr<TableOutput, Term<(NameT::Ast,)>> {
    Expr::raw(Term(ql2_proto::mod_Term::TermType::TABLE as u32, (
        name.as_expr()
            .ast,
    )))
}

#[derive(Serialize)]
pub struct Term<ArgsT: Serialize>(u32, ArgsT);

impl<OutputT: IsAddOutput, LhsT: Serialize, RhsT: AsExpr<OutputT>> Add<RhsT>
    for Expr<OutputT, LhsT> {
    type Output = Expr<OutputT, Term<(LhsT, RhsT::Ast)>>;

    fn add(self, other: RhsT) -> Self::Output {
        Expr::raw(Term(ql2_proto::mod_Term::TermType::ADD as u32, (
            self.ast,
            other.as_expr().ast,
        )))
    }
}
