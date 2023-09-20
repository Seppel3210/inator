use inator::*;

#[inline]
fn parenthesized(automaton: Parser<char>) -> Parser<char> {
    d('(') >> automaton >> d(')')
}

fn main() {
    let abc = d('A') | d('B') | d('C');
    println!("abc:");
    println!("{abc}");

    // let left_paren = d('(');
    // println!("left_paren:");
    // println!("{left_paren}");

    // let right_paren = d(')');
    // println!("right_paren:");
    // println!("{right_paren}");

    // let left_paren_abc = d('(') >> abc;
    // println!("left_paren_abc:");
    // println!("{left_paren_abc}");

    // let in_parentheses = left_paren_abc << d(')');
    // let in_parentheses = left_paren_abc >> d(')');
    let in_parentheses = parenthesized(abc);
    println!("in_parentheses:");
    println!("{in_parentheses}");

    let compiled = in_parentheses.compile();
    println!("compiled:");
    println!("{compiled}");

    for fuzz in compiled.fuzz().unwrap().take(10) {
        println!("Fuzz: {}", fuzz.into_iter().collect::<String>());
    }

    assert!(compiled.accept("(A)".chars()));
}
