//! Topic prerequisite graph for the edureel learning engine.
//!
//! A thin wrapper over estate `graphkit::LinkGraph` using the predicate
//! `prerequisite-of`: an edge `prereq --prerequisite-of--> topic` means
//! `prereq` must be mastered before `topic` unlocks. Supports the
//! CAPABILITY surface `crates/topics`:
//!
//! - [`Topics::unlock`] — whether all of a topic's prerequisites are in a
//!   mastered set (topic state: locked → unlocked).
//! - [`Topics::frontier`] — sorted list of unlocked-but-unmastered topics
//!   (the session's next-new queue; INV-6 order is due > frontier-new).
//! - [`load_topics_json`] — deserialize the versioned `content/topics.json`
//!   seed shape: `{"topics": [{"id", "title", "prerequisites", "track"}]}`.

use std::collections::{BTreeMap, HashSet};
use std::fmt;

use graphkit::{LinkGraph, NeighborView};
use serde::{Deserialize, Serialize};

/// Predicate slug used for every prerequisite edge in the wrapped graph.
pub const PREREQUISITE_OF: &str = "prerequisite-of";

/// One seeded topic from `content/topics.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicSeed {
    /// Stable topic id (also the graph node id).
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Ids of topics that must be mastered first. Empty when the topic has
    /// no prerequisites.
    #[serde(default)]
    pub prerequisites: Vec<String>,
    /// Track this topic belongs to (v0: only `cs-fundamentals`).
    pub track: String,
}

/// File-level shape of `content/topics.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TopicsFile {
    topics: Vec<TopicSeed>,
}

/// Everything that can go wrong while loading topic seeds.
#[derive(Debug)]
pub enum TopicsError {
    /// The JSON did not match the documented shape.
    Json(serde_json::Error),
    /// A topic entry had an empty `id`.
    EmptyTopicId {
        /// Zero-based index of the offending entry.
        index: usize,
    },
    /// The same topic id appeared more than once.
    DuplicateTopic {
        /// The duplicated id.
        id: String,
    },
    /// A topic lists itself as a prerequisite.
    SelfPrerequisite {
        /// The offending topic id.
        id: String,
    },
    /// A prerequisite id has no topic entry.
    UnknownPrerequisite {
        /// The topic declaring the prerequisite.
        id: String,
        /// The missing prerequisite id.
        missing: String,
    },
}

impl fmt::Display for TopicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(f, "invalid topics JSON: {error}"),
            Self::EmptyTopicId { index } => {
                write!(f, "topic at index {index} has an empty id")
            }
            Self::DuplicateTopic { id } => write!(f, "duplicate topic id: {id}"),
            Self::SelfPrerequisite { id } => {
                write!(f, "topic {id} lists itself as a prerequisite")
            }
            Self::UnknownPrerequisite { id, missing } => {
                write!(f, "topic {id} references unknown prerequisite: {missing}")
            }
        }
    }
}

impl std::error::Error for TopicsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

/// Topic metadata plus the underlying prerequisite graph.
#[derive(Debug, Default)]
pub struct Topics {
    graph: LinkGraph,
    meta: BTreeMap<String, TopicSeed>,
}

impl Topics {
    /// Empty topic set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a topic and one `prerequisite-of` edge per prerequisite.
    /// Prerequisite ids may reference topics that are not registered yet;
    /// [`load_topics_json`] validates the full seed set before use.
    pub fn add_topic(&mut self, seed: TopicSeed) {
        for prerequisite in &seed.prerequisites {
            self.graph
                .add_link(prerequisite, &seed.id, PREREQUISITE_OF, 0);
        }
        self.meta.insert(seed.id.clone(), seed);
    }

    /// Sorted prerequisite ids of `topic`; empty when the topic is unknown
    /// or has none.
    #[must_use]
    pub fn prerequisites_of(&self, topic: &str) -> Vec<String> {
        self.meta
            .get(topic)
            .map(|seed| {
                let mut ids = seed.prerequisites.clone();
                ids.sort_unstable();
                ids
            })
            .unwrap_or_default()
    }

