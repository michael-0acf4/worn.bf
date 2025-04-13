use crate::parser::{ast::Reconstruct, parse_program};
use insta::assert_debug_snapshot;

#[test]
fn test_super_instruction() {
    assert_debug_snapshot!(
        parse_program(
            "
        super instr(arg1, arg2, arg3) {
            +-arg1 arg2 arg3
        }

        super instr2() {
            instr(++, --, super instr3() {
                instr(++, --, ++)
            })
        }
        ",
        )
        .map(|is| is.reconstruct())
    )
}

#[test]
fn test_expr_with_comments() {
    let out = parse_program(
        "
        /*
        * Awesome
        */

        super deluxe() { // skip me
        +
    }

        yay(1, 2, /* and me*/3,

        // and also me
        4,
        /* and me*/f(f(   5,   6 )))
        ",
    );
    assert_eq!(
        out.map(|is| is.reconstruct()),
        Ok("super deluxe() {\n +\n}\n\nyay(1, 2, 3, 4, f(f(5, 6)))".to_string())
    );
}
