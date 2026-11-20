//! Official DE drop tables (`https://www.warframe.com/droptables`): only the `relicRewards`
//! chapter is read, according to `docs/04-REGLES-DROPS.md` (rule 16) and ADR-002.

use super::{DropSource, SourceError};
use crate::model::{DropDataset, Rarity, Refinement, RelicReward};

/// Public address of the DE drop tables; it redirects to a static HTML file.
pub const DE_DROP_TABLES_URL: &str = "https://www.warframe.com/droptables";

/// Label used in errors and warnings.
pub const DE_DROP_TABLES: &str = "tables officielles DE";

const SECTION_START: &str = "id=\"relicRewards\"";
const SECTION_END: &str = "<h3";

/// Raw HTML of the DE drop tables page.
pub struct DeRelicSource {
    html: String,
}

impl DeRelicSource {
    pub fn new(html: impl Into<String>) -> Self {
        Self { html: html.into() }
    }
}

impl DropSource for DeRelicSource {
    /// Yields only `relic_rewards` and the warnings of their reading.
    fn load(&self) -> Result<DropDataset, SourceError> {
        let (relic_rewards, warnings) = parse_relic_rewards(&self.html)?;
        Ok(DropDataset {
            relic_rewards,
            warnings,
            ..DropDataset::default()
        })
    }
}

/// Reads the `relicRewards` chapter. The format is fixed (one `<th>` per relic and
/// refinement, then one `<tr>` per reward), so a dedicated scanner is enough; an unexpected
/// row becomes a warning rather than an error, because the other relics remain usable.
pub fn parse_relic_rewards(html: &str) -> Result<(Vec<RelicReward>, Vec<String>), SourceError> {
    let start = html
        .find(SECTION_START)
        .ok_or_else(|| SourceError::Structure {
            module: DE_DROP_TABLES,
            message: "chapitre « relicRewards » absent".to_owned(),
        })?;
    let section = &html[start + SECTION_START.len()..];
    let section = section
        .find(SECTION_END)
        .map_or(section, |end| &section[..end]);

    let mut rewards = Vec::new();
    let mut warnings = Vec::new();
    // `None` after an unreadable header: its rows are skipped with the header's warning.
    let mut current: Option<(String, Refinement)> = None;
    let mut seen_header = false;
    for row in section.split("<tr").skip(1) {
        if let Some(header) = cell_texts(row, "th").into_iter().next() {
            seen_header = true;
            current = parse_header(&header);
            if current.is_none() {
                warnings.push(format!(
                    "{DE_DROP_TABLES}, relique « {header} » illisible, ignorée"
                ));
            }
            continue;
        }
        let cells = cell_texts(row, "td");
        if cells.iter().all(|cell| cell.is_empty()) {
            // Separator row between relics.
            continue;
        }
        let Some((relic, refinement)) = &current else {
            if !seen_header {
                warnings.push(format!(
                    "{DE_DROP_TABLES}, ligne sans relique ignorée : {}",
                    cells.join(" | ")
                ));
            }
            continue;
        };
        match parse_reward(&cells) {
            Some((item, chance)) => rewards.push(RelicReward {
                relic: relic.clone(),
                refinement: *refinement,
                item,
                chance,
                rarity: Rarity::from_chance(chance),
            }),
            None => warnings.push(format!(
                "{DE_DROP_TABLES}, {relic} ({}), ligne illisible ignorée : {}",
                refinement.code(),
                cells.join(" | ")
            )),
        }
    }
    Ok((rewards, warnings))
}

/// `Axi A1 Relic (Intact)` → (`Axi A1 Relic`, Intact).
fn parse_header(header: &str) -> Option<(String, Refinement)> {
    let (relic, rest) = header.rsplit_once(" (")?;
    let refinement = Refinement::from_code(rest.strip_suffix(')')?)?;
    let relic = relic.trim();
    (!relic.is_empty()).then(|| (relic.to_owned(), refinement))
}

/// `[item, "Uncommon (25.33%)"]` → (item, 0.2533).
fn parse_reward(cells: &[String]) -> Option<(String, f64)> {
    let [item, chance] = cells else {
        return None;
    };
    let percent = chance.rsplit_once('(')?.1.strip_suffix("%)")?;
    let percent: f64 = percent.trim().parse().ok()?;
    if item.is_empty() || !(0.0..=100.0).contains(&percent) {
        return None;
    }
    Some((item.clone(), super::wiki::round_chance(percent / 100.0)))
}

/// Texts of the `<tag>` cells of a row, entities decoded.
fn cell_texts(row: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut texts = Vec::new();
    let mut rest = row;
    while let Some(start) = rest.find(&open) {
        let after = &rest[start + open.len()..];
        let Some(content_start) = after.find('>') else {
            break;
        };
        let content = &after[content_start + 1..];
        let Some(end) = content.find(&close) else {
            break;
        };
        texts.push(decode_entities(content[..end].trim()));
        rest = &content[end + close.len()..];
    }
    texts
}

/// Decodes the few entities DE may write; none appear in the chapter today.
fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_headers() {
        assert_eq!(
            parse_header("Lith Q3 Relic (Radiant)"),
            Some(("Lith Q3 Relic".to_owned(), Refinement::Radiant))
        );
        assert_eq!(parse_header("Lith Q3 Relic"), None);
        assert_eq!(parse_header("Lith Q3 Relic (Shiny)"), None);
    }

    #[test]
    fn reads_rewards_and_entities() {
        let cells = cell_texts(
            "><td>Gyre &amp; Co</td><td>Uncommon (25.33%)</td></tr>",
            "td",
        );
        assert_eq!(parse_reward(&cells), Some(("Gyre & Co".to_owned(), 0.2533)));
        assert_eq!(parse_reward(&["X".to_owned(), "Rare".to_owned()]), None);
        assert_eq!(parse_reward(&["X".to_owned()]), None);
    }

    #[test]
    fn missing_chapter_is_an_error() {
        assert!(matches!(
            parse_relic_rewards("<h3 id=\"missionRewards\">"),
            Err(SourceError::Structure { .. })
        ));
    }
}