    /// Whether `topic` exists and every prerequisite is in `mastered`.
    ///
    /// Prerequisite-only check per CAPABILITY: a topic that is itself
    /// already mastered still answers `true` (it passed the unlocked stage
    /// earlier). Unknown topics answer `false`.
    #[must_use]
    pub fn unlock(&self, topic: &str, mastered: &HashSet<String>) -> bool {
        self.meta
            .get(topic)
            .is_some_and(|seed| seed.prerequisites.iter().all(|p| mastered.contains(p)))
    }

    /// Sorted unlocked-but-unmastered topic ids: the deterministic
    /// frontier of what the learner may pick up next.
    #[must_use]
    pub fn frontier(&self, mastered: &HashSet<String>) -> Vec<String> {
        self.meta
            .keys()
            .filter(|id| !mastered.contains(*id) && self.unlock(id, mastered))
            .cloned()
            .collect()
    }

    /// n-hop neighborhood reachable from `topic` through the underlying
    /// graph (graphkit BFS semantics: both edge directions, shallowest
    /// path wins), keyed by node id.
    #[must_use]
    pub fn reachable_within(&self, topic: &str, hops: usize) -> BTreeMap<String, NeighborView> {
        self.graph.hop_neighbors(topic, hops)
    }

    /// Number of registered topics.
    #[must_use]
    pub fn len(&self) -> usize {
        self.meta.len()
    }

    /// Whether no topics are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.meta.is_empty()
    }

    /// Underlying graph node count (== [`Self::len`] once loaded).
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Underlying graph edge count (one merged edge per
    /// (prerequisite, topic) pair, graphkit semantics).
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}

impl Serialize for Topics {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TopicsFile {
            topics: self.meta.values().cloned().collect(),
        }
        .serialize(serializer)
    }
}

/// Parses `content/topics.json` text into a validated [`Topics`] graph.
///
/// # Errors
///
/// Returns [`TopicsError`] when the JSON shape is invalid, an id is empty
/// or duplicated, a topic lists itself as a prerequisite, or a
/// prerequisite references a topic absent from the file.
pub fn load_topics_json(json: &str) -> Result<Topics, TopicsError> {
    let file: TopicsFile = serde_json::from_str(json).map_err(TopicsError::Json)?;

    let mut seen = HashSet::new();
    for (index, seed) in file.topics.iter().enumerate() {
        if seed.id.is_empty() {
            return Err(TopicsError::EmptyTopicId { index });
        }
        if !seen.insert(seed.id.clone()) {
            return Err(TopicsError::DuplicateTopic {
                id: seed.id.clone(),
            });
        }
    }
    for seed in &file.topics {
        for prerequisite in &seed.prerequisites {
            if prerequisite == &seed.id {
                return Err(TopicsError::SelfPrerequisite {
                    id: seed.id.clone(),
                });
            }
            if !seen.contains(prerequisite) {
                return Err(TopicsError::UnknownPrerequisite {
                    id: seed.id.clone(),
                    missing: prerequisite.clone(),
                });
            }
        }
    }

    let mut topics = Topics::new();
    for seed in file.topics {
        topics.add_topic(seed);
    }
    Ok(topics)
}

#[cfg(test)]
mod tests {
    use super::{load_topics_json, TopicSeed, Topics, PREREQUISITE_OF};
    use std::collections::HashSet;

    fn mastered_of(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(std::string::ToString::to_string).collect()
    }

    fn json_with(topic_json: &[&str]) -> String {
        format!("{{\"topics\":[{}]}}", topic_json.join(","))
    }

