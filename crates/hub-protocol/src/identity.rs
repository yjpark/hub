//! Node and link identity — how the mesh is addressed.
//!
//! Uniform addressing (§2.4): a node is reached by its **name** in the hub's
//! registry, never a local endpoint. One machine is one *node*; a node may offer
//! several *links* (§2.5) — a default `stable` link and, say, a session-scoped
//! `dev` link — told apart by a [`LinkLabel`], not by listen address.

// implements: bf406db5b660bd595cccdfcec5921121bb5a1ac9281a0d9a5c28be2d7ccc79b4@bf406db5b660bd595cccdfcec5921121bb5a1ac9281a0d9a5c28be2d7ccc79b4
// implements: 08c3ce357742ccf578caddcfb6578a92216b515adcfe1ac9de4543406be52cb0@08c3ce357742ccf578caddcfb6578a92216b515adcfe1ac9de4543406be52cb0

use facet::Facet;

/// A node's stable identity: its name in the hub registry (§2.4).
///
/// This is the *only* way a node is addressed. It is stable across a node's
/// links and across daemon restarts.
#[derive(Facet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub String);

/// Between a peer hub's name and one of its nodes: `edger::yolo`. Reserved —
/// a node may not register a name containing it.
// implements: a297d880b240fa86f9872753ebc3009c0d3af0dd572ec9eaa2a17a3f815f8d45@6a55d418a00ed4bd5b3644ffa295bc579963314652f4b658ef1e40e185002757
pub const PEER_SEPARATOR: &str = "::";

impl NodeId {
    /// A peer hub's node, as this hub's consumers see it: `edger::yolo`.
    pub fn qualified(hub: &str, node: &NodeId) -> Self {
        Self(format!("{hub}{PEER_SEPARATOR}{}", node.0))
    }

    /// The peer hub and the node within it, for a qualified id; `None` for a
    /// bare (local) one.
    pub fn split_peer(&self) -> Option<(&str, NodeId)> {
        self.0
            .split_once(PEER_SEPARATOR)
            .map(|(hub, node)| (hub, NodeId(node.to_string())))
    }
}

/// Which link of a node a request concerns (§2.5).
///
/// `stable` is distinguished: it is the routing default and the fallback when a
/// requested link is absent. Other labels (e.g. `dev`) are conventions carried
/// as-is.
#[derive(Facet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinkLabel(pub String);

impl LinkLabel {
    /// The distinguished default/fallback link.
    pub const STABLE: &'static str = "stable";
    /// The conventional development link (its own toolchain environment).
    pub const DEV: &'static str = "dev";

    /// The `stable` link — the routing default.
    pub fn stable() -> Self {
        Self(Self::STABLE.to_string())
    }

    /// The `dev` link.
    pub fn dev() -> Self {
        Self(Self::DEV.to_string())
    }

    /// Whether this is the distinguished `stable` link.
    pub fn is_stable(&self) -> bool {
        self.0 == Self::STABLE
    }
}

impl Default for LinkLabel {
    /// Absent an explicit label, routing targets `stable` (§2.5).
    fn default() -> Self {
        Self::stable()
    }
}

/// A fully-qualified target: a specific link of a specific node.
#[derive(Facet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeLink {
    pub node: NodeId,
    pub link: LinkLabel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_is_the_default_link() {
        assert_eq!(LinkLabel::default(), LinkLabel::stable());
        assert!(LinkLabel::default().is_stable());
        assert!(!LinkLabel::dev().is_stable());
    }

    #[test]
    fn node_id_round_trips() {
        let v = NodeId("alpha".to_string());
        let json = facet_json::to_string(&v).unwrap();
        let back: NodeId = facet_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn node_link_round_trips() {
        let v = NodeLink {
            node: NodeId("alpha".to_string()),
            link: LinkLabel::dev(),
        };
        let json = facet_json::to_string(&v).unwrap();
        let back: NodeLink = facet_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }
}
