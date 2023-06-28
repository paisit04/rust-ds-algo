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

type Tree = Option<Box<Node>>;
struct Node {
    pub dev: IoTDevice,
    left: Tree,
    right: Tree,
}

impl Node {
    pub fn new(dev: IoTDevice) -> Tree {
        Some(Box::new(Node {
            dev: dev,
            left: None,
            right: None,
        }))
    }
}

pub struct DeviceRegistry {
    root: Tree,
    pub length: u64,
}

impl DeviceRegistry {
    pub fn new_empty() -> DeviceRegistry {
        DeviceRegistry {
            root: None,
            length: 0,
        }
    }

    pub fn add(&mut self, device: IoTDevice) {
        self.length += 1;
        let root = mem::replace(&mut self.root, None);
        self.root = self.add_rec(root, device);
    }

    fn add_rec(&mut self, node: Tree, device: IoTDevice) -> Tree {
        match node {
            Some(mut n) => {
                if n.dev.numerical_id <= device.numerical_id {
                    n.left = self.add_rec(n.left, device);
                } else {
                    n.right = self.add_rec(n.right, device);
                }
                Some(n)
            }
            _ => Node::new(device),
        }
    }

    pub fn find(&self, numerical_id: u64) -> Option<IoTDevice> {
        self.find_r(&self.root, numerical_id)
    }

    fn find_r(&self, node: &Tree, numerical_id: u64) -> Option<IoTDevice> {
        match node {
            Some(n) => {
                if n.dev.numerical_id == numerical_id {
                    Some(n.dev.clone())
                } else if n.dev.numerical_id < numerical_id {
                    self.find_r(&n.left, numerical_id)
                } else {
                    self.find_r(&n.right, numerical_id)
                }
            }
            _ => None,
        }
    }

    pub fn walk(&self, callback: impl Fn(&IoTDevice) -> ()) {
        self.walk_in_order(&self.root, &callback);
    }

    fn walk_in_order(&self, node: &Tree, callback: &impl Fn(&IoTDevice) -> ()) {
        if let Some(n) = node {
            self.walk_in_order(&n.left, callback);
            callback(&n.dev);
            self.walk_in_order(&n.right, callback);
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
    fn binary_search_tree_add() {
        let mut tree = DeviceRegistry::new_empty();
        tree.add(new_device_with_id(4));
        tree.add(new_device_with_id(3));
        tree.add(new_device_with_id(2));
        tree.add(new_device_with_id(1));
        tree.add(new_device_with_id(5));
        tree.add(new_device_with_id(6));
        tree.add(new_device_with_id(7));
        assert_eq!(tree.length, 7);
    }

    #[test]
    fn binary_search_tree_find() {
        let mut tree = DeviceRegistry::new_empty();

        tree.add(new_device_with_id(4));
        tree.add(new_device_with_id(3));
        tree.add(new_device_with_id(2));
        tree.add(new_device_with_id(1));
        tree.add(new_device_with_id(5));
        tree.add(new_device_with_id(6));
        tree.add(new_device_with_id(7));
        assert_eq!(tree.find(100), None);
        assert_eq!(tree.find(4), Some(new_device_with_id(4)));
        assert_eq!(tree.find(3), Some(new_device_with_id(3)));
        assert_eq!(tree.find(2), Some(new_device_with_id(2)));
        assert_eq!(tree.find(1), Some(new_device_with_id(1)));
        assert_eq!(tree.find(5), Some(new_device_with_id(5)));
        assert_eq!(tree.find(6), Some(new_device_with_id(6)));
        assert_eq!(tree.find(7), Some(new_device_with_id(7)));
        assert_eq!(tree.length, 7);
    }

    #[test]
    fn binary_search_tree_walk_in_order() {
        let len = 10;

        let mut tree = DeviceRegistry::new_empty();
        let mut items: Vec<IoTDevice> = (0..len).map(new_device_with_id).collect();

        let mut rng = thread_rng();
        rng.shuffle(&mut items);

        for item in items.iter() {
            tree.add(item.clone());
        }

        assert_eq!(tree.length, len);
        let v: RefCell<Vec<IoTDevice>> = RefCell::new(vec![]);
        tree.walk(|n| v.borrow_mut().push(n.clone()));
        let mut items = items;
        // sort in descending order:
        items.sort_by(|a, b| b.numerical_id.cmp(&a.numerical_id));
        assert_eq!(v.into_inner(), items)
    }
}
