use std::{path::Path, process::exit};

use clap::{Args, Command, FromArgMatches, Parser, Subcommand};
use frs::{builtin, Context, TEMPLATE_PLACEHOLDER};
use once_cell::sync::Lazy;

pub mod build_info {
    /// The version of the frs crate.
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}

#[derive(Debug, Parser)]
#[clap(name = "frs", version = build_info::VERSION)]
pub struct Opts {
    #[clap(subcommand)]
    pub sub: Option<Subcommands>,
}

#[derive(Debug, Subcommand)]
#[clap(
    about = "The cli for frs.",
    after_help = "",
    next_display_order = None
)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommands {
    /// Manipulate context.
    With(WithArgs),

    /// Run with context.
    Run(RunArgs),

    /// Save context.
    Save(SaveArgs),

    /// Inspect context.
    Inspect(InspectArgs),

    /// Get Propmt of context.
    Prompt,
}

#[derive(Debug, Parser)]
pub struct WithArgs {
    #[clap(index = 1, help = "The context builder.")]
    pub builder: String,

    #[clap(index = 2, trailing_var_arg = true, help = "The rest arguments.")]
    pub rest: Vec<String>,
}

impl WithArgs {
    fn rest_as_args(&self) -> Vec<String> {
        let args = vec![std::env::args().next().unwrap(), self.builder.clone()];
        args.into_iter()
            .chain(self.rest.iter().cloned())
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Parser)]
pub struct RunArgs {
    #[clap(long, help = "Using context", default_value = "default")]
    pub context: String,

    #[clap(long, help = "Show executing command", default_value_t = false)]
    pub show: bool,

    #[clap(index = 1, trailing_var_arg = true, help = "The rest arguments.")]
    pub rest: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct SaveArgs {
    #[clap(long, help = "Save into namespace.", default_value = "default")]
    pub namespace: String,
    #[clap(index = 1, help = "Save as name.")]
    pub name: String,
}

#[derive(Debug, Parser)]
pub struct InspectArgs {
    #[clap(
        long,
        help = "Inspect context within namespace.",
        default_value = "default"
    )]
    pub namespace: String,
    #[clap(
        index = 1,
        help = "Inspect by context name.",
        default_value = "default"
    )]
    pub name: String,
}

fn get_cli(sub_command_required: bool) -> Command {
    let cli = Command::new("$").disable_version_flag(true);
    Opts::augment_args(cli).subcommand_required(sub_command_required)
}

fn help_sub_command() {
    Opts::from_arg_matches(&get_cli(true).get_matches()).unwrap();
}

fn main() {
    let opts = Opts::from_arg_matches(&get_cli(false).get_matches())
        .map_err(|err| err.exit())
        .unwrap();

    match opts.sub {
        Some(Subcommands::With(args)) => with_context(args),
        Some(Subcommands::Run(args)) => run_context(args),
        Some(Subcommands::Save(args)) => save_context(args),
        Some(Subcommands::Inspect(args)) => inspect_context(args),
        Some(Subcommands::Prompt) => prompt_context(),

        None => help_sub_command(),
    };

    #[allow(unreachable_code)]
    {
        unreachable!("The subcommand must exit the process.");
    }
}

fn get_context_from_file(fi: &Path) -> std::io::Result<Context> {
    // open
    let data = std::fs::read(fi)?;
    Ok(serde_json::from_slice(data.as_slice()).unwrap())
}

fn get_base_context() -> Context {
    let mut base_state = Context::default();
    base_state.meta.name = "default".to_owned();
    base_state.meta.namespace = "default".to_owned();
    base_state
        .env
        .insert("FRS_VERSION".to_string(), build_info::VERSION.to_owned());
    base_state.template = TEMPLATE_PLACEHOLDER.to_owned();
    base_state
}

static CURRENT_STAT_PATH: Lazy<String> = Lazy::new(|| {
    // get if frs term_pid env is set
    let parent_pid: u32 = if let Ok(pid) = std::env::var("FRS_TERM_PID") {
        pid.parse().unwrap()
    } else if cfg!(all(unix)) {
        std::os::unix::process::parent_id()
    } else {
        unimplemented!()
    };
    let start_time: u64 = if cfg!(all(unix)) {
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", parent_pid)).unwrap();
        let start_time = stat.split(' ').nth(21).unwrap().parse().unwrap();
        start_time
    } else {
        unimplemented!()
    };

    format!("/tmp/{}.{}.json", parent_pid, start_time)
});

