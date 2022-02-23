// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

fn main() {
    let matches = App::new("autocxx-mdbook-preprocessor")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Expands code examples in the autocxx book.")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Whether a given renderer is supported by this preprocessor"),
        );
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        process::exit(0); // we support all.
    }
    preprocess.unwrap();
}

fn preprocess() -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    Book::for_each_mut(&mut book, |sec| {
        if let mdbook::BookItem::Chapter(chapter) = sec {
            chapter.content = substitute_chapter(&chapter.content);
        }
    });

    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn substitute_chapter(chapter: &str) -> String {
    let mut state = ChapterParseState::Start;
    let mut out = Vec::new();
    for line in chapter.lines() {
        let line_type = recognize_line(line);
        let mut push_line = true;
        state = match state {
            ChapterParseState::Start => match line_type {
                LineType::CodeBlockStart | LineType::CodeBlockEnd => {
                    ChapterParseState::OtherCodeBlock
                }
                LineType::CodeBlockStateNoPlayground => {
                    push_line = false;
                    ChapterParseState::CandidateCodeBlock(line.to_string())
                }
                LineType::Macro => panic!("Found macro outside code block"),
                LineType::Misc => ChapterParseState::Start,
            },
            ChapterParseState::OtherCodeBlock => match line_type {
                LineType::Macro => {
                    panic!("Found macro inside a code block which wasn't rust,noplayground")
                }
                LineType::CodeBlockEnd => ChapterParseState::Start,
                LineType::Misc => ChapterParseState::OtherCodeBlock,
                _ => panic!("Found confusing conflicting block markers"),
            },
            ChapterParseState::CandidateCodeBlock(entry_line) => match line_type {
                LineType::Macro => {
                    push_line = false;
                    ChapterParseState::OurCodeBlock(Vec::new())
                },
                LineType::Misc => {
                    out.push(entry_line);
                    ChapterParseState::OtherCodeBlock
                }
                LineType::EndCodeBlock => {
                    out.push(entry_line);
                    ChapterParseState::Start
                }
                _ => panic!("Found confusing conflicting block markers"),
            },
            ChapterParseState::OurCodeBlock(lines) => match line_type {
                LineType::Misc => {
                    push_line = false;
                    lines.push(line.to_string());
                    ChapterParseState::OurCodeBlock(lines)
                }
                LineType::EndCodeBlock => {
                    out.extend(handle_code_block(lines));
                    ChapterParseState::Start
                }
                _ => panic!("Found something unexpected in one of our code blocks"),
            },
        };
        if push_line {
            out.push(line.to_string());
        }
    }

    out.join("\n")
}
enum ChapterParseState {
    Start,
    OtherCodeBlock,
    CandidateCodeBlock(String), // have found rust,noplayground
    OurCodeBlock(Vec<String>),  // have found #[autocxx_integration_tests::doctest]
}

enum LineType {
    CodeBlockStart,
    CodeBlockStateNoPlayground,
    CodeBlockEnd,
    Macro,
    Misc,
}

fn recognize_line(line: &str) -> LineType {
    if line == "```rust" {
        LineType::CodeBlockStart
    } else if line == "```rust,noplayground" {
        LineType::CodeBlockStateNoPlayground
    } else if line == "#[autocxx_integration_tests::doctest]" {
        LineType::Macro
    } else if line == "```" {
        LineType::CodeBlockEnd
    } else {
        LineType::Misc
    }
}

fn handle_code_block(lines: Vec<String>) -> impl Iterator<Item = String> {
    let input = std::iter::once("fake_function(".to_string())
        .chain(lines.into_iter())
        .chain(std::iter::once(");"))
        .collect_vec();
    let fn_call: syn::ExprCall = syn::parse_str(&input.join("\n")).expect("Unable to parse");
    let mut args_iter = fn_call.args.iter();
    let cpp = extract_span(&input, args_iter.next().unwrap());
    let hdr = extract_span(&input, args_iter.next().unwrap());
    let rs = extract_span(&input, args_iter.next().unwrap());
    let mut output = Vec::new();
    output.push("C++ header:".to_string());
    output.push("```cpp".to_string());
    output.push(hdr.to_string());
    output.push("```".to_string());
    if !cpp.is_empty() {
        output.push("C++ implementation:".to_string());
        output.push("```cpp".to_string());
        output.push(cpp.to_string());
        output.push("```".to_string());
    }
    output.push("Rust:".to_string());
    output.push("```rust,noplayground".to_string());
    output.push(rs.to_string());
    output.push("```".to_string());
    output.into_iter()
}

fn extract_span(text: &[String], span: Span) -> Cow<str> {
    let start_line = span.start().line;
    let start_col = span.start().column;
    let end_line = span.end().line;
    let end_col = span.end().column;
    if start_line == end_line {
        Cow::Borrowed(&text[start_line][start_col..end_col])
    } else {
        let start_subset = text[start_line][start_col..];
        let end_subset = text[end_line][..end_col];
        let mid_lines = text[start_line + 1..end_line];
        Cow::Owned(
            std::iter::once(start_subset)
                .chain(mid_lines.iter())
                .chain(std::iter::once(end_subset))
                .join("\n"),
        )
    }
}
