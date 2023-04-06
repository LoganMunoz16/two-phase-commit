# Rust - Two-Phased Commit Linked List

## Operation
This repository contains a single source code file. As such, the recommended way of compiling and running this program (at least in an Ubuntu environment) is pasted below.

First, compile the rust source file using rustc:
```
rustc two-phase-commit -o test
```

Second, run the "test" executable you just created:
```
./test
```

## Brief Description

The following is a brief description of the structure of this linked list and how the two-phased commit is implemented:

### Nodes
The nodes are essentially the same as a normal linked list, except just with some added Rust code to work with the "pointers" in Rust. The following is a list and description of the contents of a node:

* data: The data (in this case a string) to be added to the list
* next: The next node in the list, wrapped in first a Box, and then an Option

### Linked List
The linked list itself is modified a bit to allow for the commit and rollback interface. The following is a list and description of the contents of a linked list:

* _size_saved : The size(length) of the most recently saved linked list
* _size: The size(length) of the linked list the user is currently modifying
* _num_ops: The number of **attempted** operations in between commits
* _num_completed: The number of **completed** operations in between commits
* _head: The head node for this linked list
* _saved_list: Another nested linked list containing all the saved values

### Commit Implementation
The first phase of a two-phase commit usually checks that all services are prepared and able to commit. In our case, this would mean that the number of attempted operations (_num_ops), is the same as the number of completed operations (_num_completed). Should this comparison fail, then one of our operations must have failed, meaning that we need to rollback to the last saved state of our list.

The second phase of a two-phase commit then usually gives the go-ahead to all operations to make their commits. In our case, that means we have to save our newly created linked list. We do this by simply replacing our saved linked list with the one we have modified.

## Future Work
The primary concern I have for this implementation is the save mechanism. I personally think that in the second phase, there is a cleaner and less memory intense method of savind your edited array. Additionally, there could be issues if the code breaks somewhere when saving to the internal linked list. If this happens we would need a separate rollback function to try and fix those errors as well. However, I still think this implementation is a good demonstration of a two-phase commit on a small scale for a Rust linked list.

If I were to do this project again, I would try to change the implementation to only do the Adds and Deletes on the saved list instead of copying the entire list over. This would likely require another data structure to track the operations done by the user, which I could then use to re-perform those operations on the saved list. This would also allow me to use the edited list as a pseudo-saved list should those operations fail when trying to save to the actual saved list.
