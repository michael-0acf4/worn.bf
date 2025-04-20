use std::path::PathBuf;

use crate::{
    cli::CompilerArgs,
    parser::{ast::Reconstruct, parse_program},
    wbf::WBFEmitter,
};
use insta::assert_debug_snapshot;

#[test]
pub fn test_simple_code_expansion() {
    let source = r#"
    super incr(n) {
        // R repeats the second argument the amount of the first argument
        super first(a) {
            super second(b) {
                super third(c) {
                    R(b, +)
                }

                super fourth(d) {
                    third(d)
                }

                fourth(b)
            }

            second(a)
        }

        first(n)
    }

    super six() {
        incr(6) // instruction as arg works as long as they are simple and does not have ',' as it is a valid bf instruction
    }

    super ten() {
        six()++++
    }

    super printASCII(index) {
        > ten() [>six()<-]> +++++ incr(index) .
    }

    printASCII(0) // A
    printASCII(1) // B
    printASCII(2) // C
    + . // D
    + . // E
    printASCII(0) // A
    printASCII(1) // B
    >65+. // B

    >>>>"CD"<<.>.>. // "CD" is a shortcut for  R(68, +)>R(69, +)
    "#;

    let ret = parse_program(&source).and_then(|program| {
        let mut emitter = WBFEmitter::new(program);
        emitter.compile().map_err(|e| e.to_string())?;
        emitter.finalize().map(|bi| bi.reconstruct())
    });

    assert_debug_snapshot!(ret)
}

#[test]
pub fn test_simple_optimization() {
    let file = PathBuf::from("./examples/rinari.bf");
    let no_opt = CompilerArgs {
        file: file.clone(),
        output: None,
        optimize: Some(0),
        print: false,
    }
    .run()
    .unwrap()
    .reconstruct();

    let after_opt = CompilerArgs {
        file: file.clone(),
        output: None,
        optimize: Some(1),
        print: false,
    }
    .run()
    .unwrap()
    .reconstruct();

    assert!(after_opt.len() < no_opt.len());
    let reduction_amount = 100.0 * after_opt.len() as f32 / no_opt.len() as f32;
    println!("Reduction {reduction_amount}%");
    assert!(reduction_amount < 45.0);
}

#[test]
pub fn test_complex_optimization() {
    let file = PathBuf::from("./src/tests/fold_me.wbf");
    let no_opt = CompilerArgs {
        file: file.clone(),
        output: None,
        optimize: Some(0),
        print: false,
    }
    .run()
    .unwrap()
    .reconstruct();

    let after_opt = CompilerArgs {
        file: file.clone(),
        output: None,
        optimize: Some(5),
        print: false,
    }
    .run()
    .unwrap()
    .reconstruct();

    assert!(after_opt.len() < no_opt.len());
    let reduction_amount = 100.0 * after_opt.len() as f32 / no_opt.len() as f32;
    println!("Reduction {reduction_amount}%");
    assert!(reduction_amount < 50.0);
}
