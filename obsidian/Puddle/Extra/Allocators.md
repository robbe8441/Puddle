
the allocators crate contains simple memory allocators used in #puddle 

1. #StackAllocator
	it manages memory allocations with popping old and pushing new data to a stack
	allocates memory in a FIFO (first in first out) pattern
	downside of this is that the memory cant be freed in any order,
	but in the same order as they are allocated 
	it also allows you to set a 'Maker' in memory
	this can be set before an algorithm runs to bulk free everything allocated by the 
	algorithm
	
	the layout of the memory may look something like this
	`|    data1    | data2 | data3 |   free space |
	this eliminates memory fragmentation and may be used in some algorithms
	
2. #PoolAllocator
	manages memory by creating a big list of 'pools', witch are the same size each
	can be used to store game objects (for example)
	it works like a mix out of a linked list and a vector
	the order of what objects are freed doesn't matter
	
	the layout of the memory may look something like this
	`| data2 | free | data3 | data4 | data1 | free |

3. #FreeListAllocator 
	a dynamic-allocator
	works like the pool allocator but with dynamic sizes
	at the start its just one big block, on allocation this is resized
	on deallocation then merged together (if next to each other)
	
	the layout of the memory may look something like this
	`| data2 |    free    | data3 | data4 |    free    |
	every free block contains some metadata
	1. size of the free space
	2. pointer to the next free node (if there is one)
	this also means that a free block has a minimum size of 16 bytes


