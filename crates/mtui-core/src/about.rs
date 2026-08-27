//! On-demand screen guides for `RouterOS` menus.
//!
//! Copy is original wording aligned with
//! <https://manual.mikrotik.com/docs/>. Property tables are not reproduced.
//! Every catalog id must have an entry; the CLI reference URL is derived from
//! the resource path so field names stay tied to what `RouterOS` exposes.

use crate::resources::{DASHBOARD_ID, resource_by_id};

const CLI_DOCS: &str = "https://manual.mikrotik.com/docs/cli-reference";
const MANUAL: &str = "https://manual.mikrotik.com/docs";

/// Section heading for the operator-facing “do I need this?” copy.
pub const WHEN_YOU_NEED_IT: &str = "When you need it";

/// Curated explanation for one navigation screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenGuide {
    pub summary: &'static str,
    pub use_when: &'static str,
    pub fields: &'static str,
    /// Conceptual manual page when one exists; CLI reference is always added.
    pub docs_url: Option<&'static str>,
}

/// Title, path kicker, and wrapped body for the about overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutCopy {
    pub title: String,
    pub kicker: String,
    pub body: String,
}

/// Guide for the open screen (`dashboard` or a resource id).
#[must_use]
pub fn screen_guide(id: &str) -> Option<&'static ScreenGuide> {
    crate::features::interfaces::guides::GUIDES
        .iter()
        .chain(crate::features::wireguard::guides::GUIDES)
        .chain(crate::features::ppp::guides::GUIDES)
        .chain(crate::features::bridge::guides::GUIDES)
        .chain(crate::features::switch::guides::GUIDES)
        .chain(crate::features::ip::guides::GUIDES)
        .chain(crate::features::ipv6::guides::GUIDES)
        .chain(crate::features::routing::guides::GUIDES)
        .chain(crate::features::queues::guides::GUIDES)
        .chain(crate::features::files::guides::GUIDES)
        .chain(crate::features::tools::guides::GUIDES)
        .chain(crate::features::radius::guides::GUIDES)
        .chain(crate::features::container::guides::GUIDES)
        .chain(crate::features::system::guides::GUIDES)
        .chain(GUIDES)
        .find(|(key, _)| *key == id)
        .map(|(_, guide)| guide)
}

/// Formatted overlay copy, or `None` when the id is unknown.
#[must_use]
pub fn about_copy(id: &str) -> Option<AboutCopy> {
    let guide = screen_guide(id)?;
    let (title, kicker, cli_url) = if id == DASHBOARD_ID {
        (
            "About Dashboard".to_string(),
            "overview".to_string(),
            format!("{MANUAL}/introduction/"),
        )
    } else {
        let spec = resource_by_id(id)?;
        (
            format!("About {}", spec.label),
            spec.cli_path().to_string(),
            format!("{CLI_DOCS}{}/", spec.cli_path()),
        )
    };

    let mut body = String::new();
    body.push_str(guide.summary);
    body.push_str("\n\n");
    body.push_str(WHEN_YOU_NEED_IT);
    body.push('\n');
    body.push_str(guide.use_when);
    if !guide.fields.is_empty() {
        body.push_str("\n\nNotable fields\n");
        body.push_str(guide.fields);
    }
    body.push_str("\n\nOfficial documentation\n");
    if let Some(url) = guide.docs_url
        && url != cli_url
    {
        body.push_str(url);
        body.push('\n');
    }
    body.push_str(&cli_url);
    Some(AboutCopy {
        title,
        kicker,
        body,
    })
}

macro_rules! guide {
    ($id:literal, $summary:literal, $when:literal, $fields:literal) => {
        (
            $id,
            ScreenGuide {
                summary: $summary,
                use_when: $when,
                fields: $fields,
                docs_url: None,
            },
        )
    };
    ($id:literal, $summary:literal, $when:literal, $fields:literal, $docs:literal) => {
        (
            $id,
            ScreenGuide {
                summary: $summary,
                use_when: $when,
                fields: $fields,
                docs_url: Some($docs),
            },
        )
    };
}

