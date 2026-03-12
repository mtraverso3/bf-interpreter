/// A single node in the Brainfuck AST.
#[derive(Debug, Clone)]
pub enum Node {
    /// `>`
    MoveRight,
    /// `<`
    MoveLeft,
    /// `+`
    Increment,
    /// `-`
    Decrement,
    /// `.`
    Output,
    /// `,`
    Input,
    /// `[ <body> ]`
    Loop(Vec<Node>),
}

impl Node {
    /// Get a human-readable symbol for this node.
    pub fn symbol(&self) -> char {
        match self {
            Node::MoveRight => '>',
            Node::MoveLeft => '<',
            Node::Increment => '+',
            Node::Decrement => '-',
            Node::Output => '.',
            Node::Input => ',',
            Node::Loop(_) => '[',
        }
    }
}

/// Parse a Brainfuck source string into an AST.
///
/// Returns `Err` with a human-readable message if the brackets are unbalanced.
pub fn parse(source: &str) -> Result<Vec<Node>, String> {
    // Each entry on the stack holds the nodes for one nesting level.
    // The bottom of the stack is the top-level program.
    let mut stack: Vec<Vec<Node>> = vec![Vec::new()];

    for (i, ch) in source.chars().enumerate() {
        match ch {
            '>' => stack.last_mut().unwrap().push(Node::MoveRight),
            '<' => stack.last_mut().unwrap().push(Node::MoveLeft),
            '+' => stack.last_mut().unwrap().push(Node::Increment),
            '-' => stack.last_mut().unwrap().push(Node::Decrement),
            '.' => stack.last_mut().unwrap().push(Node::Output),
            ',' => stack.last_mut().unwrap().push(Node::Input),
            '[' => stack.push(Vec::new()),
            ']' => {
                if stack.len() < 2 {
                    return Err(format!("Unmatched ']' at character {i}"));
                }
                let body = stack.pop().unwrap();
                stack.last_mut().unwrap().push(Node::Loop(body));
            }
            _ => {} // anything else is a comment
        }
    }

    if stack.len() > 1 {
        return Err(format!(
            "{} unclosed '[' bracket(s) in source",
            stack.len() - 1
        ));
    }

    Ok(stack.pop().unwrap())
}
