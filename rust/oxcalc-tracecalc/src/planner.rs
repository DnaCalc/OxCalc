#![forbid(unsafe_code)]

//! `TraceCalc` workset planner.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde_json::Value;

use crate::contracts::{TraceCalcNode, TraceCalcScenario};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCalcWorksetPlan {
    pub groups: Vec<Vec<String>>,
    pub ordered_nodes: Vec<String>,
    pub impacted_nodes: Vec<String>,
    pub cycle_groups: Vec<Vec<String>>,
}

impl TraceCalcWorksetPlan {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            groups: Vec::new(),
            ordered_nodes: Vec::new(),
            impacted_nodes: Vec::new(),
            cycle_groups: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceCalcScenarioPlanner {
    nodes: BTreeMap<String, TraceCalcNode>,
    direct_dependencies: BTreeMap<String, Vec<String>>,
    reverse_dependencies: BTreeMap<String, Vec<String>>,
}

impl TraceCalcScenarioPlanner {
    #[must_use]
    pub fn new(scenario: &TraceCalcScenario) -> Self {
        let nodes = scenario
            .initial_graph
            .nodes
            .iter()
            .cloned()
            .map(|node| (node.node_id.clone(), node))
            .collect::<BTreeMap<_, _>>();
        let direct_dependencies = build_direct_dependencies(&nodes);
        let reverse_dependencies = build_reverse_dependencies(&direct_dependencies);
        Self {
            nodes,
            direct_dependencies,
            reverse_dependencies,
        }
    }