static GUIDES: &[(&str, ScreenGuide)] = &[guide!(
    "dashboard",
    "Live CPU, memory, WAN throughput, and firewall-hit overview for the connected \
         router. This is an mtui dashboard, not a RouterOS menu.",
    "Use it as a first look at whether the device is busy or the WAN is moving traffic. \
         Open a specific menu when you need to change configuration.",
    "Charts are sampled locally. Firewall rows are hit counters from filter rules, not \
         a rule editor.",
    "https://manual.mikrotik.com/docs/introduction/"
)];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::ALL_RESOURCES;

    #[test]
    fn every_resource_and_dashboard_has_a_guide() {
        let mut ids: Vec<&str> = crate::features::interfaces::guides::GUIDES
            .iter()
            .chain(crate::features::wireguard::guides::GUIDES)
            .chain(crate::features::ppp::guides::GUIDES)
            .chain(crate::features::bridge::guides::GUIDES)
            .chain(crate::features::switch::guides::GUIDES)
            .chain(crate::features::ip::guides::GUIDES)
            .chain(crate::features::ipv6::guides::GUIDES)
            .chain(crate::features::routing::guides::GUIDES)
            .chain(crate::features::queues::guides::GUIDES)
            .chain(crate::features::files::guides::GUIDES)
            .chain(crate::features::tools::guides::GUIDES)
            .chain(crate::features::radius::guides::GUIDES)
            .chain(crate::features::container::guides::GUIDES)
            .chain(crate::features::system::guides::GUIDES)
            .chain(GUIDES)
            .map(|(id, _)| *id)
            .collect();
        let original = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), original, "duplicate screen guide ids");
        assert!(screen_guide(DASHBOARD_ID).is_some());
        for spec in ALL_RESOURCES.iter() {
            assert!(
                screen_guide(spec.id).is_some(),
                "missing screen guide for {}",
                spec.id
            );
        }
    }

    #[test]
    fn neighbors_guide_mentions_connect_tab() {
        let guide = screen_guide("neighbors").expect("neighbors");
        assert!(guide.use_when.contains("Connect"), "{}", guide.use_when);
        assert!(guide.use_when.contains("device tab"), "{}", guide.use_when);
        let copy = about_copy("neighbors").expect("copy");
        assert!(copy.kicker.contains("/ip/neighbor"));
        assert!(!copy.body.contains('\u{2014}'));
    }

    #[test]
    fn macsec_guide_tracks_the_manual() {
        let guide = screen_guide("macsec").expect("macsec");
        let copy = about_copy("macsec").expect("copy");
        let hay = format!("{} {} {}", guide.summary, guide.use_when, guide.fields);
        for needle in ["802.1AE", "GCM-AES-128", "CAK", "CKN", "Dot1x", "Ethernet"] {
            assert!(hay.contains(needle), "missing {needle}");
        }
        assert!(copy.body.contains("manual.mikrotik.com"));
        assert!(copy.body.contains("/interface/macsec"));
        assert_eq!(
            guide.docs_url,
            Some("https://manual.mikrotik.com/docs/bridging-and-switching/macsec/")
        );
        assert!(copy.kicker.contains("/interface/macsec"));
        assert!(
            !copy.body.to_ascii_lowercase().contains("paraphrased"),
            "about copy must not mention paraphrasing"
        );
    }

    #[test]
    fn lte_apn_guide_points_at_the_cli_reference() {
        let guide = screen_guide("lte-apn").expect("lte-apn");
        let copy = about_copy("lte-apn").expect("copy");
        let hay = format!("{} {} {}", guide.summary, guide.use_when, guide.fields);
        for needle in ["APN", "authentication", "use-network-apn"] {
            assert!(hay.contains(needle), "missing {needle}");
        }
        assert!(copy.kicker.contains("/interface/lte/apn"));
        assert!(copy.body.contains("/interface/lte/apn"));
        assert!(copy.body.contains("manual.mikrotik.com"));
        assert!(
            !copy.body.to_ascii_lowercase().contains("paraphrased"),
            "about copy must not mention paraphrasing"
        );
        assert!(!copy.body.contains('\u{2014}'));
    }

    #[test]
    fn cli_docs_url_follows_the_resource_path() {
        let copy = about_copy("vlan").expect("vlan");
        assert!(
            copy.body
                .contains("https://manual.mikrotik.com/docs/cli-reference/interface/vlan/")
        );
        assert_eq!(copy.title, "About VLAN");
    }

    #[test]
    fn interface_list_guides_cross_link_definitions_and_members() {
        let lists = about_copy("interface-lists").expect("lists");
        let members = about_copy("interface-list-members").expect("members");
        assert_eq!(lists.title, "About Lists");
        assert_eq!(members.title, "About List members");
        assert!(lists.body.contains("List members"));
        assert!(members.body.contains("Lists"));
        assert!(lists.body.to_ascii_lowercase().contains("include"));
        assert!(lists.body.to_ascii_lowercase().contains("exclude"));
        assert!(members.body.contains("join") || members.body.contains("Joins"));
    }

    #[test]
    fn ipv6_firewall_connections_guide_mirrors_ipv4() {
        let guide = screen_guide("ipv6-firewall-connections").expect("guide");
        let copy = about_copy("ipv6-firewall-connections").expect("copy");
        assert_eq!(copy.title, "About Connections");
        assert!(copy.kicker.contains("/ipv6/firewall/connection"));
        assert!(copy.body.contains("/ipv6/firewall/connection"));
        assert!(guide.summary.to_ascii_lowercase().contains("ipv6"));
        assert!(guide.use_when.to_ascii_lowercase().contains("remove"));
        assert!(guide.fields.contains("src/dst-address"));
        assert_eq!(
            guide.docs_url,
            Some(
                "https://manual.mikrotik.com/docs/firewall-and-quality-of-service/connection-tracking/"
            )
        );
        assert!(
            !copy.body.contains('\u{2014}'),
            "about copy must not use em dashes"
        );
    }

    #[test]
    fn ospf_interface_guide_is_runtime_not_a_template() {
        let live = about_copy("ospf-interfaces").expect("ospf-interfaces");
        let templates = about_copy("ospf-interface-templates").expect("templates");
        assert_eq!(live.title, "About OSPF Interface");
        assert_eq!(templates.title, "About OSPF Interface Templates");
        assert!(live.kicker.contains("/routing/ospf/interface"));
        assert!(!live.kicker.contains("interface-template"));
        assert!(templates.kicker.contains("interface-template"));
        assert!(live.body.contains("Monitor-only"));
        assert!(live.body.contains("cost"));
        assert!(templates.body.contains("OSPF Interface"));
        assert!(!templates.body.contains("no separate"));
        assert!(
            live.body
                .contains("https://manual.mikrotik.com/docs/cli-reference/routing/ospf/interface/")
        );
    }

    #[test]
    fn traffic_flow_and_igmp_guides_track_the_manual() {
        let flow = about_copy("traffic-flow").expect("traffic-flow");
        assert_eq!(flow.title, "About Traffic Flow");
        assert!(flow.kicker.contains("/ip/traffic-flow"));
        assert!(flow.body.contains("NetFlow") || flow.body.contains("IPFIX"));
        assert!(flow.body.contains("Packet Sampling"));
        assert!(!flow.body.contains('\u{2014}'));

        let targets = about_copy("traffic-flow-targets").expect("targets");
        assert!(targets.body.contains("Dst. Address"));
        assert!(targets.body.contains("ipfix"));

        let proxy = about_copy("igmp-proxy").expect("igmp-proxy");
        assert!(proxy.kicker.contains("/routing/igmp-proxy"));
        assert!(proxy.body.contains("upstream"));
        assert!(!proxy.body.contains('\u{2014}'));

        let ifaces = about_copy("igmp-proxy-interfaces").expect("ifaces");
        assert!(ifaces.body.contains("Upstream"));
        let mfc = about_copy("igmp-proxy-mfc").expect("mfc");
        assert!(mfc.body.contains("Group"));
    }

    #[test]
    fn romon_and_graphing_guides_track_the_manual() {
        let romon = about_copy("romon").expect("romon");
        let ports = about_copy("romon-ports").expect("ports");
        let graphing = about_copy("graphing").expect("graphing");
        assert_eq!(romon.title, "About RoMON");
        assert_eq!(ports.title, "About RoMON Ports");
        assert_eq!(graphing.title, "About Graphing");
        assert!(romon.kicker.contains("/tool/romon"));
        assert!(ports.kicker.contains("/tool/romon/port"));
        assert!(graphing.kicker.contains("/tool/graphing"));
        assert!(romon.body.to_ascii_lowercase().contains("secret"));
        assert!(ports.body.contains("forbid"));
        assert!(graphing.body.contains("/graphs/"));
        assert!(
            about_copy("graphing-interface")
                .expect("gi")
                .kicker
                .contains("/tool/graphing/interface")
        );
        assert!(
            about_copy("graphing-queue")
                .expect("gq")
                .kicker
                .contains("/tool/graphing/queue")
        );
        assert!(
            about_copy("graphing-resource")
                .expect("gr")
                .kicker
                .contains("/tool/graphing/resource")
        );
        for copy in [&romon, &ports, &graphing] {
            assert!(
                !copy.body.contains('\u{2014}'),
                "about copy must not use em dashes"
            );
        }
        assert_eq!(
            screen_guide("romon").expect("g").docs_url,
            Some("https://manual.mikrotik.com/docs/management-tools/romon/")
        );
        assert_eq!(
            screen_guide("graphing").expect("g").docs_url,
            Some(
                "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/graphing/"
            )
        );
    }

    #[test]
    fn history_guide_covers_undo_and_keeps_safe_mode_separate() {
        let copy = about_copy("history").expect("history");
        assert_eq!(copy.title, "About History");
        assert!(copy.kicker.contains("/system/history"));
        assert!(copy.body.contains("undo"));
        assert!(copy.body.contains("F4"));
        assert!(copy.body.contains("Safe Mode"));
        assert!(copy.body.contains("floating-undo"));
    }
}
