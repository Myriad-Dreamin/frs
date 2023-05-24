use std::{
    collections::HashMap,
    fmt::{Display, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetadataStepLog {
    pub description: String,
    pub prompt: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub namespace: String,
    pub name: String,
    pub is_dirty: bool,
    pub step_log: Vec<MetadataStepLog>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Context {
    pub meta: Metadata,
    pub env: HashMap<String, String>,
    pub template: String,
}

const STRING: ansi_term::Color = ansi_term::Color::RGB(0x9e, 0xce, 0x6a);
const KEYWORD: ansi_term::Color = ansi_term::Color::RGB(0xbb, 0x9a, 0xf7);
const FUNCTION: ansi_term::Color = ansi_term::Color::RGB(0x7a, 0xa2, 0xf7);

impl Context {
    pub fn to_shell(&self) -> String {
        let mut shell = String::new();

        shell.push_str(&format!("{}\n", self.template));

        for step in self.meta.step_log.iter() {
            if let Some(prompt) = &step.prompt {
                shell.push_str(&format!("# $ {}\n", prompt));
            }
            shell.push_str(&format!("# ! {}\n", step.description));
        }
        // self json
        shell.push_str(&format!(
            "# FRS_META={:?}\n",
            serde_json::to_string(&self).unwrap()
        ));

        shell
    }

    pub fn pretty_print(&self) -> PrettyContext {
        PrettyContext(self)
    }

    pub fn pretty_prompt(&self) -> PrettyPrompt {
        PrettyPrompt(self)
    }
}

fn painted(f: &mut std::fmt::Formatter<'_>, c: ansi_term::Color, s: String) {
    f.write_fmt(format_args!("{}", c.paint(s))).unwrap();
}

fn sanitize_string(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() || *c == ' ')
        .collect()
}

fn sanitize_and_painted(f: &mut std::fmt::Formatter<'_>, c: ansi_term::Color, s: &str) {
    // contains printable
    if s.chars().any(|c| c.is_whitespace() && c != ' ') {
        painted(f, c, sanitize_string(s));
    } else {
        painted(f, c, s.to_owned());
    }
}

pub struct PrettyContext<'a>(&'a Context);

impl<'a> Display for PrettyContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.meta.namespace == "default" {
            painted(f, STRING, format!("# name: {}\n", self.0.meta.name));
        } else {
            painted(
                f,
                STRING,
                format!("# name: {}::{}\n", self.0.meta.namespace, self.0.meta.name),
            );
        }
        for step_log in self.0.meta.step_log.iter() {
            if let Some(prompt) = &step_log.prompt {
                painted(f, KEYWORD, format!("# $ {}\n", prompt));
            }
            painted(f, KEYWORD, format!("# ! {}\n", step_log.description));
        }

        for (key, value) in self.0.env.iter() {
            painted(f, STRING, format!("# frs_env: {}={}\n", key, value));
        }
        painted(f, FUNCTION, format!("{}\n", self.0.template));

        Ok(())
    }
}

pub struct PrettyPrompt<'a>(&'a Context);

impl<'a> Display for PrettyPrompt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('(')?;
        if self.0.meta.namespace == "default" {
            painted(f, STRING, format!("{}", self.0.meta.name));
        } else {
            painted(
                f,
                STRING,
                format!("{}::{}", self.0.meta.namespace, self.0.meta.name),
            );
        }
        f.write_char(')')?;
        if !self.0.meta.is_dirty {
            return Ok(());
        }

        for step_log in self.0.meta.step_log.iter() {
            if let Some(prompt) = &step_log.prompt {
                f.write_char(' ')?;
                sanitize_and_painted(f, KEYWORD, prompt);
            }
        }
        Ok(())
    }
}

pub const TEMPLATE_PLACEHOLDER: &str = "(((( echo 'frs placeholder' ))))";

pub fn get_saved_context_path(namespace: &str, name: &str) -> String {
    // $HOME/.config/frs/context/namespace/name
    let home = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE").unwrap()
    } else {
        std::env::var("HOME").unwrap()
    };
    format!(
        "{}/.config/frs/context/{}/{}.json",
        home,
        namespace.replace(['/', '\\'], "·"),
        name.replace(['/', '\\'], "·")
    )
}

pub fn load_context(namespace: &str, name: &str) -> Context {
    let fi = get_saved_context_path(namespace, name);

    let data = std::fs::read(fi)
        .map_err(|err| {
            eprintln!("load context failed {}::{}: {}", namespace, name, err);
            std::process::exit(1);
        })
        .unwrap();
    serde_json::from_slice(data.as_slice()).unwrap()
}

