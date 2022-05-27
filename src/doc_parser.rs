use std::fs;
use serde::Deserialize;
use serde_json::Value;

pub struct DocParser {
    json: Value
}

#[derive(Deserialize)]
struct Class {
    name: String,
    desc: Option<Vec<String>>,
    import_path: Option<String>,
    static_members: Option<Vec<Member>>,
    members: Option<Vec<Member>>,
    constructors: Option<Vec<Constructor>>,
    methods: Option<Vec<Function>>,
    static_methods: Option<Vec<Function>>
}

#[derive(Deserialize)]
struct Function {
    name: String,
    deprecated: Option<Vec<String>>,
    desc: Option<Vec<String>>,
    params: Option<Vec<Param>>,
    returns: Option<Return>,
    throws: Option<Vec<String>>,
    examples: Option<Vec<String>>
}

#[derive(Deserialize)]
struct Constructor {
    desc: Vec<String>,
    params: Option<Vec<Param>>,
    examples: Vec<String>
}

#[derive(Deserialize)]
struct Member {
    name: String,
    assignable: Option<bool>,
    desc: Option<Vec<String>>,
    #[serde(rename = "type")]
    type_name: Option<String>,
    examples: Option<Vec<String>>
}

#[derive(Deserialize)]
struct Param {
    name: String,
    #[serde(rename = "type")]
    type_name: String,
    desc: String
}

#[derive(Deserialize)]
struct Return {
    #[serde(rename = "type")]
    type_name: String,
    desc: String
}

impl DocParser {
    pub fn new(path: &str) -> DocParser {
        let content = fs::read_to_string(path).unwrap();
        DocParser {
            json: serde_json::from_str(&content).unwrap()
        }
    }

    pub fn parse_extensions(&self) -> String {
        let extensions = self.json["extensions"].as_object().unwrap();
        let mut iter = extensions.iter().peekable();
        let mut md = String::new();

        while let Some(extension) = iter.next() {
            let name = extension.0;
            let functions = extension.1.as_array().unwrap();
            md.push_str(&DocParser::parse_extension(name, functions));

            if iter.peek().is_some() {
                md.push_str("\n\n");
            }
        }

        md
    }

    pub fn parse_classes(&self) -> String {
        let classes = self.json["classes"].as_object().unwrap();
        let mut iter = classes.values().peekable();
        let mut md = String::new();

        while let Some(class) = iter.next() {
            md.push_str(&DocParser::parse_class(class));

            if iter.peek().is_some() {
                md.push_str("\n\n");
            }
        }

        md
    }

    fn parse_extension(name: &str, functions: &Vec<Value>) -> String {
        let mut md = String::new();

        md.push_str("## ");
        md.push_str(name);
        md.push_str("\n\n");

        for i in 0..functions.len() {
            let f: &Value = &functions[i];
            let function = serde_json::from_value(f.clone()).unwrap();
            let func_s = DocParser::add_function(None, function);
            if func_s.is_none() {
                continue;
            }

            md.push_str(&func_s.unwrap());

            if i + 1 < functions.len() {
                md.push('\n');
            }
        }

        md
    }

    fn parse_class(class_v: &Value) -> String {
        let mut md = String::new();

        let class: Class = serde_json::from_value(class_v.to_owned()).unwrap();

        // Class name
        md.push_str("# ");
        md.push_str(&class.name);
        md.push_str(" class\n");
        md.push_str(&class.name);
        md.push_str(" class for Arucas.\n\n");

        // Class description
        if let Some(desc) = class.desc {
            DocParser::add_from_string_array(&mut md, &desc);
            md.push('\n');
        }

        // Class import path (if needed)
        if let Some(import_path) = class.import_path {
            md.push_str("Import with `import ");
            md.push_str(&import_path);
            md.push_str(" from ");
            md.push_str(&import_path);
            md.push_str(";`\n\n");
        }
        else {
            md.push_str("Class does not need to be imported.\n\n");
        }

        md.push_str("Fully Documented.\n\n");

        // Static members of the class
        if let Some(statics) = class.static_members {
            if !statics.is_empty() {
                md.push_str("## Static Members\n\n");
                DocParser::add_member(&mut md, &class.name, &statics);
                md.push('\n');
            }
        }

        // Instance members (wrappers)
        if let Some(members) = class.members {
            if !members.is_empty() {
                let member_class = String::new() + "<" + &class.name + ">";
                md.push_str("## Members\n\n");
                DocParser::add_member(&mut md, &member_class, &members);
                md.push('\n');
            }
        }

        // Constructors
        if let Some(constructors) = class.constructors {
            if !constructors.is_empty() {
                md.push_str("## Constructors\n\n");
                let mut iter = constructors.into_iter().peekable();
                while let Some(constructor) = iter.next() {
                    md.push_str("### `new ");
                    md.push_str(&class.name);
                    md.push('(');

                    if let Some(params) = &constructor.params {
                        DocParser::add_params_in_function(&mut md, params);
                    }

                    md.push_str(")`\n");

                    DocParser::add_description(&mut md, &constructor.desc);

                    if let Some(params) = &constructor.params {
                        DocParser::add_params(&mut md, params);
                    }

                    DocParser::add_examples(&mut md, &constructor.examples);
                }
                md.push_str("\n");
            }
        }

        // Methods
        if let Some(methods) = class.methods {
            if !methods.is_empty() {
                md.push_str("## Methods\n\n");
                let member_class = String::new() + "<" + &class.name + ">";
                let mut iter = methods.into_iter().peekable();
                while let Some(value) = iter.next() {
                    let func_s = DocParser::add_function(Some(&member_class), value);
                    if func_s.is_none() {
                        continue;
                    }

                    md.push_str(&func_s.unwrap());

                    if iter.peek().is_some() {
                        md.push_str("\n");
                    }
                }
                md.push_str("\n");
            }
        }

        // Static methods
        if let Some(static_methods) = class.static_methods {
            if !static_methods.is_empty() {
                md.push_str("## Static Methods\n\n");
                let mut iter = static_methods.into_iter().peekable();
                while let Some(value) = iter.next() {
                    let func_s = DocParser::add_function(Some(&class.name), value);
                    if func_s.is_none() {
                        continue;
                    }

                    md.push_str(&func_s.unwrap());

                    if iter.peek().is_some() {
                        md.push_str("\n");
                    }
                }
            }
        }

        md
    }

