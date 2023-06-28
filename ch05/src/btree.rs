use std::cmp;
use std::collections::HashMap;
use std::mem;

#[derive(Clone, Debug)]
pub struct IoTDevice {
    pub numerical_id: u64,
    pub path: String,
    pub address: String,
}

impl IoTDevice {
    pub fn new(id: u64, address: impl Into<String>, path: impl Into<String>) -> IoTDevice {
        IoTDevice {
            address: address.into(),
            numerical_id: id,
            path: path.into(),
        }
    }
}

impl PartialEq for IoTDevice {
    fn eq(&self, other: &IoTDevice) -> bool {
        self.numerical_id == other.numerical_id && self.address == other.address
    }
}

type Tree = Box<Node>;
type KeyType = u64;

type Data = (Option<IoTDevice>, Option<Tree>);

#[derive(Clone, PartialEq, Debug)]
enum NodeType {
    Leaf,
    Regular,
}

#[derive(Clone, PartialEq)]
enum Direction {
    Left,
    Right(usize),
}

#[derive(Clone)]
struct Node {
    devices: Vec<Option<IoTDevice>>,
    children: Vec<Option<Tree>>,
    left_child: Option<Tree>,
    pub node_type: NodeType,
}

impl Node {
    pub fn new_leaf() -> Tree {
        Node::new(NodeType::Leaf)
    }

    pub fn new_regular() -> Tree {
        Node::new(NodeType::Regular)
    }

    fn new(node_type: NodeType) -> Tree {
        Box::new(Node {
            left_child: None,
            devices: vec![],
            children: vec![],
            node_type: node_type,
        })
    }

    pub fn len(&self) -> usize {
        self.children.len() + 1
    }

    pub fn split(&mut self) -> (IoTDevice, Tree) {
        let mut sibling = Node::new(self.node_type.clone());

        let no_of_devices = self.devices.len();
        let split_at = no_of_devices / 2usize;

        let dev = self.devices.remove(split_at);
        let node = self.children.remove(split_at);

        for _ in split_at..self.devices.len() {
            let device = self.devices.pop().unwrap();
            let child = self.children.pop().unwrap();
            sibling.add_key(device.as_ref().unwrap().numerical_id, (device, child));
        }

        sibling.add_left_child(node);
        (dev.unwrap(), sibling)
    }

    pub fn add_left_child(&mut self, tree: Option<Tree>) {
        self.left_child = tree;
    }

    pub fn add_key(&mut self, key: KeyType, value: Data) -> bool {
        let pos = match self.find_closest_index(key) {
            Direction::Left => 0,
            Direction::Right(p) => p + 1,
        };
        let (dev, tree) = value;

        if pos >= self.devices.len() {
            self.devices.push(dev);
            self.children.push(tree);
        } else {
            self.devices.insert(pos, dev);
            self.children.insert(pos, tree);
        }
        true
    }

    pub fn remove_key(&mut self, id: KeyType) -> Option<(KeyType, Data)> {
        match self.find_closest_index(id) {
            Direction::Left => {
                let tree = mem::replace(&mut self.left_child, None);
                Some((id, (None, tree)))
            }
            Direction::Right(index) => {
                let dev = self.devices.remove(index);
                let tree = self.children.remove(index);
                Some((dev.as_ref().unwrap().numerical_id, (dev, tree)))
            }
        }
    }

    pub fn find_closest_index(&self, key: KeyType) -> Direction {
        let mut index = Direction::Left;
        for (i, pair) in self.devices.iter().enumerate() {
            if let Some(dev) = pair {
                if dev.numerical_id <= key {
                    index = Direction::Right(i);
                } else {
                    break;
                }
            }
        }
        index
    }

    pub fn get_device(&self, key: KeyType) -> Option<&IoTDevice> {
        let mut result = None;
        for d in self.devices.iter() {
            if let Some(device) = d {
                if device.numerical_id == key {
                    result = Some(device);
                    break;
                }
            }
        }
        result
    }

    pub fn get_child(&self, key: KeyType) -> Option<&Tree> {
        match self.find_closest_index(key) {
            Direction::Left => self.left_child.as_ref(),
            Direction::Right(i) => self.children[i].as_ref(),
        }
    }
}

pub struct DeviceDatabase {
    root: Option<Tree>,
    order: usize,
    pub length: u64,
}

impl DeviceDatabase {
    pub fn new_empty(order: usize) -> DeviceDatabase {
        DeviceDatabase {
            root: None,
            length: 0,
            order: order,
        }
    }

    pub fn add(&mut self, device: IoTDevice) {
        let node = if self.root.is_some() {
            mem::replace(&mut self.root, None).unwrap()
        } else {
            Node::new_leaf()
        };

        let (root, _) = self.add_r(node, device, true);

        self.root = Some(root);
    }

    fn add_r(&mut self, node: Tree, device: IoTDevice, is_root: bool) -> (Tree, Option<Data>) {
        let mut node = node;
        let id = device.numerical_id;

        match node.node_type {
            NodeType::Leaf => {
                if node.add_key(id, (Some(device), None)) {
                    self.length += 1;
                }
            }
            NodeType::Regular => {
                let (key, (dev, tree)) = node.remove_key(id).unwrap();
                let new = self.add_r(tree.unwrap(), device, false);
                if dev.is_none() {
                    node.add_left_child(Some(new.0));
                } else {
                    node.add_key(key, (dev, Some(new.0)));
                }
                if let Some(split_result) = new.1 {
                    let new_id = &split_result.0.clone().unwrap();
                    node.add_key(new_id.numerical_id, split_result);
                }
            }
        }

        if node.len() > self.order {
            let (new_parent, sibling) = node.split();

            // Check if the root node is "full" and add a new level
            if is_root {
                let mut parent = Node::new_regular();
                // Add the former root to the left
                parent.add_left_child(Some(node));
                // Add the new right part as well
                parent.add_key(new_parent.numerical_id, (Some(new_parent), Some(sibling)));
                (parent, None)
            } else {
                (node, Some((Some(new_parent), Some(sibling))))
            }
        } else {
            (node, None)
        }
    }

