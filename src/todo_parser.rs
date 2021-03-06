use std::{borrow::Cow, collections::HashMap};

use regex::{escape, Captures, Regex};

use crate::todo::Todo;

const COMMENT_PATTERN: &str = "^({}) (?P<comment>.*)$|^({})$";

// Memory usage is probably high on big files
// As the will be completly loaded into ram
pub fn find_todos(keywords: &[String], input: &str) -> Vec<Todo> {
    let regex = todo_regex(keywords);
    let mut last_todo: Option<Todo> = None;
    let mut output: Vec<Todo> = vec![];

    for (line, text) in input.lines().enumerate() {
        let mut parsed = parse_line(&regex, &text);

        if let Some(ref mut new_todo) = parsed {
            if let Some(todo) = last_todo {
                output.push(todo);
            }

            new_todo.line = line as u32 + 1;
            last_todo = parsed;
        } else if let Some(ref mut todo) = last_todo {
            let reg = Regex::new(&COMMENT_PATTERN.replace("{}", &escape(&todo.prefix))).unwrap();

            if let Some(m) = reg.captures(&text) {
                todo.comments.push(
                    m.name("comment")
                        .map(|m| m.as_str())
                        .unwrap_or("")
                        .to_owned(),
                )
            }
        }
    }

    if let Some(todo) = last_todo {
        output.push(todo)
    }

    output
}

// Config guarantees that keywords won't be empty
fn todo_regex(keywords: &[String]) -> Regex {
    let prefix = "(?P<prefix>.*)";
    let keyword = format!("(?P<keyword>{})", keywords.join("|"));
    let title = "(?P<title>.+)";
    let issue_id = r"(\(#(?P<issue_id>\d+)\))";

    Regex::new(&format!("^{} {}{}?: {}$", prefix, keyword, issue_id, title)).unwrap()
}

fn parse_line(regex: &Regex, text: &str) -> Option<Todo> {
    regex.captures(&text).map(|c| Todo {
        prefix: c.name("prefix").unwrap().as_str().to_owned(),
        keyword: c.name("keyword").unwrap().as_str().to_owned(),
        title: c.name("title").unwrap().as_str().to_owned(),
        issue_id: c
            .name("issue_id")
            .map(|s| s.as_str().parse::<u32>().unwrap()),
        comments: vec![],

        // Line will be changed later
        line: 0,
    })
}

pub fn mark_todos<'a>(input: &'a str, todos: &[Todo]) -> Cow<'a, str> {
    let mut map = HashMap::new();
    let filtered_todos: Vec<_> = todos
        .into_iter()
        .filter_map(|t| {
            if let Some(reported) = t.reported_view() {
                let unreported = t.unreported_pattern();

                map.insert(unreported.clone(), reported);

                Some(unreported)
            } else {
                None
            }
        })
        .collect();

    if let Some(regex) = build_regex(&filtered_todos) {
        regex.replace_all(input, |cap: &Captures| {
            // Has to be escaped because the key is also escaped
            let key = escape(cap.get(0).unwrap().as_str());

            map.get(&key).unwrap()
        })
    } else {
        Cow::from(input)
    }
}

fn build_regex(todos: &[String]) -> Option<Regex> {
    if todos.len() == 0 {
        None
    } else {
        Some(Regex::new(&format!("(?m){}", todos.join("|"))).unwrap())
    }
}