    fn add_function(class_op: Option<&str>, function: Function) -> Option<String> {
        // Every function should have an example
        if function.examples.is_none() {
            return None;
        }

        let mut md = String::new();

        md.push_str("### `");
        if let Some(class) = class_op {
            md.push_str(class);
            md.push('.');
        }
        md.push_str(&function.name);
        md.push('(');

        if let Some(params) = &function.params {
            DocParser::add_params_in_function(&mut md, params)
        }

        md.push_str(")`\n");

        if let Some(deprecation) = &function.deprecated {
            md.push_str("- Deprecated: ");
            DocParser::add_from_string_array(&mut md, deprecation);
        }

        DocParser::add_description(&mut md, &function.desc.unwrap());

        if let Some(params) = &function.params {
            DocParser::add_params(&mut md, params);
        }

        if let Some(returns) = &function.returns {
            md.push_str("- Returns - ");
            md.push_str(&returns.type_name);
            md.push_str(": ");
            md.push_str(&returns.desc);
            md.push_str("\n");
        }

        if let Some(throws) = &function.throws {
            md.push_str("- Throws - Error:\n");
            for value in throws {
                md.push_str("  - `'");
                md.push_str(value);
                md.push_str("'`\n");
            }
        }

        DocParser::add_examples(&mut md, &function.examples.unwrap());

        Some(md)
    }

    fn add_params_in_function(md: &mut String, params: &Vec<Param>) {
        for i in 0..params.len() {
            let param: &Param = &params[i];
            md.push_str(&param.name);

            if i + 1 < params.len() {
                md.push_str(", ");
            }
        }
    }

    fn add_member(md: &mut String, class_name: &str, members: &Vec<Member>) {
        for member in members {
            // Every member should have this field, otherwise invalid
            if member.assignable.is_none() {
                continue;
            }

            md.push_str("### `");
            md.push_str(class_name);
            md.push_str(".");
            md.push_str(&member.name);
            md.push_str("`\n");

            DocParser::add_description(md, &member.desc.as_ref().unwrap());

            md.push_str("- Type: ");
            md.push_str(&member.type_name.as_ref().unwrap());
            md.push('\n');

            md.push_str("- Assignable: ");
            md.push_str(&member.assignable.unwrap().to_string());
            md.push('\n');

            DocParser::add_examples(md, &member.examples.as_ref().unwrap());
        }
    }

    fn add_description(md: &mut String, desc: &Vec<String>) {
        md.push_str("- Description: ");
        DocParser::add_from_string_array(md, desc);
    }

    fn add_params(md: &mut String, params: &Vec<Param>) {
        if params.len() == 1 {
            let param = &params[0];
            md.push_str("- Parameter - ");
            md.push_str(&param.type_name);
            md.push_str(" (`");
            md.push_str(&param.name);
            md.push_str("`): ");
            md.push_str(&param.desc);
            md.push('\n');
            return
        }

        md.push_str("- Parameters:\n");
        for param in params {
            md.push_str("  - ");
            md.push_str(&param.type_name);
            md.push_str(" (`");
            md.push_str(&param.name);
            md.push_str("`): ");
            md.push_str(&param.desc);
            md.push('\n');
        }
    }

    fn add_examples(md: &mut String, examples: &Vec<String>) {
        md.push_str(if examples.len() > 1 { "- Examples:\n" } else { "- Example:\n" });
        for example in examples {
            md.push_str("```kt\n");
            md.push_str(&example.replace("\t", "    "));

            while md.ends_with("\n") {
                md.remove(md.len() - 1);
            }

            md.push_str("\n```\n");
        }
    }

    fn add_from_string_array(md: &mut String, array: &Vec<String>) {
        for value in array {
            md.push_str(value);
            md.push('\n');
        }
    }
}