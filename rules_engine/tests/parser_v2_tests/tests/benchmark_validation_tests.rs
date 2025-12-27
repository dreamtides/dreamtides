use parser_v2_benchmarks::benchmark_utils;

#[test]
fn test_parser_bench_draw_cards() {
    benchmark_utils::parse_single_card("Draw {cards}.", "cards: 2");
}

#[test]
fn test_full_pipeline_bench() {
    let cards_file = benchmark_utils::load_cards_toml();
    benchmark_utils::parse_all_cards(cards_file);
}
