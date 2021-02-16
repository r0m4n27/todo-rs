use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct Todo {
    pub line: u32,
    pub prefix: String,
    pub keyword: String,
    pub title: String,
    pub issue_id: Option<String>,
    pub comments: Vec<String>,
}

impl Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let issue_str = if let Some(ref issue) = self.issue_id {
            format!("({})", issue)
        } else {
            String::new()
        };

        let comments_str = if self.comments.len() == 0 {
            String::new()
        } else {
            let mut out = String::from("\n  ");
            out.push_str(&self.comments.join("\n  "));

            out
        };

        write!(
            f,
            "{}: {}{}: {}{}",
            self.line, self.keyword, issue_str, self.title, comments_str
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_without_issue_and_comments() {
        let todo = Todo {
            line: 10,
            prefix: String::from(r"// "),
            keyword: String::from("TODO"),
            title: String::from("Something"),
            issue_id: None,
            comments: vec![],
        };

        assert_eq!("10: TODO: Something", format!("{}", todo))
    }

    #[test]
    fn display_with_issue() {
        let todo = Todo {
            line: 10,
            prefix: String::from(r"// "),
            keyword: String::from("TODO"),
            title: String::from("Something"),
            issue_id: Some(String::from("#42")),
            comments: vec![],
        };

        assert_eq!("10: TODO(#42): Something", format!("{}", todo))
    }

    #[test]
    fn display_with_comments() {
        let todo = Todo {
            line: 10,
            prefix: String::from(r"// "),
            keyword: String::from("TODO"),
            title: String::from("Something"),
            issue_id: None,
            comments: vec!["More".to_owned(), "And More".to_owned()],
        };

        assert_eq!(
            "10: TODO: Something\n  More\n  And More",
            format!("{}", todo)
        )
    }
}
