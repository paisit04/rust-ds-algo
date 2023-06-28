use std::boxed::Box;
use std::collections::HashMap;
use std::mem;
use std::str::Chars;

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

type Link = Box<Node>;

struct Node {
    pub key: char,
    next: HashMap<char, Link>,
    pub value: Option<IoTDevice>,
}

impl Node {
    pub fn new(key: char, device: Option<IoTDevice>) -> Link {
        Box::new(Node {
            key: key,
            next: HashMap::new(),
            value: device,
        })
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.key == other.key
    }
}

pub struct BestDeviceRegistry {
    pub length: u64,
    root: HashMap<char, Link>,
}

impl BestDeviceRegistry {
    pub fn new_empty() -> BestDeviceRegistry {
        BestDeviceRegistry {
            length: 0,
            root: HashMap::new(),
        }
    }

    pub fn add(&mut self, device: IoTDevice) {
        let p = device.path.clone();
        let mut path = p.chars();

        if let Some(start) = path.next() {
            self.length += 1;
            let mut n = self.root.entry(start).or_insert(Node::new(start, None));
            for c in path {
                let tmp = n.next.entry(c).or_insert(Node::new(c, None));
                n = tmp;
            }
            n.value = Some(device);
        }
    }

    pub fn find(&self, path: &str) -> Option<IoTDevice> {
        let mut path = path.chars();

        if let Some(start) = path.next() {
            self.root.get(&start).map_or(None, |mut n| {
                for c in path {
                    match n.next.get(&c) {
                        Some(ref tmp) => n = tmp,
                        None => break,
                    }
                }
                n.value.clone()
            })
        } else {
            None
        }
    }

    pub fn walk(&self, callback: impl Fn(&IoTDevice) -> ()) {
        for r in self.root.values() {
            self.walk_r(&r, &callback);
        }
    }

    fn walk_r(&self, node: &Link, callback: &impl Fn(&IoTDevice) -> ()) {
        for n in node.next.values() {
            self.walk_r(&n, callback);
        }
        if let Some(ref dev) = node.value {
            callback(dev);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use rand::Rng;
    use std::cell::RefCell;

    const LIST_ITEMS: u64 = 1_000;

    fn new_device_with_id(id: u64) -> IoTDevice {
        new_device_with_id_path(id, "")
    }

    fn new_device_with_id_path(id: u64, path: impl Into<String>) -> IoTDevice {
        IoTDevice::new(id, format!("My address is {}", id), path)
    }

    #[test]
    fn trie_add() {
        let mut trie = BestDeviceRegistry::new_empty();
        let len = 10;

        let mut rng = thread_rng();

        for i in 0..len {
            trie.add(new_device_with_id_path(
                i,
                format!("factory{}/machineA/{}", rng.gen_range(0, len), i),
            ));
        }

        assert_eq!(trie.length, len);
    }

    #[test]
    fn trie_walk_in_order() {
        let mut trie = BestDeviceRegistry::new_empty();
        let len = 10;

        let mut rng = thread_rng();
        let items: Vec<IoTDevice> = (0..len)
            .map(|i| {
                new_device_with_id_path(
                    i,
                    format!("factory{}/machineA/{}", rng.gen_range(0, len), i),
                )
            })
            .collect();

        for item in items.iter() {
            trie.add(item.clone());
        }
        assert_eq!(trie.length, len);
        let v: RefCell<Vec<IoTDevice>> = RefCell::new(vec![]);
        trie.walk(|n| v.borrow_mut().push(n.clone()));
        let mut items = items;
        // sort in descending order:
        items.sort_by(|a, b| b.numerical_id.cmp(&a.numerical_id));
        let mut actual = v.into_inner();
        actual.sort_by(|a, b| b.numerical_id.cmp(&a.numerical_id));
        assert_eq!(actual, items)
    }

    #[test]
    fn trie_find() {
        let mut trie = BestDeviceRegistry::new_empty();
        let len = 10;

        let mut rng = thread_rng();
        let mut paths = vec![];
        for i in 0..len {
            let s = format!("factory{}/machineA/{}", rng.gen_range(0, len), i);
            trie.add(new_device_with_id_path(i, s.clone()));
            paths.push(s);
        }

        assert_eq!(trie.length, len);
        assert_eq!(trie.find("100"), None);
    }
}
