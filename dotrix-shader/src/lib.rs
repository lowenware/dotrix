use dotrix_assets as assets;
use dotrix_log as log;
use dotrix_types::id;
use std::collections::HashMap;

pub const NAMESPACE: u64 = 0x03;

pub struct Shader {
    name: String,
    code: Code,
}

impl Shader {
    pub fn new(name: &str, code: Code) -> Self {
        Self {
            name: String::from(name),
            code,
        }
    }

    pub fn try_as_str(&self) -> Option<&str> {
        if self.code.nodes.len() == 1 {
            if let Some(Node::Text(text)) = self.code.nodes.first() {
                return Some(text);
            }
        }
        None
    }

    pub fn code(&self, variables: Option<&HashMap<String, String>>) -> String {
        self.code.compose(variables)
    }
}

#[derive(Debug)]
pub struct Code {
    nodes: Vec<Node>,
}

impl Code {
    pub fn empty() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn compose(&self, variables: Option<&HashMap<String, String>>) -> String {
        self.nodes
            .iter()
            .map(|node| node.compose(variables))
            .filter(|chunk| chunk.is_some())
            .map(|chunk| chunk.unwrap())
            .collect::<String>()
    }
}

impl From<String> for Code {
    fn from(text: String) -> Self {
        Self {
            nodes: vec![Node::Text(text)],
        }
    }
}

#[derive(Debug)]
pub enum Node {
    Text(String),
    Var {
        name: String,
    },
    Block {
        condition: Option<String>,
        code: Code,
    },
    LineBreak,
}

impl Node {
    fn compose(&self, variables: Option<&HashMap<String, String>>) -> Option<String> {
        match self {
            Node::Text(text) => Some(String::from(text)),
            Node::Var { name } => variables
                .map(|map| map.get(name).map(|value| String::from(value)))
                .unwrap_or(None),
            Node::Block { condition, code } => {
                if condition
                    .as_ref()
                    .map(|variable| {
                        variables
                            .map(|map| map.get(variable).is_some())
                            .unwrap_or(false)
                    })
                    .unwrap_or(true)
                {
                    Some(code.compose(variables))
                } else {
                    None
                }
            }
            Node::LineBreak => Some(String::from("\n")),
        }
    }
}

impl id::NameSpace for Shader {
    fn namespace() -> u64 {
        assets::NAMESPACE | NAMESPACE
    }
}

impl assets::Asset for Shader {
    fn name(&self) -> &str {
        &self.name
    }
    fn namespace(&self) -> u64 {
        <Self as id::NameSpace>::namespace()
    }
}

pub struct ShaderLoader {
    regex_macro: regex::Regex,
}

const MACRO: &str =
    r"(?i)\$\{(([a-z0-9_]+)|(:if\(([a-z0-9_]+)\))|(:include\(([a-z0-9_\-\.]+)\))|(:end))}";

impl Default for ShaderLoader {
    fn default() -> Self {
        Self {
            regex_macro: regex::Regex::new(MACRO).unwrap(),
        }
    }
}

impl ShaderLoader {
    fn push_node(stack: &mut Vec<(Option<String>, Code)>, node: Node) {
        if let Some((_, code)) = stack.last_mut() {
            code.nodes.push(node);
        }
    }

