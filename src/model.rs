//! Who is online, by rank.
use std::{cmp::Reverse, collections::BTreeMap};

/// The most players named in one answer: one question to the rank script. The rest are counted.
pub const MAX_LISTED: usize = 128;

/// What the list needs of a player's rank. (The rank script's own type stays in the plugin half: this half
/// is also built for the host's tests, where nothing of the server's interface may be linked.)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rank {
    pub group: String,
    pub weight: i32,
    pub prefix: String,
}

/// The heading of one line of names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Label {
    /// A rank, by its prefix (or its group name when it has no prefix).
    Rank(String),
    /// Everybody without a rank; the script's own word, in the reader's language.
    Players,
}

/// A rank's heading. A word from the rank script can bring no colour code into the line.
fn label(rank: &Rank) -> Label {
    let clean = |word: &str| -> String {
        word.chars()
            .filter(|c| !matches!(c, '&' | '\u{a7}') && !c.is_control())
            .take(32)
            .collect::<String>()
            .trim()
            .to_owned()
    };
    let prefix = clean(&rank.prefix);
    if !prefix.is_empty() {
        return Label::Rank(prefix);
    }
    let group = clean(&rank.group);
    if group.is_empty() || group == "default" {
        Label::Players
    } else {
        Label::Rank(group)
    }
}

/// The lines of names: the heaviest rank first, players without a rank last, names in the order of the
/// alphabet whatever their letter case.
pub fn lines(players: Vec<(String, Rank)>) -> Vec<(Label, Vec<String>)> {
    let mut groups: BTreeMap<(Reverse<i32>, Label), Vec<String>> = BTreeMap::new();
    for (name, rank) in players {
        let label = label(&rank);
        // Players without a rank always come last, whatever weight the default group carries.
        let weight = match label {
            Label::Players => i32::MIN,
            Label::Rank(_) => rank.weight,
        };
        groups
            .entry((Reverse(weight), label))
            .or_default()
            .push(name);
    }
    groups
        .into_iter()
        .map(|((_, label), mut names)| {
            names.sort_by_key(|name| name.to_lowercase());
            (label, names)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rank(group: &str, weight: i32, prefix: &str) -> Rank {
        Rank {
            group: group.into(),
            weight,
            prefix: prefix.into(),
        }
    }

    #[test]
    fn the_heaviest_rank_is_first_and_players_without_a_rank_are_last() {
        let listed = lines(vec![
            ("zoe".into(), Rank::default()),
            ("Bo".into(), rank("vip", 50, "VIP")),
            ("alex".into(), rank("admin", 100, "ADMIN")),
            ("Adam".into(), Rank::default()),
            ("Steve".into(), rank("admin", 100, "ADMIN")),
        ]);
        assert_eq!(
            listed,
            vec![
                (
                    Label::Rank("ADMIN".into()),
                    vec!["alex".to_owned(), "Steve".to_owned()]
                ),
                (Label::Rank("VIP".into()), vec!["Bo".to_owned()]),
                (Label::Players, vec!["Adam".to_owned(), "zoe".to_owned()]),
            ]
        );
    }

    #[test]
    fn without_a_rank_script_everybody_is_one_line() {
        let listed = lines(vec![
            ("b".into(), Rank::default()),
            ("a".into(), Rank::default()),
        ]);
        assert_eq!(
            listed,
            vec![(Label::Players, vec!["a".to_owned(), "b".to_owned()])]
        );
        assert!(lines(vec![]).is_empty());
    }

    #[test]
    fn a_rank_is_named_by_its_prefix_else_its_group_and_brings_no_colour_code() {
        assert_eq!(label(&rank("mod", 10, "")), Label::Rank("mod".into()));
        assert_eq!(
            label(&rank("mod", 10, "&c\u{a7}lMOD ")),
            Label::Rank("clMOD".into())
        );
        assert_eq!(label(&rank("default", 0, "")), Label::Players);
        assert_eq!(label(&rank("", 5, "&")), Label::Players);
        // A heavy default group still reads as "players", last.
        let listed = lines(vec![
            ("a".into(), rank("default", 900, "")),
            ("b".into(), rank("vip", 50, "VIP")),
        ]);
        assert_eq!(listed[0].0, Label::Rank("VIP".into()));
        assert_eq!(listed[1].0, Label::Players);
    }
}
