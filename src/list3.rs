#![allow(dead_code)]

use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self { head: None }
    }

    // give elem (addtl head), return list with elem at head
    // old list remains active
    pub fn prepend(&self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                elem,
                next: self.head.clone(), // increase ref count
            })),
        }
    }

    // return list without head elem
    pub fn tail(&self) -> List<T> {
        List {
            // we use and_then because node.next is a Link (Option/RC/Node)
            // otherwise we'd get Option/Option/RC/Node
            head: self.head.as_ref().and_then(|node| node.next.clone()), // as ref returns
                                                                         // Option<&node>
        }
    }

    pub fn head(&self) -> Option<&T> {
        // we use map because closure returns &T and we want to wrap it in an option
        self.head.as_ref().map(|node| &node.elem)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>, // returns optional ref to next node
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take(); // self.head = None
        while let Some(node) = head {
            if let Ok(mut node) = Rc::try_unwrap(node) {
                // works only if node has only 1 ref
                head = node.next.take(); // next entry in list = None
            } else {
                break; // can partially destruct the list, only unused elems are removed
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        let list = list.tail();
        assert_eq!(list.head(), None);
    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);
        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}
