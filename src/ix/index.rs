use super::*;
use dashmap::DashSet;
use itertools::Itertools;
use std::cmp::min;

#[derive(Clone, Copy)]
pub struct Node<M: Copy, const N: usize> {
    pub key: &'static str,
    pub vector: Vector<N>,
    pub metadata: M,
}

#[derive(Debug)]
pub struct QueryResult<M: Copy> {
    pub key: &'static str,
    pub distance: f32,
    pub metadata: M,
}

pub struct Index<M: Copy, const N: usize> {
    trees: Vec<Tree<N>>,
    metadata: HashMap<&'static str, M>,
    vectors: HashMap<&'static str, Vector<N>>,
    config: IndexConfig,
}

#[derive(Clone, Copy)]
pub struct IndexConfig {
    pub num_trees: i32,
    pub max_leaf_size: i32,
}

impl<M: Copy, const N: usize> Index<M, N> {
    fn deduplicate(nodes: &Vec<Node<M, N>>) -> Vec<Node<M, N>> {
        let mut unique_nodes = vec![];
        let hashes_seen = DashSet::new();

        for node in nodes {
            let hash_key = node.vector.to_hashkey();
            if !hashes_seen.contains(&hash_key) {
                hashes_seen.insert(hash_key);
                unique_nodes.push(*node);
            }
        }

        unique_nodes
    }

    pub fn build(nodes: &Vec<Node<M, N>>, config: &IndexConfig) -> Index<M, N> {
        let nodes = Self::deduplicate(nodes);

        let keys = nodes.iter().map(|node| node.key).collect();

        let mut metadata = HashMap::new();
        let mut vectors = HashMap::new();

        for node in nodes.iter() {
            metadata.insert(node.key, node.metadata);
            vectors.insert(node.key, node.vector);
        }

        let trees: Vec<Tree<N>> = (0..config.num_trees)
            .map(|_| Tree::build(&keys, &vectors, config.max_leaf_size))
            .collect();

        let config = *config;

        Index::<M, N> { trees, metadata, vectors, config }
    }

    fn candidates_from_leaf(
        candidates: &DashSet<&str>,
        leaf: &Vec<&'static str>,
        n: i32,
    ) -> i32 {
        let num_candidates = min(n as usize, leaf.len());
        for item in leaf.iter().take(num_candidates) {
            candidates.insert(item);
        }
        num_candidates as i32
    }

    fn candidates_from_branch(
        candidates: &DashSet<&str>,
        branch: &Branch<N>,
        vector: &Vector<N>,
        n: i32,
    ) -> i32 {
        let above = branch.hyperplane.point_is_above(vector);

        let (main_tree, backup_tree) = match above {
            true => (&branch.right_tree, &branch.left_tree),
            false => (&branch.left_tree, &branch.right_tree),
        };

        let num_candidates =
            Self::get_candidates(candidates, main_tree, vector, n);

        if num_candidates >= n {
            return num_candidates;
        }

        num_candidates
            + Self::get_candidates(
                candidates,
                backup_tree,
                vector,
                n - num_candidates,
            )
    }

    fn get_candidates(
        candidates: &DashSet<&str>,
        tree: &Tree<N>,
        vector: &Vector<N>,
        n: i32,
    ) -> i32 {
        match tree {
            Tree::Leaf(leaf) => Self::candidates_from_leaf(candidates, leaf, n),
            Tree::Branch(branch) => {
                Self::candidates_from_branch(candidates, branch, vector, n)
            }
        }
    }

    pub fn query(&self, vector: &Vector<N>, n: i32) -> Vec<QueryResult<M>> {
        let candidates = DashSet::new();

        self.trees.iter().for_each(|tree| {
            Self::get_candidates(&candidates, tree, vector, n);
        });

        let sorted_candidates: Vec<_> = candidates
            .into_iter()
            .map(|key| (key, self.vectors[key].euclidean_distance(vector)))
            .sorted_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .take(n as usize)
            .collect();

        let mut result = vec![];

        for (key, distance) in sorted_candidates.iter() {
            let metadata = self.metadata[key];
            result.push(QueryResult { key, distance: *distance, metadata });
        }

        result
    }
}