    #[must_use]
    pub fn plan_workset(
        &self,
        explicit_targets: &[String],
        dirty_seeds: &BTreeSet<String>,
    ) -> TraceCalcWorksetPlan {
        let mut impacted = BTreeSet::new();
        let mut queue = VecDeque::new();

        for dirty_seed in dirty_seeds {
            if self.nodes.contains_key(dirty_seed) && impacted.insert(dirty_seed.clone()) {
                queue.push_back(dirty_seed.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(dependents) = self.reverse_dependencies.get(&current) {
                for dependent in dependents {
                    if impacted.insert(dependent.clone()) {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        for target in explicit_targets {
            if self.nodes.contains_key(target) {
                impacted.insert(target.clone());
            }
        }

        if impacted.is_empty() {
            return TraceCalcWorksetPlan::empty();
        }

        let components = self.compute_components(&impacted);
        let mut component_index = BTreeMap::new();
        for (index, component) in components.iter().enumerate() {
            for node_id in component {
                component_index.insert(node_id.clone(), index);
            }
        }

        let mut outgoing = BTreeMap::new();
        let mut indegrees = BTreeMap::new();
        for index in 0..components.len() {
            outgoing.insert(index, BTreeSet::new());
            indegrees.insert(index, 0_usize);
        }

        for node_id in &impacted {
            let from_component = component_index[node_id];
            if let Some(dependencies) = self.direct_dependencies.get(node_id) {
                for dependency in dependencies {
                    if !impacted.contains(dependency) {
                        continue;
                    }
                    let dependency_component = component_index[dependency];
                    if dependency_component == from_component {
                        continue;
                    }
                    if outgoing
                        .get_mut(&dependency_component)
                        .expect("component map initialized")
                        .insert(from_component)
                    {
                        *indegrees
                            .get_mut(&from_component)
                            .expect("component map initialized") += 1;
                    }
                }
            }
        }

        let mut ready = indegrees
            .iter()
            .filter(|(_, indegree)| **indegree == 0)
            .map(|(index, _)| *index)
            .collect::<BTreeSet<_>>();

        let mut ordered_groups = Vec::new();
        while let Some(current_component) = ready.iter().next().copied() {
            ready.remove(&current_component);
            ordered_groups.push(components[current_component].clone());
            for next_component in outgoing[&current_component].clone() {
                let indegree = indegrees
                    .get_mut(&next_component)
                    .expect("component map initialized");
                *indegree -= 1;
                if *indegree == 0 {
                    ready.insert(next_component);
                }
            }
        }

        let ordered_nodes = ordered_groups
            .iter()
            .flat_map(|group| group.iter().cloned())
            .collect::<Vec<_>>();
        let cycle_groups = components
            .iter()
            .filter(|component| self.is_cycle_group(component))
            .cloned()
            .collect::<Vec<_>>();

        TraceCalcWorksetPlan {
            groups: ordered_groups,
            ordered_nodes,
            impacted_nodes: impacted.into_iter().collect(),
            cycle_groups,
        }
    }

    fn compute_components(&self, impacted: &BTreeSet<String>) -> Vec<Vec<String>> {
        let mut index_by_node = BTreeMap::new();
        let mut low_link = BTreeMap::new();
        let mut active = BTreeSet::new();
        let mut stack = Vec::new();
        let mut components = Vec::new();
        let mut index = 0_usize;

        for node_id in impacted {
            if !index_by_node.contains_key(node_id) {
                self.strong_connect(
                    node_id,
                    impacted,
                    &mut index,
                    &mut index_by_node,
                    &mut low_link,
                    &mut active,
                    &mut stack,
                    &mut components,
                );
            }
        }

        components
    }

    #[allow(clippy::too_many_arguments)]
    fn strong_connect(
        &self,
        node_id: &str,
        impacted: &BTreeSet<String>,
        index: &mut usize,
        index_by_node: &mut BTreeMap<String, usize>,
        low_link: &mut BTreeMap<String, usize>,
        active: &mut BTreeSet<String>,
        stack: &mut Vec<String>,
        components: &mut Vec<Vec<String>>,
    ) {
        index_by_node.insert(node_id.to_string(), *index);
        low_link.insert(node_id.to_string(), *index);
        *index += 1;
        stack.push(node_id.to_string());
        active.insert(node_id.to_string());

        if let Some(dependencies) = self.direct_dependencies.get(node_id) {
            for dependency in dependencies {
                if !impacted.contains(dependency) {
                    continue;
                }

                if !index_by_node.contains_key(dependency) {
                    self.strong_connect(
                        dependency,
                        impacted,
                        index,
                        index_by_node,
                        low_link,
                        active,
                        stack,
                        components,
                    );
                    let candidate = low_link[dependency];
                    let entry = low_link.get_mut(node_id).expect("existing low link");
                    *entry = (*entry).min(candidate);
                } else if active.contains(dependency) {
                    let candidate = index_by_node[dependency];
                    let entry = low_link.get_mut(node_id).expect("existing low link");
                    *entry = (*entry).min(candidate);
                }
            }
        }

        if low_link[node_id] != index_by_node[node_id] {
            return;
        }

        let mut component = Vec::new();
        while let Some(member) = stack.pop() {
            active.remove(&member);
            component.push(member.clone());
            if member == node_id {
                break;
            }
        }

        component.sort();
        components.push(component);
    }

    fn is_cycle_group(&self, component: &[String]) -> bool {
        if component.len() > 1 {
            return true;
        }

        let node_id = &component[0];
        self.direct_dependencies
            .get(node_id)
            .is_some_and(|dependencies| dependencies.contains(node_id))
    }
}

fn build_direct_dependencies(
    nodes: &BTreeMap<String, TraceCalcNode>,
) -> BTreeMap<String, Vec<String>> {
    let mut result = BTreeMap::new();
    for (node_id, node) in nodes {
        let mut dependencies = parse_dependencies(&node.expression)
            .into_iter()
            .filter(|dependency| nodes.contains_key(dependency))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        dependencies.sort();
        result.insert(node_id.clone(), dependencies);
    }

    result
}

fn build_reverse_dependencies(
    direct_dependencies: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, Vec<String>> {
    let mut reverse = direct_dependencies
        .keys()
        .map(|node_id| (node_id.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();

    for (node_id, dependencies) in direct_dependencies {
        for dependency in dependencies {
            reverse
                .entry(dependency.clone())
                .or_default()
                .insert(node_id.clone());
        }
    }

    reverse
        .into_iter()
        .map(|(node_id, dependents)| (node_id, dependents.into_iter().collect()))
        .collect()
}

fn parse_dependencies(expression: &Value) -> Vec<String> {
    let op = expression
        .get("op")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match op {
        "sum" | "concat" => read_string_array(expression.get("deps")),
        "choose" => read_explicit_dependencies(expression, &["control", "when_true", "when_false"]),
        "dyn_select" => read_dynamic_select_dependencies(expression),
        "cap_gate" | "delay" => read_explicit_dependencies(expression, &["dep"]),
        _ => read_string_array(expression.get("deps")),
    }
}

fn read_dynamic_select_dependencies(expression: &Value) -> Vec<String> {
    let mut values = read_explicit_dependencies(expression, &["selector"]);
    if let Some(candidates) = expression.get("candidates").and_then(Value::as_object) {
        for value in candidates.values() {
            if let Some(candidate) = value.as_str() {
                values.push(candidate.to_string());
            }
        }
    }
    values
}

fn read_explicit_dependencies(expression: &Value, property_names: &[&str]) -> Vec<String> {
    let mut values = Vec::new();
    for property_name in property_names {
        if let Some(value) = expression.get(property_name).and_then(Value::as_str) {
            values.push(value.to_string());
        }
    }
    values
}

fn read_string_array(node: Option<&Value>) -> Vec<String> {
    match node.and_then(Value::as_array) {
        Some(array) => array
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        None => Vec::new(),
    }
}
