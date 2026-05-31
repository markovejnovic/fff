#[path = "rg_compat/hay.rs"]
mod hay;
#[path = "rg_compat/util.rs"]
mod util;

use hay::SHERLOCK;
use util::Dir;

#[test]
fn smoke_basic_search() {
    let dir = Dir::new("smoke");
    dir.create("sherlock", SHERLOCK);

    let out = dir.command().arg("--color=never").arg("--no-heading").arg("Sherlock").stdout();
    assert!(out.contains("Sherlock"), "expected Sherlock in output, got: {out}");
    let lines: Vec<&str> = out.lines().filter(|l| *l != "--").collect();
    assert_eq!(lines.len(), 2, "expected 2 matching lines, got {}: {out}", lines.len());
}


#[test]
fn case_insensitive() {
    let dir = Dir::new("case_i");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--no-heading", "-i", "sherlock"]).stdout();
    let lines: Vec<&str> = out.lines().filter(|l| *l != "--").collect();
    assert_eq!(lines.len(), 2, "expected 2 case-insensitive matches, got: {out}");
    assert_eq!(
        lines[0],
        "sherlock:For the Doctor Watsons of this world, as opposed to the Sherlock"
    );
    assert_eq!(
        lines[1],
        "sherlock:be, to a very large extent, the result of luck. Sherlock Holmes"
    );
}

#[test]
fn smart_case_lower() {
    let dir = Dir::new("smart_lower");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--no-heading", "sherlock"]).stdout();
    assert!(out.contains("Sherlock"), "smart case should match uppercase with lowercase query");
    let lines: Vec<&str> = out.lines().filter(|l| *l != "--").collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn smart_case_upper() {
    let dir = Dir::new("smart_upper");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--no-heading", "Sherlock"]).stdout();
    let lines: Vec<&str> = out.lines().filter(|l| *l != "--").collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Sherlock"));
}

#[test]
fn case_sensitive() {
    let dir = Dir::new("case_s");
    dir.create("sherlock", SHERLOCK);
    let code = dir.command().args(&["--color=never", "--no-heading", "-s", "sherlock"]).exit_code();
    assert_eq!(code, 1, "case-sensitive 'sherlock' should find nothing");
}

#[test]
fn fixed_strings() {
    let dir = Dir::new("fixed");
    dir.create("test", "foo.bar\nfooXbar\n");
    let out = dir.command().args(&["--color=never", "--no-heading", "-F", "foo.bar"]).stdout();
    assert_eq!(out, "test:foo.bar\n");
}

#[test]
fn fixed_strings_regex_chars() {
    let dir = Dir::new("fixed_regex");
    dir.create("test", "a(b)c\nabc\n");
    let out = dir.command().args(&["--color=never", "--no-heading", "-F", "a(b)c"]).stdout();
    assert_eq!(out, "test:a(b)c\n");
}

#[test]
fn no_match_exit_code() {
    let dir = Dir::new("no_match");
    dir.create("sherlock", SHERLOCK);
    let code = dir.command().args(&["--color=never", "--no-heading", "ZZZZNOTFOUND"]).exit_code();
    assert_eq!(code, 1);
}

#[test]
fn match_exit_code() {
    let dir = Dir::new("match_exit");
    dir.create("sherlock", SHERLOCK);
    let code = dir.command().args(&["--color=never", "--no-heading", "Sherlock"]).exit_code();
    assert_eq!(code, 0);
}


#[test]
fn line_numbers() {
    let dir = Dir::new("line_num");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--no-heading", "-n", "Sherlock"]).stdout();
    let lines: Vec<&str> = out.lines().filter(|l| *l != "--").collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("sherlock:1:"));
    assert!(lines[1].starts_with("sherlock:3:"));
}

#[test]
fn column_numbers() {
    let dir = Dir::new("columns");
    dir.create("sherlock", SHERLOCK);
    let out = dir
        .command()
        .args(&["--color=never", "--no-heading", "-n", "--column", "Sherlock"])
        .stdout();
    let lines: Vec<&str> = out.lines().filter(|l| *l != "--").collect();
    assert_eq!(lines.len(), 2);
    // Format: file:line:col:content — at least 4 colon-separated parts
    let parts: Vec<&str> = lines[0].splitn(4, ':').collect();
    assert_eq!(parts.len(), 4, "expected file:line:col:content format");
    assert_eq!(parts[0], "sherlock");
}

#[test]
fn heading_mode() {
    let dir = Dir::new("heading");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--heading", "Sherlock"]).stdout();
    let lines: Vec<&str> = out.lines().filter(|l| *l != "--").collect();
    // First line should be just the filename (heading)
    assert_eq!(lines[0], "sherlock");
    // Match lines should NOT have filename prefix
    assert!(!lines[1].starts_with("sherlock:"));
    assert!(lines[1].contains("Sherlock"));
}

#[test]
fn heading_with_line_numbers() {
    let dir = Dir::new("heading_ln");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--heading", "-n", "Sherlock"]).stdout();
    let lines: Vec<&str> = out.lines().filter(|l| *l != "--").collect();
    assert_eq!(lines[0], "sherlock");
    assert!(lines[1].starts_with("1:"), "expected line number prefix, got: {}", lines[1]);
    assert!(lines[2].starts_with("3:"), "expected line number prefix, got: {}", lines[2]);
}