    pub fn is_a_valid_btree(&self) -> bool {
        if let Some(tree) = self.root.as_ref() {
            let total = self.validate(tree, 0);
            total.0 && total.1 == total.2
        } else {
            false // there is no tree
        }
    }

    fn validate(&self, node: &Tree, level: usize) -> (bool, usize, usize) {
        //node.print(format!("Level: {}", level));
        match node.node_type {
            NodeType::Leaf => (node.len() <= self.order, level, level),
            NodeType::Regular => {
                // Root node only requires two children, every other node at least half the
                // order
                let min_children = if level > 0 { self.order / 2usize } else { 2 };
                let key_rules = node.len() <= self.order && node.len() >= min_children;

                let mut total = (key_rules, usize::max_value(), level);
                for n in node.children.iter().chain(vec![&node.left_child]) {
                    if let Some(ref tree) = n {
                        let stats = self.validate(tree, level + 1);
                        total = (
                            total.0 && stats.0,
                            cmp::min(stats.1, total.1),
                            cmp::max(stats.2, total.2),
                        );
                    }
                }
                total
            }
        }
    }

    pub fn find(&self, id: KeyType) -> Option<IoTDevice> {
        match self.root.as_ref() {
            Some(tree) => self.find_r(tree, id),
            _ => None,
        }
    }

    fn find_r(&self, node: &Tree, id: KeyType) -> Option<IoTDevice> {
        match node.get_device(id) {
            Some(device) => Some(device.clone()),
            None if node.node_type != NodeType::Leaf => {
                if let Some(tree) = node.get_child(id) {
                    self.find_r(tree, id)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn walk(&self, callback: impl Fn(&IoTDevice) -> ()) {
        if let Some(ref root) = self.root {
            self.walk_in_order(root, &callback);
        }
    }

    fn walk_in_order(&self, node: &Tree, callback: &impl Fn(&IoTDevice) -> ()) {
        if let Some(ref left) = node.left_child {
            self.walk_in_order(left, callback);
        }

        for i in 0..node.devices.len() {
            if let Some(ref k) = node.devices[i] {
                callback(k);
            }

            if let Some(ref c) = node.children[i] {
                self.walk_in_order(&c, callback);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use rand::Rng;
    use std::cell::RefCell;

    fn new_device_with_id(id: u64) -> IoTDevice {
        new_device_with_id_path(id, "")
    }

    fn new_device_with_id_path(id: u64, path: impl Into<String>) -> IoTDevice {
        IoTDevice::new(id, format!("My address is {}", id), path)
    }

    #[test]
    fn btree_add() {
        let mut tree = DeviceDatabase::new_empty(3);
        tree.add(new_device_with_id(0));
        tree.add(new_device_with_id(2));
        tree.add(new_device_with_id(4));
        tree.add(new_device_with_id(3));
        tree.add(new_device_with_id(5));
        tree.add(new_device_with_id(6));
        tree.add(new_device_with_id(7));

        assert_eq!(tree.length, 7);
        assert!(tree.is_a_valid_btree());
    }

    #[test]
    fn btree_walk_in_order() {
        let len = 7;

        let mut tree = DeviceDatabase::new_empty(3);
        let mut items: Vec<IoTDevice> = (0..len).map(new_device_with_id).collect();

        let mut rng = thread_rng();
        rng.shuffle(&mut items);

        for item in items.iter() {
            tree.add(item.clone());
        }
        assert!(tree.is_a_valid_btree());
        assert_eq!(tree.length, len);
        let v: RefCell<Vec<IoTDevice>> = RefCell::new(vec![]);
        tree.walk(|n| v.borrow_mut().push(n.clone()));
        let mut items = items;
        // sort in descending order:
        items.sort_by(|a, b| a.numerical_id.cmp(&b.numerical_id));
        assert_eq!(v.into_inner(), items)
    }

    #[test]
    fn btree_find() {
        let mut tree = DeviceDatabase::new_empty(3);

        tree.add(new_device_with_id(3));
        tree.add(new_device_with_id(2));
        tree.add(new_device_with_id(1));
        tree.add(new_device_with_id(6));
        tree.add(new_device_with_id(4));
        tree.add(new_device_with_id(5));
        tree.add(new_device_with_id(7));

        assert!(tree.is_a_valid_btree());
        assert_eq!(tree.length, 7);

        assert_eq!(tree.find(100), None);
        assert_eq!(tree.find(4), Some(new_device_with_id(4)));
        assert_eq!(tree.find(3), Some(new_device_with_id(3)));
        assert_eq!(tree.find(2), Some(new_device_with_id(2)));
        assert_eq!(tree.find(1), Some(new_device_with_id(1)));
        assert_eq!(tree.find(5), Some(new_device_with_id(5)));
        assert_eq!(tree.find(6), Some(new_device_with_id(6)));
        assert_eq!(tree.find(7), Some(new_device_with_id(7)));
    }
}
