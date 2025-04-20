use crate::{
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

// #[test]
// pub fn test_simple_optimization() {
//     let args = CompilerArgs {
//         file: PathBuf::from("./rinari.bf"),
//         output: None,
//         optimize: true,
//         print: false,
//     };

//     let program = args.run();
// }