#[test]
fn no_filename() {
    let dir = Dir::new("no_filename");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--no-heading", "-I", "Sherlock"]).stdout();
    let lines: Vec<&str> = out.lines().filter(|l| *l != "--").collect();
    assert!(!lines[0].starts_with("sherlock:"), "should not have filename prefix");
    assert!(lines[0].contains("Sherlock"));
}

#[test]
fn count() {
    let dir = Dir::new("count");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--no-heading", "-c", "Sherlock"]).stdout();
    assert_eq!(out, "sherlock:2\n");
}

#[test]
fn files_with_matches() {
    let dir = Dir::new("files_match");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--no-heading", "-l", "Sherlock"]).stdout();
    assert_eq!(out, "sherlock\n");
}

#[test]
fn quiet_match() {
    let dir = Dir::new("quiet_match");
    dir.create("sherlock", SHERLOCK);
    let mut cmd = dir.command();
    cmd.args(&["--color=never", "--no-heading", "-q", "Sherlock"]);
    let out = cmd.stdout();
    assert!(out.is_empty(), "quiet mode should produce no output, got: {out}");
}

#[test]
fn quiet_no_match() {
    let dir = Dir::new("quiet_nomatch");
    dir.create("sherlock", SHERLOCK);
    let code =
        dir.command().args(&["--color=never", "--no-heading", "-q", "ZZZZNOTFOUND"]).exit_code();
    assert_eq!(code, 1);
}

#[test]
fn vimgrep() {
    let dir = Dir::new("vimgrep");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--vimgrep", "Sherlock"]).stdout();
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "expected 2 vimgrep lines, got: {out}");
    for line in &lines {
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        assert_eq!(parts.len(), 4, "vimgrep format: file:line:col:content");
        assert_eq!(parts[0], "sherlock");
        assert!(parts[1].parse::<u64>().is_ok(), "line should be numeric");
        assert!(parts[2].parse::<u64>().is_ok(), "col should be numeric");
    }
}

#[test]
fn files_mode() {
    let dir = Dir::new("files_mode");
    dir.create("alpha.txt", "content");
    dir.create("beta.rs", "fn main() {}");
    let out = dir.command().args(&["--color=never", "--files"]).stdout();
    let mut lines: Vec<&str> = out.lines().collect();
    lines.sort();
    assert!(lines.contains(&"alpha.txt"), "should list alpha.txt, got: {out}");
    assert!(lines.contains(&"beta.rs"), "should list beta.rs, got: {out}");
}

/// Without context flags, non-adjacent matches should NOT have -- separator
#[test]
fn no_separator_without_context() {
    let dir = Dir::new("no_sep");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--no-heading", "Sherlock"]).stdout();
    assert!(!out.contains("--"), "should not emit -- separator without context flags, got: {out}");
}


#[test]
fn after_context() {
    let dir = Dir::new("after_ctx");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--no-heading", "-A1", "Sherlock"]).stdout();
    // Should have match lines + context lines
    assert!(out.contains("Holmeses"), "after-context should include next line");
    assert!(out.contains("can extract"), "after-context should include next line for 2nd match");
}

#[test]
fn after_context_line_numbers() {
    let dir = Dir::new("after_ctx_ln");
    dir.create("sherlock", SHERLOCK);
    let out =
        dir.command().args(&["--color=never", "--no-heading", "-A1", "-n", "Sherlock"]).stdout();
    // Match lines use ":" separator, context lines use "-" separator
    assert!(out.contains("sherlock:1:"), "should have match with line number");
    assert!(out.contains("sherlock-2-"), "context line should use - separator");
}

#[test]
fn before_context() {
    let dir = Dir::new("before_ctx");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--no-heading", "-B1", "Sherlock"]).stdout();
    // Line 3 matches; line 2 should be before-context
    assert!(out.contains("Holmeses"), "before-context of 2nd match should include line 2");
}

#[test]
fn before_context_line_numbers() {
    let dir = Dir::new("before_ctx_ln");
    dir.create("sherlock", SHERLOCK);
    let out =
        dir.command().args(&["--color=never", "--no-heading", "-B1", "-n", "Sherlock"]).stdout();
    assert!(out.contains("sherlock:1:"), "first match at line 1");
    assert!(out.contains("sherlock-2-"), "before-context for 2nd match");
    assert!(out.contains("sherlock:3:"), "second match at line 3");
}

#[test]
fn context_separator() {
    let dir = Dir::new("ctx_sep");
    dir.create("sherlock", SHERLOCK);
    // "world" is on line 1, "attached" is on line 6 — gap between them
    let out =
        dir.command().args(&["--color=never", "--no-heading", "-C1", "world|attached"]).stdout();
    assert!(out.contains("--"), "should have -- separator between non-adjacent groups");
}

#[test]
fn trim_whitespace() {
    let dir = Dir::new("trim");
    dir.create("indented", "    indented line\nnormal line\n");
    let out = dir.command().args(&["--color=never", "--no-heading", "--trim", "indented"]).stdout();
    // --trim strips leading whitespace
    assert!(out.contains("indented:indented line"), "should trim leading spaces, got: {out}");
    assert!(!out.contains("    indented"), "leading spaces should be stripped");
}

#[test]
fn max_count() {
    let dir = Dir::new("max_count");
    dir.create("sherlock", SHERLOCK);
    let out = dir.command().args(&["--color=never", "--no-heading", "-m1", "Sherlock"]).stdout();
    let lines: Vec<&str> = out.lines().filter(|l| *l != "--").collect();
    assert_eq!(lines.len(), 1, "max-count 1 should return 1 line, got: {out}");
    assert!(lines[0].contains("Sherlock"));
}