    fn seed_json(id: &str, title: &str, prereqs: &[&str], track: &str) -> String {
        let prereq_list = prereqs
            .iter()
            .map(|p| format!("\"{p}\""))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"id\":\"{id}\",\"title\":\"{title}\",\
             \"prerequisites\":[{prereq_list}],\"track\":\"{track}\"}}"
        )
    }

    /// variables-types --prerequisite-of--> big-o --prerequisite-of--> sorting
    fn chain_topics() -> Result<Topics, Box<dyn std::error::Error>> {
        let json = json_with(&[
            &seed_json(
                "variables-types",
                "Variables & Types",
                &[],
                "cs-fundamentals",
            ),
            &seed_json(
                "big-o",
                "Big-O notation",
                &["variables-types"],
                "cs-fundamentals",
            ),
            &seed_json(
                "sorting",
                "Sorting algorithms",
                &["big-o"],
                "cs-fundamentals",
            ),
        ]);
        Ok(load_topics_json(&json)?)
    }

    #[test]
    fn unlock_true_when_all_prerequisites_mastered() -> Result<(), Box<dyn std::error::Error>> {
        let topics = chain_topics()?;
        assert!(topics.unlock("big-o", &mastered_of(&["variables-types"])));
        // No-prerequisite topics are always unlocked.
        assert!(topics.unlock("variables-types", &mastered_of(&[])));
        Ok(())
    }

    #[test]
    fn unlock_false_when_one_prerequisite_missing() -> Result<(), Box<dyn std::error::Error>> {
        let topics = chain_topics()?;
        assert!(!topics.unlock("big-o", &mastered_of(&[])));
        // Partial mastery along the chain leaves downstream topics locked.
        assert!(!topics.unlock("sorting", &mastered_of(&["variables-types"])));
        Ok(())
    }

    #[test]
    fn unlock_false_for_unknown_topic() -> Result<(), Box<dyn std::error::Error>> {
        let topics = chain_topics()?;
        assert!(!topics.unlock("networking", &mastered_of(&["variables-types"])));
        assert!(!topics.unlock("networking", &mastered_of(&[])));
        Ok(())
    }

    #[test]
    fn frontier_returns_sorted_unlocked_unmastered() -> Result<(), Box<dyn std::error::Error>> {
        let json = json_with(&[
            &seed_json("zeta", "Zeta", &[], "cs-fundamentals"),
            &seed_json("alpha", "Alpha", &[], "cs-fundamentals"),
            &seed_json("beta", "Beta", &["alpha"], "cs-fundamentals"),
            &seed_json("gamma", "Gamma", &["beta"], "cs-fundamentals"),
        ]);
        let topics = load_topics_json(&json)?;

        // Nothing mastered: only the two roots are on the frontier, sorted.
        assert_eq!(topics.frontier(&mastered_of(&[])), vec!["alpha", "zeta"]);
        // alpha mastered: beta unlocks and sorts ahead of zeta.
        assert_eq!(
            topics.frontier(&mastered_of(&["alpha"])),
            vec!["beta", "zeta"]
        );
        // Everything mastered: empty frontier.
        assert_eq!(
            topics.frontier(&mastered_of(&["alpha", "beta", "gamma", "zeta"])),
            Vec::<String>::new()
        );
        Ok(())
    }

    #[test]
    fn empty_graph_unlock_false_and_frontier_empty() {
        let topics = Topics::new();
        assert!(topics.is_empty());
        assert_eq!(topics.node_count(), 0);
        assert_eq!(topics.edge_count(), 0);
        assert!(!topics.unlock("anything", &mastered_of(&["anything"])));
        assert_eq!(topics.frontier(&mastered_of(&[])), Vec::<String>::new());
        assert!(topics.reachable_within("anything", 2).is_empty());
    }

    #[test]
    fn json_round_trip_preserves_topics() -> Result<(), Box<dyn std::error::Error>> {
        let topics = chain_topics()?;
        let json = serde_json::to_string(&topics)?;
        let round = load_topics_json(&json)?;

        assert_eq!(round.len(), topics.len());
        assert_eq!(round.node_count(), topics.node_count());
        assert_eq!(round.edge_count(), topics.edge_count());
        assert_eq!(round.prerequisites_of("big-o"), vec!["variables-types"]);
        // Meta fields survive the trip.
        assert!(json.contains("\"title\":\"Big-O notation\""));
        assert!(json.contains("\"track\":\"cs-fundamentals\""));
        Ok(())
    }

    #[test]
    fn reachable_within_reports_hop_depths() -> Result<(), Box<dyn std::error::Error>> {
        let topics = chain_topics()?;
        let reachable = topics.reachable_within("variables-types", 2);
        let views: Vec<_> = reachable.values().collect();
        assert_eq!(views.len(), 2);
        assert!(views
            .iter()
            .any(|v| v.id == "big-o" && v.depth == 1 && v.predicate == PREREQUISITE_OF));
        assert!(views
            .iter()
            .any(|v| v.id == "sorting" && v.depth == 2 && v.via == "big-o"));
        // Depth 1 only reaches big-o.
        assert_eq!(topics.reachable_within("variables-types", 1).len(), 1);
        Ok(())
    }

    #[test]
    fn load_rejects_invalid_seeds() {
        // Unknown prerequisite.
        let json = json_with(&[&seed_json(
            "big-o",
            "Big-O",
            &["variables-types"],
            "cs-fundamentals",
        )]);
        assert!(matches!(
            load_topics_json(&json),
            Err(super::TopicsError::UnknownPrerequisite { .. })
        ));

        // Duplicate id.
        let json = json_with(&[
            &seed_json("big-o", "Big-O", &[], "cs-fundamentals"),
            &seed_json("big-o", "Big-O again", &[], "cs-fundamentals"),
        ]);
        assert!(matches!(
            load_topics_json(&json),
            Err(super::TopicsError::DuplicateTopic { .. })
        ));

        // Self prerequisite.
        let json = json_with(&[&seed_json("big-o", "Big-O", &["big-o"], "cs-fundamentals")]);
        assert!(matches!(
            load_topics_json(&json),
            Err(super::TopicsError::SelfPrerequisite { .. })
        ));

        // Empty id.
        let json = json_with(&[&seed_json("", "No id", &[], "cs-fundamentals")]);
        assert!(matches!(
            load_topics_json(&json),
            Err(super::TopicsError::EmptyTopicId { .. })
        ));

        // Shape mismatch.
        assert!(matches!(
            load_topics_json("{\"topics\": 42}"),
            Err(super::TopicsError::Json(_))
        ));
    }

    #[test]
    fn load_accepts_empty_topic_list_and_missing_prereq_field(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let topics = load_topics_json("{\"topics\": []}")?;
        assert!(topics.is_empty());
        assert_eq!(topics.frontier(&mastered_of(&[])), Vec::<String>::new());

        // `prerequisites` defaults to empty when absent.
        let json =
            json_with(&["{\"id\":\"solo\",\"title\":\"Solo\",\"track\":\"cs-fundamentals\"}"]);
        let topics = load_topics_json(&json)?;
        assert!(topics.prerequisites_of("solo").is_empty());
        assert!(topics.unlock("solo", &mastered_of(&[])));
        assert_eq!(topics.frontier(&mastered_of(&[])), vec!["solo"]);
        Ok(())
    }

    #[test]
    fn add_topic_registers_edges_without_validation() {
        let mut topics = Topics::new();
        topics.add_topic(TopicSeed {
            id: "big-o".to_string(),
            title: "Big-O".to_string(),
            prerequisites: vec!["variables-types".to_string()],
            track: "cs-fundamentals".to_string(),
        });
        // graphkit auto-creates the dangling prerequisite node.
        assert_eq!(topics.node_count(), 2);
        assert_eq!(topics.edge_count(), 1);
        assert_eq!(topics.prerequisites_of("big-o"), vec!["variables-types"]);
        // Frontiers only ever surface registered (meta) topics.
        assert_eq!(
            topics.frontier(&mastered_of(&["variables-types"])),
            vec!["big-o"]
        );
    }
}
