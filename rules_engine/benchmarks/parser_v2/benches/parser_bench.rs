use chumsky::Parser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parser_v2::lexer::tokenize;
use parser_v2::parser::ability;
use parser_v2::variables::binding::VariableBindings;
use parser_v2::variables::substitution;

fn parse_single_card(text: &str, vars: &str) {
    let bindings = VariableBindings::parse(vars).expect("Failed to parse variables");
    let lex_result = tokenize::lex(text).expect("Failed to lex text");
    let resolved = substitution::resolve_variables(&lex_result.tokens, &bindings)
        .expect("Failed to resolve variables");
    let parser = ability::ability_parser();
    parser.parse(&resolved).into_result().expect("Failed to parse ability");
}

fn parser_benchmarks(c: &mut Criterion) {
    c.bench_function("parse_draw_cards", |b| {
        b.iter(|| {
            parse_single_card(black_box("Draw {cards}."), black_box("cards: 2"));
        });
    });
}

criterion_group!(benches, parser_benchmarks);
criterion_main!(benches);
