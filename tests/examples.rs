use ttysvg::redact::Rules;
use ttysvg::tape;

const README: &str = include_str!("../README.md");
const FENCE: &str = "```tape";

fn tape_blocks() -> Vec<String> {
    let mut blocks = Vec::new();
    let mut rest = README;

    while let Some(open) = rest.find(FENCE) {
        let after = &rest[open + FENCE.len()..];
        let Some(nl) = after.find('\n') else { break };
        let body = &after[nl + 1..];
        let close = body
            .find("```")
            .expect("unterminated tape block in the readme");
        blocks.push(body[..close].to_string());
        rest = &body[close..];
    }

    blocks
}

#[test]
fn every_readme_example_parses() {
    let blocks = tape_blocks();
    assert!(
        blocks.len() >= 8,
        "expected the readme to carry example tapes, found {}",
        blocks.len()
    );

    for (i, block) in blocks.iter().enumerate() {
        let parsed = tape::parse(block)
            .unwrap_or_else(|e| panic!("readme tape block {} does not parse: {e:#}", i + 1));
        assert!(
            !parsed.ops.is_empty(),
            "readme tape block {} does nothing",
            i + 1
        );
    }
}

#[test]
fn every_readme_example_compiles_its_redact_patterns() {
    for (i, block) in tape_blocks().iter().enumerate() {
        let parsed = tape::parse(block).unwrap();
        Rules::from_config(&parsed.config)
            .unwrap_or_else(|e| panic!("readme tape block {} has a bad pattern: {e:#}", i + 1));
    }
}

#[test]
fn every_readme_example_names_an_output_and_a_size() {
    for (i, block) in tape_blocks().iter().enumerate() {
        let cfg = tape::parse(block).unwrap().config;
        assert!(
            cfg.output.extension().is_some_and(|e| e == "svg"),
            "readme tape block {} does not write an svg",
            i + 1
        );
        assert!(
            cfg.cols >= 20 && cfg.rows >= 6,
            "readme tape block {} has an unusable size",
            i + 1
        );
    }
}
