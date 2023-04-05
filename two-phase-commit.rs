//Unable to get linked list working in rust... but here was my implementation plan:
//Have nodes that also include a "changed" and "is_deleted" field
//The Add/Delete will do the usual operations, but only on the "changed" field
//  This way, the "next" field is maintained, so if a rollback is needed we know what the list looked like.
//  Additionally, if one is to be deleted, that boolean will be set to true.
//  After doing this operation, the function will add a value to a global counter
//1st phase commit
//  We check that counter, and if it is 0, that means no issues happened
//  Otherwise, we rollback to what he had before and all is well
//2nd phase commit
//  Since everything went alright, now we just sweep through and do two things
//  First, we delete any nodes marked for deletion
//  Second, we copy all "changed" pointers to the "next" pointers, and update _size_initial
//  And our list is now saved!

struct Node<'a> {
    data: &'a str,
    next: Option<Box<Node<'a>>>,
    changed: Option<Box<Node<'a>>>,
    is_deleted: bool,
}

struct LinkedList<'a> {
    _size_initial: usize,
    _size: usize,
    _num_ops: usize,
    _num_completed: usize,
    _head: Option<Box<Node<'a>>>,
    _saved_list: Option<Box<LinkedList<'a>>>
}

//Add in fake head node here for consistency
impl<'a> LinkedList<'a> {

    fn new() -> LinkedList<'a> {
        LinkedList {
            _size_initial: 0,
            _size: 0,
            _num_ops: 0,
            _num_completed: 0,
            _head: None,
            _saved_list: Some(Box::new(LinkedList {
                _size_initial: 0,
                _size: 0,
                _num_ops: 0,
                _num_completed: 0,
                _head: None,
                _saved_list: None,
            }))
        }
    }

    fn Add(&mut self, iKey: usize, string: &'a str) {
        
        if iKey > self._size {
            return;
        }

        self._num_ops += 1;

        let mut new_node = Node {
            data: string,
            next: None,
            changed: None,
            is_deleted: false,
        };

        if self._size == 0 && iKey == 0 {
            self._head = Some(Box::new(new_node));
            self._size += 1;
            self._num_completed += 1;
            return;
        }

        if iKey == 0 {
            new_node.changed = self._head.take();
            self._head = Some(Box::new(new_node));
            self._size += 1;
            self._num_completed += 1;
            return;
        }

        let mut node = self._head.as_mut().unwrap();
        let mut i = 1;

        while i < iKey && node.changed.is_some() {
            node = node.next.as_mut().unwrap();
            i += 1;
        }

        new_node.changed = node.changed.take();

        node.changed = Some(Box::new(new_node));
        self._size += 1;
        self._num_completed += 1;
    }

    fn Delete(&mut self, iKey: usize) {
        
        if iKey > self._size {
            return;
        }

        self._num_ops += 1;

        if self._size == 0 && iKey == 0 {
            drop(self._head.as_mut().unwrap());
            self._size -= 1;
            self._num_completed += 1;
            return;
        }

        if iKey == 0 {
            self._head = self._head.as_mut().unwrap().changed.take();
            self._size += 1;
            self._num_completed += 1;
            return;
        }

        let mut node = self._head.as_mut().unwrap();
        let mut i = 1;

        while i < iKey && node.changed.is_some() {
            node = node.changed.as_mut().unwrap();
            i += 1;
        }
        if node.changed.as_mut().unwrap().changed.is_some() {
            node.changed.as_mut().unwrap().is_deleted = true;
            node.changed = node.changed.as_mut().unwrap().changed.take();
            drop(node.changed.as_mut().unwrap());
        } else {
            drop(node.changed.as_mut().unwrap());
            node.changed.as_mut().unwrap().is_deleted = true;
            node.changed = None;

        }

        self._size -= 1;
        self._num_completed += 1;
    }

    fn Commit(&mut self) {
        if self._num_ops != self._num_completed {
            //Call rollback - essentially just making all the "changed" into "None"
            return;
        } else {
            let mut new_node = self._head.as_mut().unwrap();
            let mut new_head = Node {
                    data: new_node.data,
                    next: None,
                    changed: None,
                    is_deleted: false,
            };
            self._saved_list.as_mut().unwrap()._head = Some(Box::new(new_head));
            let mut saved_node = self._saved_list.as_mut().unwrap()._head.as_mut().unwrap();

            let mut i = 0;

            //TODO: Find a way to move over the reference from changed to next
            while i < self._size && new_node.changed.is_some() {
                new_node = new_node.changed.as_mut().unwrap();
                let mut temp = Node {
                    data: new_node.data,
                    next: None,
                    changed: None,
                    is_deleted: false,
                };
                saved_node.next = Some(Box::new(temp));
                saved_node = saved_node.next.as_mut().unwrap();
                i += 1;
            }
            self._size_initial = self._size;
            self._num_completed = 0;
            self._num_ops = 0;
        }
    }


    fn ToStringSaved(&mut self) {
        println!("Saved List: ");
        if self._saved_list.as_mut().unwrap()._head.is_none() {
            return;
        }

        let mut iterator = self._saved_list.as_mut().unwrap()._head.as_mut().unwrap();

        let mut i = 0;

            //TODO: Find a way to move over the reference from changed to next
            while i < self._size_initial && iterator.next.is_some() {
                println!("{}", iterator.data);
                iterator = iterator.next.as_mut().unwrap();
                i += 1;
            }
            println!("{}", iterator.data);
    }

    fn ToStringEdited(&mut self) {
        println!("Edited list: ");
        if self._head.is_none() {
            return;
        }

        let mut iterator = self._head.as_mut().unwrap();

        let mut i = 0;

            //TODO: Find a way to move over the reference from changed to next
            while i < self._size_initial && iterator.changed.is_some() {
                println!("{}", iterator.data);
                iterator = iterator.changed.as_mut().unwrap();
                i += 1;
            }
            println!("{}", iterator.data);
    }
}


fn main() {
    let mut my_list = LinkedList::new();

    my_list.Add(0, "This is node number 1");
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    my_list.Add(1, "This is node number 2");
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    my_list.Add(1, "This is node number 3");
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    my_list.Delete(1);
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    my_list.Commit();
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    my_list.Delete(1);
    my_list.ToStringSaved();
    my_list.ToStringEdited();


}
