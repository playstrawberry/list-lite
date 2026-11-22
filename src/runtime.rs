use crate::{
    lang,
    model::{self, Label, MAX_LISTED, Rank},
};
use std::collections::HashMap;
use strawberry_plugin_api::{
    Context, Plugin, PluginMetadata, Result, Server,
    command::{Command, CommandError, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    common::NamedColor,
    permission::{Permission, PermissionDefault},
    register_plugin,
    text::TextComponent,
};

const USE: &str = "list-lite:use";
// Everybody named fits into one question to the rank script.
const _: () = assert!(MAX_LISTED <= sp_ranks::MAX_UUIDS);

struct Run;

impl CommandHandler for Run {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> std::result::Result<i32, CommandError> {
        let viewer = sender.as_player();
        let locale = match &viewer {
            Some(player) => player.get_locale(),
            None => format!("{:?}", sender.get_locale()),
        };
        let text = |key: &str, args: &[&str]| TextComponent::text(&lang::text(&locale, key, args));
        if viewer.as_ref().is_some_and(|p| !p.has_permission(USE)) {
            return Err(CommandError::CommandFailed(
                text("no_permission", &[]).color_named(NamedColor::Red),
            ));
        }
        let own = viewer.as_ref().map(|player| player.get_id());
        // A player the viewer cannot see (hidden by a staff script) is neither named nor counted. The
        // console sees everybody.
        let mut visible: Vec<(String, String)> = Vec::new();
        let mut count = 0usize;
        for other in server.get_all_players() {
            let id = other.get_id();
            let name = other.get_name();
            let seen = match (&viewer, &own) {
                (Some(viewer), Some(own)) => {
                    (own.high == id.high && own.low == id.low) || viewer.can_see_player(other)
                }
                _ => true,
            };
            if seen {
                count += 1;
                if visible.len() < MAX_LISTED {
                    visible.push((id.to_string(), name));
                }
            }
        }
        // One question to the rank script for everybody named; no answer means no ranks.
        let uuids: Vec<String> = visible.iter().map(|(uuid, _)| uuid.clone()).collect();
        let mut ranks: HashMap<String, Rank> = if uuids.is_empty() {
            HashMap::new()
        } else {
            sp_ranks::get(&uuids)
                .map(|answer| {
                    answer
                        .ranks
                        .into_iter()
                        .map(|(uuid, rank)| {
                            let rank = Rank {
                                group: rank.group,
                                weight: rank.weight,
                                prefix: rank.prefix,
                            };
                            (uuid, rank)
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        let more = count - visible.len();
        let players = visible
            .into_iter()
            .map(|(uuid, name)| (name, ranks.remove(&uuid).unwrap_or_default()))
            .collect();
        sender.send_message(
            text(
                "online",
                &[&count.to_string(), &server.get_max_players().to_string()],
            )
            .color_named(NamedColor::Gold),
        );
        for (label, names) in model::lines(players) {
            let label = match label {
                Label::Rank(label) => label,
                Label::Players => lang::text(&locale, "players", &[]),
            };
            sender.send_message(text("group", &[&label, &names.join(", ")]));
        }
        if more > 0 {
            sender.send_message(text("more", &[&more.to_string()]).color_named(NamedColor::Gray));
        }
        Ok(i32::try_from(count).unwrap_or(i32::MAX))
    }
}

struct ListLite;

impl Plugin for ListLite {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "list-lite".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["Strawberry Interactive LLC".into()],
            description: lang::text("en", "description", &[]),
            dependencies: vec![],
            permissions: vec![],
        }
    }

    fn on_load(&self, context: Context) -> Result<()> {
        context.register_permission(&Permission {
            node: USE.into(),
            description: lang::text("en", "permission_use", &[]),
            default: PermissionDefault::Allow,
            children: vec![],
        })?;
        context.register_command(
            Command::new(&["list".into()], &lang::text("en", "command_list", &[])).execute(Run),
            USE,
        );
        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<()> {
        Ok(())
    }
}

register_plugin!(ListLite);
