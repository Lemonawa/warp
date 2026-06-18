use std::borrow::Cow;

use crate::{
    check_if_token_has_shell_syntax, is_word, natural_language_words_score, token_preprocessing,
    wrapped_in_quotes, WordDb,
};

// --- is_word ---

#[test]
fn is_word_recognizes_common_english_word() {
    assert!(is_word("hello", WordDb::English));
}

#[test]
fn is_word_rejects_gibberish_in_english_db() {
    assert!(!is_word("xyzzyplugh", WordDb::English));
}

#[test]
fn is_word_recognizes_command_in_command_db() {
    assert!(is_word("git", WordDb::Command));
}

#[test]
fn is_word_rejects_english_word_in_command_db() {
    // "beautiful" is an English word but not a command
    assert!(!is_word("beautiful", WordDb::Command));
}

// --- check_if_token_has_shell_syntax ---

#[test]
fn shell_syntax_detected_for_variable_expansion() {
    assert!(check_if_token_has_shell_syntax("$HOME"));
}

#[test]
fn shell_syntax_detected_for_pipe() {
    assert!(check_if_token_has_shell_syntax("foo|bar"));
}

#[test]
fn shell_syntax_detected_for_redirect() {
    assert!(check_if_token_has_shell_syntax("file>out"));
}

#[test]
fn shell_syntax_detected_for_path() {
    assert!(check_if_token_has_shell_syntax("/usr/bin/env"));
}

#[test]
fn shell_syntax_detected_for_flag() {
    assert!(check_if_token_has_shell_syntax("--verbose"));
}

#[test]
fn shell_syntax_not_detected_for_plain_word() {
    assert!(!check_if_token_has_shell_syntax("hello"));
}

#[test]
fn shell_syntax_not_detected_for_word_with_space() {
    // Words with spaces are not single tokens; the function returns false.
    assert!(!check_if_token_has_shell_syntax("foo bar"));
}

// --- wrapped_in_quotes ---

#[test]
fn double_quoted_string_detected() {
    assert!(wrapped_in_quotes("\"hello world\""));
}

#[test]
fn single_quoted_string_detected() {
    assert!(wrapped_in_quotes("'hello world'"));
}

#[test]
fn unquoted_string_not_detected() {
    assert!(!wrapped_in_quotes("hello"));
}

#[test]
fn mismatched_quotes_not_detected() {
    assert!(!wrapped_in_quotes("\"hello'"));
}

// --- token_preprocessing ---

#[test]
fn preprocessing_lowercases_token() {
    assert_eq!(token_preprocessing("HELLO"), "hello");
}

#[test]
fn preprocessing_expands_contraction_is() {
    // "he's" → "he"
    assert_eq!(token_preprocessing("he's"), "he");
}

#[test]
fn preprocessing_expands_contraction_not() {
    // "don't" → "do"
    assert_eq!(token_preprocessing("don't"), "do");
}

#[test]
fn preprocessing_special_case_cant() {
    assert_eq!(token_preprocessing("can't"), "can");
}

#[test]
fn preprocessing_expands_contraction_would() {
    // "I'll" → "i"
    assert_eq!(token_preprocessing("I'll"), "i");
}

#[test]
fn preprocessing_leaves_plain_word_unchanged() {
    assert_eq!(token_preprocessing("hello"), "hello");
}

// --- natural_language_words_score ---

#[test]
fn score_for_pure_english_sentence() {
    let words: Vec<Cow<str>> = vec!["show".into(), "me".into(), "the".into(), "files".into()];
    let score = natural_language_words_score(words, false);
    assert!(score >= 3, "Expected score >= 3, got {score}");
}

#[test]
fn score_for_pure_shell_command() {
    let words: Vec<Cow<str>> = vec!["ls".into(), "-la".into(), "/tmp".into()];
    let score = natural_language_words_score(words, true);
    // "ls" is a command as first token and is_first_token_command=true, so skipped.
    // "-la" has shell syntax, "/tmp" has shell syntax
    assert_eq!(score, 0);
}

#[test]
fn score_skips_first_command_when_flagged() {
    let words: Vec<Cow<str>> = vec!["git".into(), "status".into()];
    let score_with_cmd = natural_language_words_score(words.clone(), true);
    let score_without_cmd = natural_language_words_score(words, false);
    // With first-token-is-command, "git" is skipped; without, "git" is counted as command db word
    assert!(score_with_cmd <= score_without_cmd);
}

#[test]
fn score_empty_input_returns_zero() {
    let words: Vec<Cow<str>> = vec![];
    assert_eq!(natural_language_words_score(words, false), 0);
}

#[test]
fn score_mixed_nl_and_shell() {
    let words: Vec<Cow<str>> = vec!["list".into(), "all".into(), "$HOME/files".into()];
    let score = natural_language_words_score(words, false);
    // "list" and "all" are NL words, "$HOME/files" has shell syntax
    // So score should be 2 - 1 = 1
    assert!(score >= 1, "Expected score >= 1, got {score}");
}