pub fn remove_todos<'a>(input: &'a str, todos: &[Todo]) -> Cow<'a, str> {
    let filtered_todos: Vec<_> = todos
        .into_iter()
        .filter_map(|t| t.reported_pattern())
        .collect();

    if let Some(regex) = build_regex(&filtered_todos) {
        regex.replace_all(input, "")
    } else {
        Cow::from(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_regex() {
        let reg = todo_regex(&vec!["TODO".to_owned(), "FIXME".to_owned()]);

        assert_eq!(true, reg.is_match("// TODO: Something"));
        assert_eq!(true, reg.is_match("// FIXME: Something"));
        assert_eq!(false, reg.is_match("// BUG: Something"));
        assert_eq!(true, reg.is_match("// TODO(#123): Something"))
    }

    mod parse_file {
        use super::*;

        #[test]
        fn parse_one() {
            let input = "// TODO: Something";
            let expected = Todo {
                line: 1,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: None,
                comments: vec![],
            };

            assert_eq!(vec![expected], find_todos(&vec!["TODO".to_owned()], input))
        }

        #[test]
        fn parse_comments() {
            let input = "// TODO: Something\n// More\n// And more";
            let expected = Todo {
                line: 1,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: None,
                comments: vec!["More".to_owned(), "And more".to_owned()],
            };

            assert_eq!(vec![expected], find_todos(&vec!["TODO".to_owned()], input))
        }

        #[test]
        fn parse_comments_empty_line() {
            let input = "// TODO: Something\n// More\n// And more\n//";
            let expected = Todo {
                line: 1,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: None,
                comments: vec!["More".to_owned(), "And more".to_owned(), "".to_owned()],
            };

            assert_eq!(vec![expected], find_todos(&vec!["TODO".to_owned()], input))
        }

        #[test]
        fn parse_comment_escape() {
            let input = "// TODO: Something\n// More\n// And (\\d+) more";
            let expected = Todo {
                line: 1,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: None,
                comments: vec!["More".to_owned(), r"And (\d+) more".to_owned()],
            };

            assert_eq!(vec![expected], find_todos(&vec!["TODO".to_owned()], input))
        }

        #[test]
        fn parse_issue_id() {
            let input = "// TODO(#42): Something";
            let expected = Todo {
                line: 1,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: Some(42),
                comments: vec![],
            };

            assert_eq!(vec![expected], find_todos(&vec!["TODO".to_owned()], input))
        }

        #[test]
        fn parse_mutiple() {
            let input = "// TODO: Something\n// More\n// TODO: Other\n// comment";
            let expected_one = Todo {
                line: 1,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: None,
                comments: vec!["More".to_owned()],
            };

            let expected_two = Todo {
                line: 3,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Other".to_owned(),
                issue_id: None,
                comments: vec!["comment".to_owned()],
            };

            assert_eq!(
                vec![expected_one, expected_two],
                find_todos(&vec!["TODO".to_owned()], input)
            )
        }
    }

    mod build_regex {
        use super::*;

        #[test]
        fn build_empty() {
            assert_eq!(true, build_regex(&Vec::new()).is_none())
        }

        #[test]
        fn build_with_todos() {
            let regex = build_regex(&vec!["123".to_owned(), "456".to_owned(), "789".to_owned()]);

            assert_eq!("(?m)123|456|789", regex.unwrap().as_str())
        }
    }

    mod mark_todos {
        use super::*;

        #[test]
        fn mark_single() {
            let input = "// TODO: Something\n\nSomething Else";
            let expected = Todo {
                line: 1,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: Some(42),
                comments: vec![],
            };

            assert_eq!(
                "// TODO(#42): Something\n\nSomething Else".to_owned(),
                mark_todos(input, &vec![expected])
            )
        }

        #[test]
        fn mark_mutiple() {
            let input = "// TODO: Something\n\nSomething Else\n// TODO: Other";

            let todo_one = Todo {
                line: 1,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: Some(123),
                comments: vec!["More".to_owned()],
            };

            let todo_two = Todo {
                line: 3,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Other".to_owned(),
                issue_id: Some(456),
                comments: vec!["comment".to_owned()],
            };

            assert_eq!(
                "// TODO(#123): Something\n\nSomething Else\n// TODO(#456): Other".to_owned(),
                mark_todos(input, &vec![todo_one, todo_two])
            )
        }
    }

    mod remove_todos {
        use super::*;

        #[test]
        fn remove_simple() {
            let input = "// TODO(#42): Something\n\nSomething Else";
            let expected = Todo {
                line: 1,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: Some(42),
                comments: vec![],
            };

            assert_eq!(
                "\nSomething Else".to_owned(),
                remove_todos(input, &vec![expected])
            )
        }

        #[test]
        fn remove_mutiple() {
            let input = "// TODO(#123): Something\n// More\nSomething Else\n// TODO(#456): Other";

            let todo_one = Todo {
                line: 1,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: Some(123),
                comments: vec!["More".to_owned()],
            };

            let todo_two = Todo {
                line: 3,
                prefix: "//".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Other".to_owned(),
                issue_id: Some(456),
                comments: vec![],
            };

            assert_eq!(
                "Something Else\n".to_owned(),
                remove_todos(input, &vec![todo_one, todo_two])
            )
        }
    }
}
