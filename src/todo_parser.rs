use std::io::{self, BufRead};

use regex::Regex;

use crate::todo::Todo;

pub struct TodoParser<'a> {
    todo_regex: Regex,
    comment_pattern: &'a str,
}

impl<'a> TodoParser<'a> {
    pub fn new(
        keywords: &Vec<String>,
        reported: bool,
        unreported: bool,
    ) -> Result<Self, &'static str> {
        if keywords.len() == 0 {
            return Err("Must provide at least one keyword!");
        }

        Ok(Self {
            todo_regex: TodoParser::create_todo_regex(keywords, reported, unreported)?,
            comment_pattern: "^({})(?P<comment>.*)$",
        })
    }

    fn create_todo_regex(
        keywords: &Vec<String>,
        reported: bool,
        unreported: bool,
    ) -> Result<Regex, &'static str> {
        let prefix = "(?P<prefix>.*)";
        let keyword = format!("(?P<keyword>{})", keywords.join("|"));
        let title = "(?P<title>.*)";
        let issue_id = r"(\((?P<issue_id>.*)\))";

        let todo_format = match (unreported, reported) {
            (true, true) => format!("^{}{}{}?: {}$", prefix, keyword, issue_id, title),
            (false, true) => format!("^{}{}{}: {}$", prefix, keyword, issue_id, title),
            (true, false) => format!("^{}{}: {}$", prefix, keyword, title),
            (false, false) => return Err("The pattern must be (un)-reported or both!"),
        };

        Ok(Regex::new(&todo_format).unwrap())
    }

    pub fn parse_file(&self, input: impl BufRead) -> io::Result<Vec<Todo>> {
        let mut last_todo: Option<Todo> = None;
        let mut output: Vec<Todo> = vec![];

        for (line, text) in input.lines().enumerate() {
            let text = text?;

            let parsed = self.parse_line(line as u32 + 1, &text);

            if parsed.is_some() {
                if let Some(todo) = last_todo {
                    output.push(todo);
                }

                last_todo = parsed;
            } else if let Some(ref mut todo) = last_todo {
                let reg = Regex::new(&self.comment_pattern.replace("{}", &todo.prefix)).unwrap();

                if let Some(m) = reg.captures(&text) {
                    todo.comments
                        .push(m.name("comment").unwrap().as_str().to_owned())
                }
            }
        }

        if let Some(todo) = last_todo {
            output.push(todo)
        }

        Ok(output)
    }

    fn parse_line(&self, line: u32, text: &str) -> Option<Todo> {
        self.todo_regex.captures(&text).map(|c| Todo {
            prefix: c.name("prefix").unwrap().as_str().to_owned(),
            keyword: c.name("keyword").unwrap().as_str().to_owned(),
            title: c.name("title").unwrap().as_str().to_owned(),
            issue_id: c.name("issue_id").map(|s| s.as_str().to_owned()),
            comments: vec![],
            line,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod create_regex {
        use super::*;

        #[test]
        fn create_regex_err() {
            if let Err(message) = TodoParser::create_todo_regex(&Vec::new(), false, false) {
                assert_eq!("The pattern must be (un)-reported or both!", message);
            } else {
                assert!(false)
            }
        }

        #[test]
        fn create_regex_keywords() {
            let reg = TodoParser::create_todo_regex(
                &vec!["TODO".to_owned(), "FIXME".to_owned()],
                false,
                true,
            )
            .unwrap();

            assert_eq!(true, reg.is_match("// TODO: Something"));
            assert_eq!(true, reg.is_match("// FIXME: Something"));
            assert_eq!(false, reg.is_match("// BUG: Something"))
        }

        #[test]
        fn create_regex_reported() {
            let reg = TodoParser::create_todo_regex(
                &vec!["TODO".to_owned(), "FIXME".to_owned()],
                true,
                false,
            )
            .unwrap();

            assert_eq!(false, reg.is_match("// TODO: Something"));
            assert_eq!(true, reg.is_match("// TODO(#123): Something"));
        }

        #[test]
        fn create_regex_both() {
            let reg = TodoParser::create_todo_regex(
                &vec!["TODO".to_owned(), "FIXME".to_owned()],
                true,
                true,
            )
            .unwrap();

            assert_eq!(true, reg.is_match("// TODO: Something"));
            assert_eq!(true, reg.is_match("// TODO(#123): Something"));
        }
    }

    #[test]
    fn new_parser_err() {
        if let Err(message) = TodoParser::new(&Vec::new(), false, true) {
            assert_eq!("Must provide at least one keyword!", message);
        } else {
            assert!(false)
        }
    }

    mod parse_file {
        use io::Cursor;

        use super::*;

        #[test]
        fn parse_one() {
            let parser = TodoParser::new(&vec!["TODO".to_owned()], false, true).unwrap();
            let input = Cursor::new("// TODO: Something");
            let expected = Todo {
                line: 1,
                prefix: "// ".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: None,
                comments: vec![],
            };

            assert_eq!(vec![expected], parser.parse_file(input).unwrap())
        }

        #[test]
        fn parse_comments() {
            let parser = TodoParser::new(&vec!["TODO".to_owned()], false, true).unwrap();
            let input = Cursor::new("// TODO: Something\n// More\n// And more");
            let expected = Todo {
                line: 1,
                prefix: "// ".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: None,
                comments: vec!["More".to_owned(), "And more".to_owned()],
            };

            assert_eq!(vec![expected], parser.parse_file(input).unwrap())
        }

        #[test]
        fn parse_issue_id() {
            let parser = TodoParser::new(&vec!["TODO".to_owned()], true, true).unwrap();
            let input = Cursor::new("// TODO(#42): Something");
            let expected = Todo {
                line: 1,
                prefix: "// ".to_owned(),
                keyword: "TODO".to_owned(),
                title: "Something".to_owned(),
                issue_id: Some("#42".to_owned()),
                comments: vec![],
            };

            assert_eq!(vec![expected], parser.parse_file(input).unwrap())
        }

        #[test]
        fn parse_mutiple() {
            let parser = TodoParser::new(&vec!["TODO".to_owned()], false, true).unwrap();
            let input = Cursor::new("// TODO: Something\n// More\n// TODO: Other\n// comment");
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
                parser.parse_file(input).unwrap()
            )
        }
    }
}
