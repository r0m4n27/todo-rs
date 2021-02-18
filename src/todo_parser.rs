use regex::{escape, Regex};

use crate::todo::Todo;

const COMMENT_PATTERN: &str = "^({})(?P<comment>.+)$";

// Memory usage is probably high on big files
// As the will be completly loaded into ram
pub fn find_todos(keywords: &Vec<String>, input: &str) -> Vec<Todo> {
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
                todo.comments
                    .push(m.name("comment").unwrap().as_str().to_owned())
            }
        }
    }

    if let Some(todo) = last_todo {
        output.push(todo)
    }

    output
}

fn todo_regex(keywords: &Vec<String>) -> Regex {
    let prefix = "(?P<prefix>.*)";
    let keyword = format!("(?P<keyword>{})", keywords.join("|"));
    let title = "(?P<title>.+)";
    let issue_id = r"(\(#(?P<issue_id>\d+)\))";

    Regex::new(&format!("^{}{}{}?: {}$", prefix, keyword, issue_id, title)).unwrap()
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
                prefix: "// ".to_owned(),
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
                prefix: "// ".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: None,
                comments: vec!["More".to_owned(), "And more".to_owned()],
            };

            assert_eq!(vec![expected], find_todos(&vec!["TODO".to_owned()], input))
        }

        #[test]
        fn parse_comment_escape() {
            let input = "// TODO: Something\n// More\n// And (\\d+) more";
            let expected = Todo {
                line: 1,
                prefix: "// ".to_owned(),
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
                prefix: "// ".to_owned(),
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
                prefix: "// ".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: None,
                comments: vec!["More".to_owned()],
            };

            let expected_two = Todo {
                line: 3,
                prefix: "// ".to_owned(),
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
}
