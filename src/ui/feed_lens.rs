use crate::model::{issue_id_sort_key, Issue, Status, Workspace};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

type LinkMetrics = (HashMap<String, usize>, HashMap<String, usize>, HashMap<String, usize>);
type LinkAdjacency = HashMap<String, HashSet<String>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FeedLens {
    MyOrder,
    NextUp,
    HotPath,
    QuickWins,
    LinkGroups,
}

pub(crate) fn apply_feed_lens(
    all_issues: &[Issue],
    mut issues: Vec<Issue>,
    lens: FeedLens,
) -> Vec<Issue> {
    if lens == FeedLens::MyOrder {
        return issues;
    }

    let metrics = LensMetrics::from_issues(all_issues);
    issues.sort_by(|left, right| {
        metrics
            .sort_key(left, lens)
            .cmp(&metrics.sort_key(right, lens))
            .then_with(|| issue_id_sort_key(&left.id).cmp(&issue_id_sort_key(&right.id)))
    });
    issues
}

#[derive(Debug, Default)]
struct LensMetrics {
    hot_scores: HashMap<String, usize>,
    unblock_scores: HashMap<String, usize>,
    quick_costs: HashMap<String, usize>,
    link_component_orders: HashMap<String, usize>,
    link_component_sizes: HashMap<String, usize>,
    link_degrees: HashMap<String, usize>,
}

impl LensMetrics {
    fn from_issues(issues: &[Issue]) -> Self {
        let ws = Workspace {
            root: PathBuf::new(),
            issues: issues.to_vec(),
        };

        let hot_scores = build_hot_scores(&ws, issues);
        let quick_costs = build_quick_costs(issues, &hot_scores);
        let unblock_scores = build_unblock_scores(&ws, issues);
        let (link_component_orders, link_component_sizes, link_degrees) =
            build_link_metrics(issues);

        Self {
            hot_scores,
            unblock_scores,
            quick_costs,
            link_component_orders,
            link_component_sizes,
            link_degrees,
        }
    }

    fn sort_key(
        &self,
        issue: &Issue,
        lens: FeedLens,
    ) -> (usize, usize, usize, (String, u32, String)) {
        match lens {
            FeedLens::MyOrder => (0, 0, 0, issue_id_sort_key(&issue.id)),
            FeedLens::NextUp => (
                usize::MAX
                    - self
                        .unblock_scores
                        .get(&issue.id)
                        .copied()
                        .unwrap_or_default(),
                issue.status_ord() as usize,
                self.quick_costs.get(&issue.id).copied().unwrap_or_default(),
                issue_id_sort_key(&issue.id),
            ),
            FeedLens::HotPath => (
                usize::MAX - self.hot_scores.get(&issue.id).copied().unwrap_or_default(),
                issue.status_ord() as usize,
                issue.files.len(),
                issue_id_sort_key(&issue.id),
            ),
            FeedLens::QuickWins => (
                self.quick_costs.get(&issue.id).copied().unwrap_or_default(),
                issue.status_ord() as usize,
                self.unblock_scores
                    .get(&issue.id)
                    .copied()
                    .unwrap_or_default(),
                issue_id_sort_key(&issue.id),
            ),
            FeedLens::LinkGroups => (
                self.link_component_orders
                    .get(&issue.id)
                    .copied()
                    .unwrap_or(usize::MAX),
                usize::MAX
                    - self
                        .link_component_sizes
                        .get(&issue.id)
                        .copied()
                        .unwrap_or(1),
                usize::MAX - self.link_degrees.get(&issue.id).copied().unwrap_or_default(),
                issue_id_sort_key(&issue.id),
            ),
        }
    }
}

fn build_hot_scores(ws: &Workspace, issues: &[Issue]) -> HashMap<String, usize> {
    let mut file_weights = HashMap::<String, usize>::new();
    for (file, ids) in ws.file_heatmap() {
        file_weights.insert(file, ids.len());
    }

    issues
        .iter()
        .map(|issue| {
            let score = issue
                .files
                .iter()
                .map(|file| file_weights.get(file).copied().unwrap_or(1))
                .sum::<usize>();
            (issue.id.clone(), score)
        })
        .collect::<HashMap<_, _>>()
}

fn build_quick_costs(
    issues: &[Issue],
    hot_scores: &HashMap<String, usize>,
) -> HashMap<String, usize> {
    issues
        .iter()
        .map(|issue| {
            let heat = hot_scores.get(&issue.id).copied().unwrap_or_default();
            let cost = heat + (issue.files.len() * 2) + (issue.depends_on.len() * 3);
            (issue.id.clone(), cost)
        })
        .collect::<HashMap<_, _>>()
}