    pub fn parse(
        &self,
        stack: &mut Vec<(Option<String>, Code)>,
        path: &std::path::Path,
        data: &[u8],
    ) {
        let code_str = match std::str::from_utf8(&data) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        for line in code_str.lines() {
            let line_len = line.len();
            let mut offset = 0;
            let mut text_nodes = 0;
            let mut macro_nodes = 0;
            for captures in self.regex_macro.captures_iter(line) {
                if let Some(macro_match) = captures.get(1) {
                    let start = macro_match.start() - 2;
                    if start > offset {
                        let text_node = Node::Text(String::from(&line[offset..start]));
                        Self::push_node(stack, text_node);
                        text_nodes += 1;
                    }
                    offset = macro_match.end() + 1;

                    let macro_match_str = macro_match.as_str();
                    if macro_match_str.starts_with(":if") {
                        let condition = captures.get(4).map(|m| String::from(m.as_str()));
                        stack.push((condition, Code::empty()));
                    } else if macro_match_str.starts_with(":include") {
                        let file_name = captures.get(6).unwrap();
                        stack.push((None, Code::empty()));
                        self.parse_file(stack, path, file_name.as_str());
                        let (condition, code) = stack.pop().unwrap();
                        Self::push_node(stack, Node::Block { condition, code });
                    } else if macro_match_str.starts_with(":end") {
                        let (condition, code) = stack.pop().unwrap();
                        Self::push_node(stack, Node::Block { condition, code });
                    } else {
                        let name = String::from(macro_match_str);
                        Self::push_node(stack, Node::Var { name });
                    }
                }
                macro_nodes += 1;
            }
            if offset < line_len {
                let text_node = Node::Text(String::from(&line[offset..line_len]));
                Self::push_node(stack, text_node);
                text_nodes += 1;
            }
            if text_nodes != 0 || macro_nodes == 0 {
                Self::push_node(stack, Node::LineBreak);
            }
        }
    }

    pub fn parse_file(
        &self,
        stack: &mut Vec<(Option<String>, Code)>,
        parent_path: &std::path::Path,
        file_name: &str,
    ) {
        use std::io::Read;

        let file_path = parent_path.parent().unwrap().join(file_name);

        if let Ok(mut fs_file) = std::fs::File::open(&file_path) {
            let mut data = Vec::with_capacity(
                fs_file
                    .metadata()
                    .map(|md| md.len() as usize)
                    .unwrap_or(512),
            );
            if fs_file.read_to_end(&mut data).is_ok() {
                self.parse(stack, &file_path, &data);
                return;
            }
        }
        log::error!(
            "failed to include file '{:?}' from '{:?}'",
            &file_path,
            parent_path
        );
    }
}

impl assets::Loader for ShaderLoader {
    fn can_load(&self, path: &std::path::Path) -> bool {
        path.extension()
            .map(|e| e.to_str().unwrap().eq_ignore_ascii_case("wgsl"))
            .unwrap_or(false)
    }

    fn load(&self, path: &std::path::Path, data: Vec<u8>) -> Vec<Box<dyn assets::Asset>> {
        let mut result = Vec::with_capacity(1);
        let mut stack = vec![(None, Code::empty())];
        self.parse(&mut stack, path, &data);
        if stack.len() == 1 {
            if let Some((_, code)) = stack.pop() {
                let name = path.file_stem().map(|n| n.to_str().unwrap()).unwrap();
                let asset = Box::new(Shader::new(name, code));
                result.push(asset as Box<dyn assets::Asset>);
            }
        }
        result
    }
}

/* TODO: unit tests
 *
 * const CODE: &str = "
 * Here is my code, Hello ${user}!
 *
 * Lets create some ${:if(showOptional)}optional${:end} block
 * ${:include(test.wgsl)}
 * ${:if(optionalBlock)}
 * optional Block
 * ${:end}
 * ";
 *
 * let shader_loader = shader::ShaderLoader::default();
 * let mut stack = vec![(None, shader::Code::empty())];
 * let mut map = std::collections::HashMap::new();
 * map.insert(String::from("MY_CONDITION"), String::from("1"));
 * map.insert(String::from("incVar"), String::from("incVar:OK"));
 * map.insert(String::from("showOptional"), String::from("1"));
 * map.insert(String::from("optionalBlock"), String::from("1"));
 * map.insert(String::from("user"), String::from("Elias"));
 * shader_loader.parse(
 *     &mut stack,
 *     &std::path::Path::new("./phantom.wgsl"),
 *     CODE.as_bytes(),
 * );
 * let (_, code) = stack.pop().unwrap();
 *
 * println!("CODE:\n{}", code.compose(&map));
 * return;
 *
 * #[cfg(test)]
 * mod tests {
 *    #[test]
 *    fn it_works() {
 *        let result = 2 + 2;
 *        assert_eq!(result, 4);
 *    }
 * }
 */
