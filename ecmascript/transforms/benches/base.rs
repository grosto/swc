#![feature(test)]
#![feature(specialization)]
#![feature(box_syntax)]

extern crate swc_common;
extern crate swc_ecma_ast;
extern crate swc_ecma_parser;
extern crate swc_ecma_transforms;
extern crate test;
extern crate testing;

use swc_common::{FileName, Fold, FoldWith, Visit, VisitWith, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_parser::{Parser, Session, SourceFileInput};
use swc_ecma_transforms::util::ExprFactory;
use test::Bencher;

static SOURCE: &'static str = r#"
'use strict';
/**
 * Extract red color out of a color integer:
 *
 * 0x00DEAD -> 0x00
 *
 * @param  {Number} color
 * @return {Number}
 */
function red( color )
{
    let foo = 3.14;
    return color >> 16;
}
/**
 * Extract green out of a color integer:
 *
 * 0x00DEAD -> 0xDE
 *
 * @param  {Number} color
 * @return {Number}
 */
function green( color )
{
    return ( color >> 8 ) & 0xFF;
}
/**
 * Extract blue color out of a color integer:
 *
 * 0x00DEAD -> 0xAD
 *
 * @param  {Number} color
 * @return {Number}
 */
function blue( color )
{
    return color & 0xFF;
}
/**
 * Converts an integer containing a color such as 0x00DEAD to a hex
 * string, such as '#00DEAD';
 *
 * @param  {Number} int
 * @return {String}
 */
function intToHex( int )
{
    const mask = '#000000';
    const hex = int.toString( 16 );
    return mask.substring( 0, 7 - hex.length ) + hex;
}
/**
 * Converts a hex string containing a color such as '#00DEAD' to
 * an integer, such as 0x00DEAD;
 *
 * @param  {Number} num
 * @return {String}
 */
function hexToInt( hex )
{
    return parseInt( hex.substring( 1 ), 16 );
}
module.exports = {
    red,
    green,
    blue,
    intToHex,
    hexToInt,
};"#;

#[bench]
fn module_clone(b: &mut Bencher) {
    let _ = ::testing::run_test(|logger, cm, handler| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());

        let mut parser = Parser::new(
            Session {
                logger: &logger,
                handler: &handler,
                cfg: Default::default(),
            },
            SourceFileInput::from(&*fm),
        );
        let module = parser.parse_module().unwrap();

        b.iter(|| test::black_box(module.clone()));
        Ok(())
    });
}

#[bench]
fn fold_empty(b: &mut Bencher) {
    struct Noop;
    let _ = ::testing::run_test(|logger, cm, handler| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());

        let mut parser = Parser::new(
            Session {
                logger: &logger,
                handler: &handler,
                cfg: Default::default(),
            },
            SourceFileInput::from(&*fm),
        );
        let module = parser.parse_module().unwrap();
        let mut folder = Noop;

        b.iter(|| test::black_box(module.clone().fold_with(&mut folder)));
        Ok(())
    });
}

/// Optimized out
#[bench]
fn fold_noop_impl_all(b: &mut Bencher) {
    struct Noop;
    impl<T> Fold<T> for Noop
    where
        T: FoldWith<Self>,
    {
        fn fold(&mut self, node: T) -> T {
            node
        }
    }
    let _ = ::testing::run_test(|logger, cm, handler| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());

        let mut parser = Parser::new(
            Session {
                logger: &logger,
                handler: &handler,
                cfg: Default::default(),
            },
            SourceFileInput::from(&*fm),
        );
        let module = parser.parse_module().unwrap();
        let mut folder = Noop;

        b.iter(|| test::black_box(module.clone().fold_with(&mut folder)));
        Ok(())
    });
}

/// Optimized out
#[bench]
fn fold_noop_impl_vec(b: &mut Bencher) {
    struct Noop;
    impl<T> Fold<Vec<T>> for Noop
    where
        Vec<T>: FoldWith<Self>,
    {
        fn fold(&mut self, node: Vec<T>) -> Vec<T> {
            node
        }
    }
    let _ = ::testing::run_test(|logger, cm, handler| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());

        let mut parser = Parser::new(
            Session {
                logger: &logger,
                handler: &handler,
                cfg: Default::default(),
            },
            SourceFileInput::from(&*fm),
        );
        let module = parser.parse_module().unwrap();
        let mut folder = Noop;

        b.iter(|| test::black_box(module.clone().fold_with(&mut folder)));
        Ok(())
    });
}

fn mk_expr() -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        callee: Ident::new("foo".into(), DUMMY_SP).as_callee(),
        args: vec![],
    })
}

#[bench]
fn boxing_boxed_clone(b: &mut Bencher) {
    let _ = ::testing::run_test(|_, _, _| {
        let expr = box mk_expr();

        b.iter(|| test::black_box(expr.clone()));
        Ok(())
    });
}

#[bench]
fn boxing_unboxed_clone(b: &mut Bencher) {
    let _ = ::testing::run_test(|_, _, _| {
        let expr = mk_expr();

        b.iter(|| test::black_box(expr.clone()));
        Ok(())
    });
}

#[bench]
fn boxing_boxed(b: &mut Bencher) {
    struct Noop;

    let _ = ::testing::run_test(|_, _, _| {
        let mut folder = Noop;
        let expr = box mk_expr();

        b.iter(|| test::black_box(expr.clone().fold_with(&mut folder)));
        Ok(())
    });
}

#[bench]
fn boxing_unboxed(b: &mut Bencher) {
    struct Noop;

    let _ = ::testing::run_test(|_, _, _| {
        let mut folder = Noop;
        let expr = mk_expr();

        b.iter(|| test::black_box(expr.clone().fold_with(&mut folder)));
        Ok(())
    });
}

#[bench]
fn visit_empty(b: &mut Bencher) {
    struct Noop;
    let _ = ::testing::run_test(|logger, cm, handler| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());

        let mut parser = Parser::new(
            Session {
                logger: &logger,
                handler: &handler,
                cfg: Default::default(),
            },
            SourceFileInput::from(&*fm),
        );
        let module = parser.parse_module().unwrap();
        let mut v = Noop;

        b.iter(|| test::black_box(module.visit_with(&mut v)));
        Ok(())
    });
}

#[bench]
fn visit_contains_this(b: &mut Bencher) {
    fn contains_this_expr(body: &Module) -> bool {
        struct Visitor {
            found: bool,
        }

        impl Visit<ThisExpr> for Visitor {
            fn visit(&mut self, _: &ThisExpr) {
                self.found = true;
            }
        }

        impl Visit<FnExpr> for Visitor {
            /// Don't recurse into fn
            fn visit(&mut self, _: &FnExpr) {}
        }

        impl Visit<FnDecl> for Visitor {
            /// Don't recurse into fn
            fn visit(&mut self, _: &FnDecl) {}
        }

        let mut visitor = Visitor { found: false };
        body.visit_with(&mut visitor);
        visitor.found
    }
    let _ = ::testing::run_test(|logger, cm, handler| {
        let fm = cm.new_source_file(FileName::Anon, SOURCE.into());

        let mut parser = Parser::new(
            Session {
                logger: &logger,
                handler: &handler,
                cfg: Default::default(),
            },
            SourceFileInput::from(&*fm),
        );
        let module = parser.parse_module().unwrap();

        b.iter(|| test::black_box(contains_this_expr(&module)));
        Ok(())
    });
}