pub fn save_context(context: &Context) {
    let fi = get_saved_context_path(&context.meta.namespace, &context.meta.name);
    let dir = Path::new(&fi).parent().unwrap();
    std::fs::create_dir_all(dir).unwrap();

    let data = serde_json::to_vec(context).unwrap();
    std::fs::write(fi, data).unwrap();
}

pub mod builtin {
    use std::path::Path;

    use crate::{load_context, Context, MetadataStepLog};

    fn path_last(path: &Path) -> &str {
        path.file_name().unwrap().to_str().unwrap()
    }

    pub fn with_workdir(mut context: Context, workdir: String) -> Context {
        context.meta.is_dirty = true;
        context.meta.step_log.push(MetadataStepLog {
            description: format!("core::with_workdir {:?}", workdir),
            prompt: Some(format!("wd(..{})", path_last(Path::new(&workdir)))),
        });
        context.template = context.template.replace(
            crate::TEMPLATE_PLACEHOLDER,
            &format!("(cd {};\n {})", workdir, crate::TEMPLATE_PLACEHOLDER),
        );
        context
    }

    pub fn with_path(mut context: Context, path: String) -> Context {
        context.meta.is_dirty = true;
        context.meta.step_log.push(MetadataStepLog {
            description: format!("core::with_path {:?}", path),
            prompt: Some({
                let path_ref = Path::new(&path);
                let file_name = path_last(path_ref);
                if file_name == "bin" {
                    format!(
                        "toolchain({})",
                        path_ref.parent().map(path_last).unwrap_or_else(|| "bin")
                    )
                } else {
                    format!("path({})", file_name.to_string())
                }
            }),
        });
        context.template = context.template.replace(
            crate::TEMPLATE_PLACEHOLDER,
            &format!(
                "(export PATH=${{PATH}}:{};\n {})",
                path,
                crate::TEMPLATE_PLACEHOLDER
            ),
        );
        context
    }

    pub fn with_env(mut context: Context, key: String, value: String) -> Context {
        context.meta.is_dirty = true;
        context.meta.step_log.push(MetadataStepLog {
            description: format!("core::with_env {:?}={:?}", key, value),
            prompt: Some(format!("env({})", key)),
        });
        context.template = context.template.replace(
            crate::TEMPLATE_PLACEHOLDER,
            &format!(
                "(export {}={};\n {})",
                key,
                value,
                crate::TEMPLATE_PLACEHOLDER
            ),
        );

        context
    }

    pub fn with_command(mut context: Context, cmd: String) -> Context {
        let cmd_first = cmd.split_whitespace().next().unwrap_or("");
        context.meta.is_dirty = true;
        context.meta.step_log.push(MetadataStepLog {
            description: format!("core::with_command {:?}", cmd),
            prompt: Some(format!("exec({})", cmd_first)),
        });
        context.template = context.template.replace(
            crate::TEMPLATE_PLACEHOLDER,
            &format!("({};\n {})", cmd, crate::TEMPLATE_PLACEHOLDER),
        );
        context
    }

    pub fn with_docker(mut context: Context, container: String) -> Context {
        context.meta.is_dirty = true;
        context.meta.step_log.push(MetadataStepLog {
            description: format!("core::with_docker {:?}", container),
            prompt: Some(format!("ctr({:?})", container)),
        });
        context.template = context.template.replace(
            crate::TEMPLATE_PLACEHOLDER,
            &format!("(docker run {} {})", container, crate::TEMPLATE_PLACEHOLDER),
        );
        context
    }

    pub fn activate_context(_context: Context, namespace: &str, name: &str) -> Context {
        load_context(namespace, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        let mut context = Context {
            meta: Metadata {
                namespace: "default".to_owned(),
                name: "".to_owned(),
                is_dirty: false,
                step_log: vec![],
            },
            env: HashMap::new(),
            template: String::from("echo hello"),
        };

        let mut manipulations: Vec<Box<dyn FnOnce(Context) -> Context>> = vec![];

        manipulations.push(Box::new(|context| {
            builtin::with_workdir(context, String::from("/tmp"))
        }));
        manipulations.push(Box::new(|context| {
            builtin::with_path(context, String::from("/usr/bin"))
        }));
        manipulations.push(Box::new(|context| {
            builtin::with_env(context, String::from("FOO"), String::from("BAR"))
        }));
        manipulations.push(Box::new(|context| {
            builtin::with_command(context, String::from("echo $PWD"))
        }));

        for manipulation in manipulations {
            context = manipulation(context);
        }

        println!("{:?}", context);
        println!("{}", context.to_shell());
    }
}
