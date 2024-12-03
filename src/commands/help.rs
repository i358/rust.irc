use tabled::{
    Table, Tabled,
    settings::{Style, Alignment, Modify, Width, object::Columns},
};
use crate::commands::{get_commands, Command};

#[derive(Tabled)]
struct CommandInfo {
    #[tabled(rename = "Komut")]
    name: String,
    #[tabled(rename = "Açıklama")]
    description: String,
    #[tabled(rename = "Kullanım")]
    usage: String,
    #[tabled(rename = "Argümanlar")]
    args: String,
}

pub const COMMAND: Command = Command {
    name: "help",
    usage: "/help",
    args: &[""],
    description: "Mevcut komutları listeler.",
    exec:|_, _, _| {
        let commands = get_commands();

        let command_data: Vec<CommandInfo> = commands
            .into_iter()
            .map(|cmd| CommandInfo {
                name: cmd.name.to_string(),
                description: cmd.description.to_string(),
                usage: cmd.usage.to_string(),
                args: cmd.args.join(", ").to_string(),
            })
            .collect();

        let mut table = Table::new(command_data);

        table
            .with(Style::modern())
            .with(Modify::new(Columns::single(0)).with(Width::wrap(20)))
            .with(Modify::new(Columns::single(1)).with(Width::wrap(40)))
            .with(Modify::new(Columns::single(2)).with(Width::wrap(30)))
            .with(Modify::new(Columns::single(3)).with(Width::wrap(40)))
            .with(Modify::new(Columns::new(..)).with(Alignment::left()));

        println!("{}", table);
    },
};
