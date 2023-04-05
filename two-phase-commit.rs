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

/*
* Node
* Desc: A simple Node struct used in a Linked List
* Attributes:
*   "data" = the string data contained in the node
*   "next" = An option class containing a Boxed Node, 
*                which is a reference to the next ndoe in the list
 */
struct Node<'a> {
    data: &'a str,
    next: Option<Box<Node<'a>>>,
}


/*
* LinkedList
* Desc: A Linked List struct designed to enable two-phase commits and rollbacks
* Attributes:
*   "_size_saved" = the size of the saved (committed) linked list
*   "_size" = the size of the unsaved linked list a user is editing
*   "_num_ops" = a counter for the number of operations *attempted* in between commits
*   "_num_completed" = a counter for the number of *completed* operations in between commits
*   "_head" = the head node for this linked list
*   "_saved_list" = a saved linked list that is only able to be updated by a commit.
*                   This is also what is rolled back to if need be
 */
struct LinkedList<'a> {
    _size_saved: usize,
    _size: usize,
    _num_ops: usize,
    _num_completed: usize,
    _head: Option<Box<Node<'a>>>,
    _saved_list: Option<Box<LinkedList<'a>>>
}

//The following contains the methods of the LinkedList struct
impl<'a> LinkedList<'a> {

    /*
    * new
    * Desc: Initializes a new LinkedList automatically (all values are 0, None, or Some in the case of the saved list)
    * @param: None
    * @return: the initialized LinkedList
    */
    fn new() -> LinkedList<'a> {
        LinkedList {
            _size_saved: 0,
            _size: 0,
            _num_ops: 0,
            _num_completed: 0,
            _head: None,
            _saved_list: Some(Box::new(LinkedList {
                _size_saved: 0,
                _size: 0,
                _num_ops: 0,
                _num_completed: 0,
                _head: None,
                _saved_list: None,
            }))
        }
    }

    /*
    * Add
    * Desc: Adds a node with the given string, at the given position.
    *   NOTE: This only updates the non-saved list to allow for rollbacks
    * @param: 
    *   self - a reference to this object instance (not needed in interface)
    *   iKey - the desired position to add the node
    *   string - the data contained within the node
    * @return: None
    */
    fn Add(&mut self, iKey: usize, string: &'a str) {
        
        if iKey > self._size {
            return;
        }

        self._num_ops += 1;

        let mut new_node = Node {
            data: string,
            next: None,
        };

        if self._size == 0 && iKey == 0 {
            self._head = Some(Box::new(new_node));
            self._size += 1;
            self._num_completed += 1;
            return;
        }

        if iKey == 0 {
            new_node.next = self._head.take();
            self._head = Some(Box::new(new_node));
            self._size += 1;
            self._num_completed += 1;
            return;
        }

        let mut node = self._head.as_mut().unwrap();
        let mut i = 1;

        while i < iKey && node.next.is_some() {
            node = node.next.as_mut().unwrap();
            i += 1;
        }

        new_node.next = node.next.take();

        node.next = Some(Box::new(new_node));
        self._size += 1;
        self._num_completed += 1;
        println!("Node \"{}\" added at position {}", string, iKey);
    }


    /*
    * Delete
    * Desc: Deletes a node at the given position.
    *   NOTE: This only updates the non-saved list to allow for rollbacks
    * @param: 
    *   self - a reference to this object instance (not needed in interface)
    *   iKey - the desired position to add the node
    * @return: None
    */
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
            self._head = self._head.as_mut().unwrap().next.take();
            self._size += 1;
            self._num_completed += 1;
            return;
        }

        let mut node = self._head.as_mut().unwrap();
        let mut i = 1;

        while i < iKey && node.next.is_some() {
            node = node.next.as_mut().unwrap();
            i += 1;
        }
        if node.next.as_mut().unwrap().next.is_some() {
            node.next = node.next.as_mut().unwrap().next.take();
            drop(node.next.as_mut().unwrap());
        } else {
            drop(node.next.as_mut().unwrap());
            node.next = None;

        }

        self._size -= 1;
        self._num_completed += 1;
        println!("Node at position {} deleted", iKey);
    }


    /*
    * Commit
    * Desc: Saves the edited linked list into the _saved_list attribute.
    *   NOTE: This has two stages - 
    *       FIRST: We check that all attempted operations are completed (i.e. all services are ready to commit)
    *               If this fails, then we had an error and we rollback to the last stable/saved state.
    *       SECOND: If all operations were performed correctly, we save the edited list over top of the old saved list
    *               After that, we reset all counters to prepare for the next batch of operations
    * @param: 
    *   self - a reference to this object instance (not needed in interface)
    * @return: None
    */
    fn Commit(&mut self) {
        if self._num_ops != self._num_completed {
            println!("Error: An Add or Delete operation has failed. Rolling back to last stable state.");
            self.Rollback();
            return;
        } else {
            let mut new_node = self._head.as_mut().unwrap();
            let mut new_head = Node {
                    data: new_node.data,
                    next: None,
            };
            self._saved_list.as_mut().unwrap()._head = Some(Box::new(new_head));
            let mut saved_node = self._saved_list.as_mut().unwrap()._head.as_mut().unwrap();

            let mut i = 0;

            while i < self._size && new_node.next.is_some() {
                new_node = new_node.next.as_mut().unwrap();
                let mut temp = Node {
                    data: new_node.data,
                    next: None,
                };
                saved_node.next = Some(Box::new(temp));
                saved_node = saved_node.next.as_mut().unwrap();
                i += 1;
            }
            self._size_saved = self._size;
            self._num_completed = 0;
            self._num_ops = 0;
            println!("Your commit has been saved successfully");
        }
    }

    /*
    * Rollback
    * Desc: Adds a node with the given string, at the given position.
    *   NOTE: This only updates the non-saved list to allow for rollbacks
    * @param: 
    *   self - a reference to this object instance (not needed in interface)
    *   iKey - the desired position to add the node
    *   string - the data contained within the node
    * @return: None
    */
    fn Rollback(&mut self) {
        let mut stored_node = self._saved_list.as_mut().unwrap()._head.as_mut().unwrap();
        let mut stored_head = Node {
                data: stored_node.data,
                next: None,
        };
        self._head = Some(Box::new(stored_head));
        let mut node = self._head.as_mut().unwrap();

        let mut i = 0;

        while i < self._size_saved && stored_node.next.is_some() {
            stored_node = stored_node.next.as_mut().unwrap();
            let mut temp = Node {
                data: stored_node.data,
                next: None,
            };
            node.next = Some(Box::new(temp));
            node = node.next.as_mut().unwrap();
            i += 1;
        }
        self._size = self._size_saved;
        self._num_completed = 0;
        self._num_ops = 0;
    }


    fn ToStringSaved(&mut self) {
        println!("Saved List: ");
        if self._saved_list.as_mut().unwrap()._head.is_none() {
            return;
        }

        let mut iterator = self._saved_list.as_mut().unwrap()._head.as_mut().unwrap();

        let mut i = 0;

            while i < self._size_saved && iterator.next.is_some() {
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

            while i < self._size && iterator.next.is_some() {
                println!("{}", iterator.data);
                iterator = iterator.next.as_mut().unwrap();
                i += 1;
            }
            println!("{}", iterator.data);
    }
}


fn main() {
    let mut my_list = LinkedList::new();

    my_list.Add(0, "This is node number 1");
    my_list.Add(1, "This is node number 2");
    my_list.Add(1, "This is node number 3");
    my_list.Commit();
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    my_list.Add(0, "This is node number 0");
    my_list.Add(1, "This is node number 0.5");
    my_list.Add(5, "This is node number 5");
    my_list.Delete(2);
    my_list.ToStringSaved();
    my_list.ToStringEdited();

    my_list.Commit();
    my_list.ToStringSaved();
    my_list.ToStringEdited();

}
