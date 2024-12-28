
the allocators crate contains simple memory allocators used in #puddle 
multiple (also different types) of allocators can be created within one memory block
these allocators are made to contain no metadata for allocated memory on the heap
so 100% of the memory allocated can also be used
(except for padding and blocks that are smaller than the minimum size of mostly 8 bytes)

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
	There is a typed version #TypedPoolAllocator that's just the same 
	but with some type safety

3. #FreeListAllocator 
	a dynamic-allocator
	works like the pool allocator but with dynamic sizes
	at the start its just one big block, on allocation this is resized
	on deallocation then merged together (if next to each other)
	
	the layout of the memory may look something like this
	`| data2 |    free    | data3 | data4 |    free    |
	every free block contains some metadata (not the allocated ones)
	1. size of the free space
	2. pointer to the next free node (if there is one)
	memory blocks are returned as a 'FreeListPtr' whats just a pointer 
	that also stores the padding before and after the block needed for deallocation
	
	this also means that a free block has a minimum size of 8 bytes
	this allocator is affected by memory fragmentation
	if you want to minimize fragmentation, consider using another allocator.
	also to improve memory usage the limit of the allocation is ``u32::MAX`` bytes (4.2 GB)
