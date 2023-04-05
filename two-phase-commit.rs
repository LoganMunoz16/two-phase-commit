/* Project: Two-phase commit with a Linked List (in Rust!)
* Author: Logan Munoz
* Date: 4/5/2023
* Brief Description: This Rust Linked List implementation allows for inserting at an index, and deleting at an index.
*                    Upon completing many operations at once, a user can then ask to commit those operations and save
*                    their changes. When committing, the first phase checks if any of those operatiosn failed. If it
*                    finds that one or more did, then the commit aborts and the list is rolled back to the last saved state.
*                    If everything looks good, the edited list is saved for future reference and the user can begin
*                    adding more operations if they wish.
*                   
*                    Please see the function and struct descriptions below for more details on the operation of this
*                    implementation. This was a challenge to learn Rust at the same time, but was definitely a fun
*                    experience, and I feel much more comfortable now! Hopefully this implementation is somewhat close 
*                    to what was asked of us!
 */


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
*                   This is also what is rolled back to if need be.
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

        self._num_ops += 1; //Note an attempted operation

        if iKey > self._size {
            println!("Add failed. Index out of bounds. Please Rollback and try again");
            return;
        }

        let mut new_node = Node {
            data: string,
            next: None,
        };

        //Inserting into an empty list
        if self._size == 0 && iKey == 0 {
            self._head = Some(Box::new(new_node));
            self._size += 1;
            self._num_completed += 1;
            println!("Node \"{}\" added at position {}", string, iKey);
            return;
        }

        //Inserting at head
        if iKey == 0 {
            new_node.next = self._head.take();
            self._head = Some(Box::new(new_node));
            self._size += 1;
            self._num_completed += 1;
            println!("Node \"{}\" added at position {}", string, iKey);
            return;
        }

        //Inserting somewhere NOT at the head, and non-empty list
        let mut node = self._head.as_mut().unwrap();
        let mut i = 1;

        while i < iKey && node.next.is_some() {
            node = node.next.as_mut().unwrap();
            i += 1;
        }

        new_node.next = node.next.take();
        node.next = Some(Box::new(new_node));

        self._size += 1;
        self._num_completed += 1; //Noting that this operation was completed and ready to commit
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

        self._num_ops += 1; //Noting an attempted operation
        
        if iKey > self._size {
            println!("Delete failed. Index out of bounds. Please Rollback and try again");
            return;
        }

        //Deleting the head if empty
        if self._size == 0 && iKey == 0 {
            drop(self._head.as_mut().unwrap());
            self._size -= 1;
            self._num_completed += 1;
            println!("Node at position {} deleted", iKey);
            return;
        }

        //Deleting the head if not empty
        if iKey == 0 {
            self._head = self._head.as_mut().unwrap().next.take();
            self._size -= 1;
            self._num_completed += 1;
            println!("Node at position {} deleted", iKey);
            return;
        }

        //Deleting something that is not the head
        let mut node = self._head.as_mut().unwrap();
        let mut i = 1;

        while i < iKey && node.next.is_some() {
            node = node.next.as_mut().unwrap();
            i += 1;
        }

        //If this node is not at the end
        if node.next.as_mut().unwrap().next.is_some() {
            node.next = node.next.as_mut().unwrap().next.take();
            drop(node.next.as_mut().unwrap());

        //If this node is at the end
        } else {
            drop(node.next.as_mut().unwrap());
            node.next = None;

        }

        self._size -= 1;
        self._num_completed += 1; //Noting that we completed this operation and are ready to commit
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

        //FIRST PHASE
        //  We check to ensure that all operations are ready to commit
        //  This is checked by ensuring the attempted and completed operations have the same count
        if self._num_ops != self._num_completed {
            println!("Error: An Add or Delete operation has failed. Rolling back to last stable state.");
            self.Rollback();
            return;
        
        //SECOND PHASE
        //  We have ensured all operations went smoothly
        //  Thus, now we just have to save our edited list (copying every node over to _saved_list)
        } else {

            //Copy edited head into the saved head
            let mut new_node = self._head.as_mut().unwrap();
            let mut new_head = Node {
                    data: new_node.data,
                    next: None,
            };
            self._saved_list.as_mut().unwrap()._head = Some(Box::new(new_head));
            let mut saved_node = self._saved_list.as_mut().unwrap()._head.as_mut().unwrap();

            let mut i = 0;

            //Copy all other nodes into the saved list
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

            //Resetting values for the saved list to get ready for next round of operations
            self._size_saved = self._size;
            self._num_completed = 0;
            self._num_ops = 0;
            println!("Your commit has been saved successfully");
        }
    }

    /*
    * Rollback
    * Desc: Reverts the edited list back to the last saved vlist
    * @param: 
    *   self - a reference to this object instance (not needed in interface)
    * @return: None
    */
    fn Rollback(&mut self) {

        //Copy the saved head into the edited head
        let mut stored_node = self._saved_list.as_mut().unwrap()._head.as_mut().unwrap();
        let mut stored_head = Node {
                data: stored_node.data,
                next: None,
        };
        self._head = Some(Box::new(stored_head));
        let mut node = self._head.as_mut().unwrap();

        let mut i = 0;

        //Copy all other saved nodes into the edited list
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

        //Reset values of list and prepare for next round of operations
        self._size = self._size_saved;
        self._num_completed = 0;
        self._num_ops = 0;
    }


    /*
    * ToStringSaved
    * Desc: Printed the current saved list
    * @param: 
    *   self - a reference to this object instance (not needed in interface)
    * @return: None
    */
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
            println!("\n");
    }


    /*
    * ToStringEdited
    * Desc: Prints the current edited list
    * @param: 
    *   self - a reference to this object instance (not needed in interface)
    * @return: None
    */
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
            println!("\n");
    }
}


fn main() {
    let mut my_list = LinkedList::new();

    println!("Adding numbers for the first commit\n");
    my_list.Add(0, "This is node number 1");
    my_list.Add(1, "This is node number 2");
    my_list.Add(1, "This is node number 3");
    println!("\nTrying to commit now...\n");
    my_list.Commit();
    println!("\n");
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    println!("\nAdding and deleting some more...\n");
    my_list.Add(0, "This is node number 0");
    my_list.Add(1, "This is node number 0.5");
    my_list.Add(5, "This is node number 5");
    my_list.Delete(2);
    println!("\nChecking progress after our operations...\n");
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    println!("\nTrying to commit again...\n");
    my_list.Commit();
    println!("\n");
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    println!("\nAdding and Deleting one more to test rollback...\n");
    my_list.Add(5, "This is node number 6");
    my_list.Delete(2);
    println!("\nChecking progress after our operations...\n");
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    println!("\nTesting manual rollback...\n");
    my_list.Rollback();
    println!("\nChecking the rollback...\n");
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    println!("\nAdding 2 more, but one causes an error...\n");
    my_list.Add(2, "This one will disappear!");
    my_list.Add(12, "This one will not be seen in the list");
    println!("\nChecking to see our addition...\n");
    my_list.ToStringSaved();
    my_list.ToStringEdited();
    println!("\nAttempting to commit...\n");
    my_list.Commit();
    println!("\nChecking what things look like after the commit\n");
    my_list.ToStringSaved();
    my_list.ToStringEdited();


}
