use std::collections::BTreeMap;
use std::path::PathBuf;

use ttysvg::redact::Rules;
use ttysvg::tape;

const README: &str = include_str!("../README.md");
const FENCE: &str = "```tape";

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n").trim_end().to_string()
}

fn examples() -> BTreeMap<String, String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut found = BTreeMap::new();

    for entry in std::fs::read_dir(&dir).expect("examples directory is missing") {
        let path = entry.expect("unreadable entry").path();
        if path.extension().is_some_and(|e| e == "tape") {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let body = std::fs::read_to_string(&path).expect("unreadable tape");
            found.insert(name, body);
        }
    }

    assert!(!found.is_empty(), "no tapes found in {}", dir.display());
    found
}

fn readme_blocks() -> Vec<String> {
    blocks_in(README)
}

fn example_section_blocks() -> Vec<String> {
    let start = README
        .find("\n## Examples")
        .expect("the readme has no examples section");
    blocks_in(&README[start..])
}

fn blocks_in(source: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut rest = source;

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
fn every_example_tape_parses_and_does_something() {
    for (name, body) in examples() {
        let parsed =
            tape::parse(&body).unwrap_or_else(|e| panic!("examples/{name} does not parse: {e:#}"));
        assert!(!parsed.ops.is_empty(), "examples/{name} does nothing");
        assert!(
            parsed.config.output.extension().is_some_and(|e| e == "svg"),
            "examples/{name} does not write an svg"
        );
        assert!(
            parsed.config.cols >= 20 && parsed.config.rows >= 6,
            "examples/{name} has an unusable size"
        );
        Rules::from_config(&parsed.config)
            .unwrap_or_else(|e| panic!("examples/{name} has a bad redact pattern: {e:#}"));
    }
}

#[test]
fn every_readme_tape_block_parses() {
    let blocks = readme_blocks();
    assert!(!blocks.is_empty(), "the readme shows no tapes at all");

    for (i, block) in blocks.iter().enumerate() {
        tape::parse(block)
            .unwrap_or_else(|e| panic!("readme tape block {} does not parse: {e:#}", i + 1));
    }
}

#[test]
fn every_example_section_block_is_a_real_file() {
    let files: Vec<String> = examples().values().map(|b| normalize(b)).collect();
    let blocks = example_section_blocks();

    assert!(
        blocks.len() >= 9,
        "expected the examples section to quote the tapes, found {}",
        blocks.len()
    );

    for (i, block) in blocks.iter().enumerate() {
        let block = normalize(block);
        assert!(
            files.contains(&block),
            "examples section block {} does not match any file in examples/, so the docs \
             have drifted from what actually runs. first line: {:?}",
            i + 1,
            block.lines().next().unwrap_or("")
        );
    }
}

#[test]
fn every_readme_block_points_at_the_examples_folder() {
    for (name, _) in examples() {
        if name == "demo.tape" {
            continue;
        }
        let reference = format!("examples/{name}");
        assert!(
            README.contains(&reference),
            "examples/{name} is never mentioned in the readme"
        );
    }
}