fn build_unblock_scores(ws: &Workspace, issues: &[Issue]) -> HashMap<String, usize> {
    let mut dependents = HashMap::<String, Vec<String>>::new();
    for (dependency, dependent) in ws.dependency_edges() {
        dependents.entry(dependency).or_default().push(dependent);
    }

    let active_issue_ids = issues
        .iter()
        .filter(|issue| issue.status != Status::Done && issue.status != Status::Descoped)
        .map(|issue| issue.id.clone())
        .collect::<HashSet<_>>();

    let mut unblock_scores = HashMap::<String, usize>::new();
    for issue in issues {
        let mut visited = HashSet::new();
        let score = transitive_dependents(&issue.id, &dependents, &active_issue_ids, &mut visited);
        unblock_scores.insert(issue.id.clone(), score);
    }
    unblock_scores
}

fn build_link_metrics(issues: &[Issue]) -> LinkMetrics {
    let adjacency = build_link_adjacency(issues);
    let link_degrees = adjacency
        .iter()
        .map(|(id, neighbors)| (id.clone(), neighbors.len()))
        .collect::<HashMap<_, _>>();
    let mut components = collect_link_components(issues, &adjacency);
    sort_components(&mut components);
    component_maps(&components, link_degrees)
}

fn build_link_adjacency(issues: &[Issue]) -> LinkAdjacency {
    let known_ids = issues
        .iter()
        .map(|issue| issue.id.clone())
        .collect::<HashSet<_>>();
    let mut adjacency = issues
        .iter()
        .map(|issue| (issue.id.clone(), HashSet::<String>::new()))
        .collect::<LinkAdjacency>();

    for issue in issues {
        for linked in valid_links(issue, &known_ids) {
            adjacency
                .entry(issue.id.clone())
                .or_default()
                .insert(linked.clone());
            adjacency
                .entry(linked)
                .or_default()
                .insert(issue.id.clone());
        }
    }

    adjacency
}

fn valid_links(issue: &Issue, known_ids: &HashSet<String>) -> Vec<String> {
    issue
        .links
        .iter()
        .filter(|linked| known_ids.contains(*linked) && *linked != &issue.id)
        .cloned()
        .collect::<Vec<_>>()
}

fn collect_link_components(issues: &[Issue], adjacency: &LinkAdjacency) -> Vec<Vec<String>> {
    let mut components = Vec::<Vec<String>>::new();
    let mut visited = HashSet::<String>::new();

    for issue in issues {
        if !visited.insert(issue.id.clone()) {
            continue;
        }
        components.push(walk_component(&issue.id, adjacency, &mut visited));
    }

    components
}

fn walk_component(
    root: &str,
    adjacency: &LinkAdjacency,
    visited: &mut HashSet<String>,
) -> Vec<String> {
    let mut stack = vec![root.to_string()];
    let mut component = vec![];

    while let Some(current) = stack.pop() {
        component.push(current.clone());
        if let Some(neighbors) = adjacency.get(&current) {
            for neighbor in neighbors {
                if visited.insert(neighbor.clone()) {
                    stack.push(neighbor.clone());
                }
            }
        }
    }

    component
}

fn sort_components(components: &mut [Vec<String>]) {
    components.sort_by_key(|component| {
        component
            .iter()
            .map(|id| issue_id_sort_key(id))
            .min()
            .unwrap_or_else(|| issue_id_sort_key(""))
    });
}

fn component_maps(
    components: &[Vec<String>],
    link_degrees: HashMap<String, usize>,
) -> LinkMetrics {
    let mut link_component_orders = HashMap::<String, usize>::new();
    let mut link_component_sizes = HashMap::<String, usize>::new();

    for (order, component) in components.iter().enumerate() {
        let size = component.len();
        for id in component {
            link_component_orders.insert(id.clone(), order);
            link_component_sizes.insert(id.clone(), size);
        }
    }

    (link_component_orders, link_component_sizes, link_degrees)
}

fn transitive_dependents(
    id: &str,
    dependents: &HashMap<String, Vec<String>>,
    active_issue_ids: &HashSet<String>,
    visited: &mut HashSet<String>,
) -> usize {
    let Some(children) = dependents.get(id) else {
        return 0;
    };

    let mut total = 0;
    for child in children {
        if !active_issue_ids.contains(child) || !visited.insert(child.clone()) {
            continue;
        }
        total += 1 + transitive_dependents(child, dependents, active_issue_ids, visited);
    }
    total
}