/// Get current shell state
fn get_current_shell_context() -> Context {
    let state_file = &*CURRENT_STAT_PATH;
    if let Ok(context) = get_context_from_file(std::path::Path::new(state_file)) {
        return context;
    }

    get_base_context()
}

fn with_context(args: WithArgs) -> ! {
    let context = get_current_shell_context();
    println!("args {:?} context {:?}", args, context);

    let context = match args.builder.as_str() {
        "workdir" => {
            #[derive(Debug, Parser)]
            pub struct WithWorkdirArgs {
                #[clap(index = 1, help = "_")]
                pub self_arg: String,
                #[clap(index = 2, help = "new workdir.")]
                pub workdir: String,
            }

            let opts = WithWorkdirArgs::parse_from(args.rest_as_args());
            builtin::with_workdir(context, opts.workdir)
        }
        "path" => {
            #[derive(Debug, Parser)]
            pub struct WithPathArgs {
                #[clap(index = 1, help = "_")]
                pub self_arg: String,
                #[clap(index = 2, help = "new path.")]
                pub path: String,
            }

            let rest_args = args.rest_as_args();
            println!("path_args {:?}", rest_args);
            let opts = WithPathArgs::parse_from(rest_args);
            builtin::with_path(context, opts.path)
        }
        "env" => {
            #[derive(Debug, Parser)]
            pub struct WithEnvArgs {
                #[clap(index = 1, help = "_")]
                pub self_arg: String,
                #[clap(index = 2, help = "new env key.")]
                pub key: String,
                #[clap(index = 3, help = "new env value.")]
                pub value: String,
            }

            let opts = WithEnvArgs::parse_from(args.rest_as_args());
            builtin::with_env(context, opts.key, opts.value)
        }
        "command" => builtin::with_command(context, args.rest.join(" ")),
        "docker" => {
            #[derive(Debug, Parser)]
            pub struct WithDockerArgs {
                #[clap(index = 1, help = "_")]
                pub self_arg: String,
                #[clap(index = 2, help = "new docker container.")]
                pub container: String,
            }

            let opts = WithDockerArgs::parse_from(args.rest_as_args());
            builtin::with_docker(context, opts.container)
        }
        "context" => {
            #[derive(Debug, Parser)]
            pub struct WithContextArgs {
                #[clap(index = 1, help = "_")]
                pub self_arg: String,
                #[clap(long, help = "new context namespace.", default_value = "default")]
                pub namespace: String,
                #[clap(index = 2, help = "new context name.")]
                pub name: String,
            }

            let opts = WithContextArgs::parse_from(args.rest_as_args());
            builtin::activate_context(context, &opts.namespace, &opts.name)
        }
        "empty" => {
            #[derive(Debug, Parser)]
            pub struct WithContextArgs {
                #[clap(index = 1, help = "_")]
                pub self_arg: String,
            }

            let _opts = WithContextArgs::parse_from(args.rest_as_args());
            get_base_context()
        }
        _ => {
            println!("unknown builder {}", args.builder);
            exit(1);
        }
    };

    let stat_file = &*CURRENT_STAT_PATH;
    let data = serde_json::to_vec(&context).unwrap();
    std::fs::write(stat_file, data).unwrap();

    exit(0)
}

fn run_context(args: RunArgs) -> ! {
    let context = get_current_shell_context();
    let command_str = args.rest.join(" ");
    let _ = args.show;
    let command_str = context.template.replace(TEMPLATE_PLACEHOLDER, &command_str);
    println!("{}", command_str);

    exit(0)
}

fn save_context(args: SaveArgs) -> ! {
    let mut context = get_current_shell_context();
    context.meta.namespace = args.namespace.clone();
    context.meta.name = args.name.clone();
    context.meta.is_dirty = false;

    frs::save_context(&context);

    let stat_file = &*CURRENT_STAT_PATH;
    let data = serde_json::to_vec(&context).unwrap();
    std::fs::write(stat_file, data).unwrap();

    exit(0)
}

fn inspect_context(args: InspectArgs) -> ! {
    let context = match args.name.as_str() {
        "default" => {
            if args.namespace == "default" {
                get_current_shell_context()
            } else {
                panic!("not support namespace {} without name", args.namespace);
            }
        }
        _ => frs::load_context(&args.namespace, &args.name),
    };
    println!("{}", context.pretty_print());

    exit(0)
}

fn prompt_context() -> ! {
    let context = get_current_shell_context();
    print!("{}", context.pretty_prompt());

    exit(0)
}
