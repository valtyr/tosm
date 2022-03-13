use kdtree::KdTree;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Node {
    id: u64,
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Way {
    id: u64,
    node_ids: Vec<u64>,
    one_way: bool,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SourceFile {
    nodes: Vec<Node>,
    ways: Vec<Way>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TOSMFile {
    nodes: Vec<Node>,
    ways: Vec<Way>,

    node_indexes: HashMap<u64, usize>,
    way_indexes: HashMap<u64, usize>,

    kd_tree: KdTree<f64, u64, [f64; 2]>,
}

fn parse_file(path: &str) -> TOSMFile {
    let mut file = TOSMFile {
        nodes: vec![],
        ways: vec![],
        node_indexes: HashMap::new(),
        way_indexes: HashMap::new(),
        kd_tree: KdTree::new(2),
    };

    {
        let source = std::fs::read_to_string(path).unwrap();
        let v: SourceFile = serde_json::from_str(&source).unwrap();

        for node in v.nodes {
            file.nodes.push(node.clone());
            file.node_indexes.insert(node.id, file.nodes.len());
            file.kd_tree.add([node.lat, node.lon], node.id).unwrap();
        }

        for way in v.ways {
            file.ways.push(way.clone());
            file.way_indexes.insert(way.id, file.ways.len());
        }
    }

    file
}

fn dist_haversine(a: &[f64], b: &[f64]) -> f64 {
    let lat1 = a[0].to_radians();
    let lon1 = a[1].to_radians();

    let lat2 = b[0].to_radians();
    let lon2 = b[1].to_radians();

    let dlathalf = (lat2 - lat1) * 0.5;
    let dlonhalf = (lon2 - lon1) * 0.5;

    let sqrth =
        (dlathalf.sin().powi(2) + (lat1.cos() * lat2.cos() * dlonhalf.sin().powi(2))).sqrt();

    return sqrth.asin() * 2.0 * 6371.0;
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::{dist_haversine, parse_file, TOSMFile};

    #[test]
    fn finds_fjolugata() {
        let file = parse_file("out.json");

        let out_file = File::create("iceland.tosm.br").unwrap();
        let compressor = brotli::CompressorWriter::new(out_file, 4096, 4, 21);
        bincode::serialize_into(compressor, &file).unwrap();

        let res = file
            .kd_tree
            .nearest(&[64.142257_f64, -21.938559_f64], 1, &dist_haversine)
            .unwrap();

        let (_, result) = res.first().unwrap().to_owned();

        assert_eq!(result, &35618126)
    }

    #[test]
    fn can_read_from_file() {
        let in_file = File::open("iceland.tosm.br").unwrap();
        let decompressor = brotli::Decompressor::new(in_file, 4096);
        let file: TOSMFile = bincode::deserialize_from(decompressor).unwrap();

        let res = file
            .kd_tree
            .nearest(&[64.142257_f64, -21.938559_f64], 1, &dist_haversine)
            .unwrap();

        let (_, result) = res.first().unwrap().to_owned();

        assert_eq!(result, &35618126)
    }
}