#[cfg(test)]
mod tests {
    use super::{apply_feed_lens, FeedLens};
    use crate::model::{Issue, Status};

    fn make_issue_with_graph(
        id: &str,
        title: &str,
        status: Status,
        files: &[&str],
        depends_on: &[&str],
    ) -> Issue {
        Issue {
            id: id.to_string(),
            title: title.to_string(),
            status,
            files: files.iter().map(|file| file.to_string()).collect(),
            labels: vec![],
            links: vec![],
            description: String::new(),
            resolution: String::new(),
            section: "ACTIVE Issues".to_string(),
            depends_on: depends_on.iter().map(|dep| dep.to_string()).collect(),
        }
    }

    #[test]
    fn next_up_lens_prioritizes_transitive_unblock_count() {
        let issues = vec![
            make_issue_with_graph("BUG-01", "Base", Status::Open, &["src/main.rs"], &[]),
            make_issue_with_graph("BUG-02", "Middle", Status::Open, &["src/main.rs"], &["BUG-01"]),
            make_issue_with_graph("BUG-03", "Leaf", Status::Open, &["src/ui.rs"], &["BUG-02"]),
        ];

        let sorted = apply_feed_lens(&issues, issues.clone(), FeedLens::NextUp);
        assert_eq!(
            sorted.iter().map(|issue| issue.id.clone()).collect::<Vec<_>>(),
            vec!["BUG-01", "BUG-02", "BUG-03"]
        );
    }

    #[test]
    fn hot_path_lens_prioritizes_hotter_files() {
        let issues = vec![
            make_issue_with_graph("BUG-01", "Shared A", Status::Open, &["src/main.rs"], &[]),
            make_issue_with_graph("BUG-02", "Shared B", Status::Open, &["src/main.rs"], &[]),
            make_issue_with_graph("BUG-03", "Cold", Status::Open, &["src/cold.rs"], &[]),
        ];

        let sorted = apply_feed_lens(&issues, issues.clone(), FeedLens::HotPath);
        assert_eq!(sorted[0].id, "BUG-01");
        assert_eq!(sorted[1].id, "BUG-02");
        assert_eq!(sorted[2].id, "BUG-03");
    }

    #[test]
    fn quick_wins_lens_prefers_lower_cost_work() {
        let issues = vec![
            make_issue_with_graph("BUG-01", "Wide", Status::Open, &["a.rs", "b.rs"], &["BUG-09"]),
            make_issue_with_graph("BUG-02", "Tight", Status::Open, &["solo.rs"], &[]),
        ];

        let sorted = apply_feed_lens(&issues, issues.clone(), FeedLens::QuickWins);
        assert_eq!(sorted[0].id, "BUG-02");
        assert_eq!(sorted[1].id, "BUG-01");
    }

    #[test]
    fn linked_lens_clusters_connected_issues() {
        let mut alpha = make_issue_with_graph("BUG-01", "Alpha", Status::Open, &["a.rs"], &[]);
        alpha.links = vec!["BUG-03".to_string()];
        let beta = make_issue_with_graph("BUG-02", "Beta", Status::Open, &["b.rs"], &[]);
        let mut gamma = make_issue_with_graph("BUG-03", "Gamma", Status::Open, &["c.rs"], &[]);
        gamma.links = vec!["BUG-01".to_string()];

        let sorted = apply_feed_lens(
            &[alpha.clone(), beta.clone(), gamma.clone()],
            vec![alpha, beta, gamma],
            FeedLens::LinkGroups,
        );
        assert_eq!(
            sorted.iter().map(|issue| issue.id.as_str()).collect::<Vec<_>>(),
            vec!["BUG-01", "BUG-03", "BUG-02"]
        );
    }

    #[test]
    fn linked_lens_keeps_unrelated_issues_in_stable_order() {
        let first = make_issue_with_graph("BUG-01", "First", Status::Open, &["a.rs"], &[]);
        let second = make_issue_with_graph("BUG-02", "Second", Status::Open, &["b.rs"], &[]);
        let third = make_issue_with_graph("BUG-03", "Third", Status::Open, &["c.rs"], &[]);

        let sorted = apply_feed_lens(
            &[first.clone(), second.clone(), third.clone()],
            vec![third, first, second],
            FeedLens::LinkGroups,
        );
        assert_eq!(
            sorted.iter().map(|issue| issue.id.as_str()).collect::<Vec<_>>(),
            vec!["BUG-01", "BUG-02", "BUG-03"]
        );
    }
}
