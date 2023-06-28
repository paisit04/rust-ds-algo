use std::boxed::Box;
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

#[derive(Clone, Debug)]
pub struct MessageNotification {
    pub no_messages: u64,
    pub device: IoTDevice,
}

impl MessageNotification {
    pub fn new(device: IoTDevice, no_messages: u64) -> MessageNotification {
        MessageNotification {
            no_messages: no_messages,
            device: device,
        }
    }
}

impl PartialEq for MessageNotification {
    fn eq(&self, other: &MessageNotification) -> bool {
        self.device.eq(&other.device) && self.no_messages == other.no_messages
    }
}

pub struct MessageChecker {
    pub length: usize,
    heap: Vec<Box<MessageNotification>>,
}

impl MessageChecker {
    pub fn new_empty() -> MessageChecker {
        MessageChecker {
            length: 0,
            heap: vec![],
        }
    }

    fn swap(&mut self, pos1: usize, pos2: usize) {
        let m2 = self.heap[pos1 - 1].clone();
        self.heap[pos1 - 1] = mem::replace(&mut self.heap[pos2 - 1], m2);
    }

    fn has_more_messages(&self, pos1: usize, pos2: usize) -> bool {
        let a = &self.heap[pos1 - 1];
        let b = &self.heap[pos2 - 1];
        a.no_messages >= b.no_messages
    }

    pub fn add(&mut self, notification: MessageNotification) {
        self.heap.push(Box::new(notification));
        self.length = self.heap.len();

        if self.length > 1 {
            let mut i = self.length;
            while i / 2 > 0 && self.has_more_messages(i, i / 2) {
                self.swap(i, i / 2);
                i /= 2;
            }
        }
    }

    pub fn pop(&mut self) -> Option<MessageNotification> {
        if self.length > 0 {
            let elem = self.heap.swap_remove(0);
            self.length = self.heap.len();
            let mut i = 1;
            while i * 2 < self.length {
                let children = (i * 2, i * 2 + 1);
                i = if self.has_more_messages(children.0, children.1) {
                    if self.has_more_messages(children.0, i) {
                        self.swap(i, children.0);
                        children.0
                    } else {
                        break;
                    }
                } else {
                    if self.has_more_messages(children.1, i) {
                        self.swap(i, children.1);
                        children.1
                    } else {
                        break;
                    }
                }
            }
            Some(*elem)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use rand::Rng;
    use std::collections::HashMap;

    fn new_device_with_id(id: u64) -> IoTDevice {
        let mut scores = HashMap::new();

        scores.insert(String::from("Blue"), 10);
        scores.insert(String::from("Yellow"), 50);

        let team_name = String::from("Blue");
        let score = scores.get(&team_name).copied().unwrap_or(0);

        new_device_with_id_path(id, "")
    }

    fn new_device_with_id_path(id: u64, path: impl Into<String>) -> IoTDevice {
        IoTDevice::new(id, format!("My address is {}", id), path)
    }

    fn new_notification_with_id(id: u64, no_messages: u64) -> MessageNotification {
        let dev = new_device_with_id(id);
        MessageNotification::new(dev, no_messages)
    }

    #[test]
    fn binary_heap_add() {
        let mut heap = MessageChecker::new_empty();

        heap.add(new_notification_with_id(1, 100));
        heap.add(new_notification_with_id(2, 200));
        heap.add(new_notification_with_id(3, 500));
        heap.add(new_notification_with_id(4, 40));
        assert_eq!(heap.length, 4);
    }

    #[test]
    fn binary_heap_pop() {
        let mut heap = MessageChecker::new_empty();

        let a = new_notification_with_id(1, 40);
        let b = new_notification_with_id(2, 300);
        let c = new_notification_with_id(3, 50);
        let d = new_notification_with_id(4, 500);

        heap.add(a.clone());
        heap.add(b.clone());
        heap.add(c.clone());
        heap.add(d.clone());

        assert_eq!(heap.length, 4);

        assert_eq!(heap.pop(), Some(d));
        assert_eq!(heap.pop(), Some(b));
        assert_eq!(heap.pop(), Some(c));
        assert_eq!(heap.pop(), Some(a));
    }
}
