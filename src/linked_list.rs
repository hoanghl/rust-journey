use log;

///////////////////////////////////////////////////
// Node
///////////////////////////////////////////////////

pub struct Node<T> {
    pub val: Option<T>,
    pub next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Node<T> {
        Node {
            val: Some(value),
            next: None,
        }
    }
}

#[allow(dead_code)]
impl Node<i32> {
    pub fn print(&self) {
        if self.val.is_none() {
            return;
        }

        println!("Node: val = {:?}", self.val.unwrap())
    }
}

///////////////////////////////////////////////////
// Linked list
///////////////////////////////////////////////////

pub struct LinkedList1<T> {
    pub head: Option<Box<Node<T>>>,
}

#[allow(dead_code)]
impl<T> LinkedList1<T> {
    pub fn insert(&mut self, val: T) {
        let mut new_node = Some(Box::new(Node::<T>::new(val)));
        if self.head.is_none() {
            self.head = new_node;
        } else {
            new_node.as_mut().unwrap().next = self.head.take();
            self.head = new_node;
        }
    }

    pub fn new() -> LinkedList1<T> {
        LinkedList1 { head: None }
    }
}

#[allow(dead_code)]
impl LinkedList1<i32> {
    pub fn print(&self) {
        if self.head.is_none() {
            log::info!("Head is None");
        } else {
            println!("head: value = {:?}", self.head.as_ref().unwrap().val);
        }
    }

    pub fn traverse(&self) {
        let mut p = &self.head;
        if !p.is_none() {
            loop {
                println!("value: {:?}", p.as_ref().unwrap().val);
                p = &p.as_ref().unwrap().next;

                if p.is_none() {
                    break;
                }
            }
        }
    }
}
