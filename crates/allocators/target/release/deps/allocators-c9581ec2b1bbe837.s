	.text
	.intel_syntax noprefix
	.file	"allocators.83699119ac4a3a0-cgu.0"
	.section	.text._ZN10allocators5stack14StackAllocator3new17hc1c0f144334cd1e1E,"ax",@progbits
	.globl	_ZN10allocators5stack14StackAllocator3new17hc1c0f144334cd1e1E
	.p2align	4, 0x90
	.type	_ZN10allocators5stack14StackAllocator3new17hc1c0f144334cd1e1E,@function
_ZN10allocators5stack14StackAllocator3new17hc1c0f144334cd1e1E:
	.cfi_startproc
	push	r14
	.cfi_def_cfa_offset 16
	push	rbx
	.cfi_def_cfa_offset 24
	push	rax
	.cfi_def_cfa_offset 32
	.cfi_offset rbx, -24
	.cfi_offset r14, -16
	mov	rbx, rsi
	mov	r14, rdi
	mov	rax, qword ptr [rip + __rust_no_alloc_shim_is_unstable@GOTPCREL]
	movzx	eax, byte ptr [rax]
	mov	esi, 1
	mov	rdi, rbx
	call	qword ptr [rip + __rust_alloc@GOTPCREL]
	mov	qword ptr [r14], rax
	mov	qword ptr [r14 + 8], rbx
	mov	qword ptr [r14 + 16], 0
	mov	rax, r14
	add	rsp, 8
	.cfi_def_cfa_offset 24
	pop	rbx
	.cfi_def_cfa_offset 16
	pop	r14
	.cfi_def_cfa_offset 8
	ret
.Lfunc_end0:
	.size	_ZN10allocators5stack14StackAllocator3new17hc1c0f144334cd1e1E, .Lfunc_end0-_ZN10allocators5stack14StackAllocator3new17hc1c0f144334cd1e1E
	.cfi_endproc

	.section	.text._ZN10allocators5stack14StackAllocator8allocate17h75e2c31672bbc85aE,"ax",@progbits
	.globl	_ZN10allocators5stack14StackAllocator8allocate17h75e2c31672bbc85aE
	.p2align	4, 0x90
	.type	_ZN10allocators5stack14StackAllocator8allocate17h75e2c31672bbc85aE,@function
_ZN10allocators5stack14StackAllocator8allocate17h75e2c31672bbc85aE:
	.cfi_startproc
	push	r15
	.cfi_def_cfa_offset 16
	push	r14
	.cfi_def_cfa_offset 24
	push	rbx
	.cfi_def_cfa_offset 32
	.cfi_offset rbx, -32
	.cfi_offset r14, -24
	.cfi_offset r15, -16
	mov	r14, rdx
	mov	rbx, rsi
	mov	r15, rdi
	mov	rax, qword ptr [rdi]
	mov	rcx, qword ptr [rdi + 16]
	lea	rdx, [rax + rcx]
	dec	rsi
	and	rsi, rdx
	sub	rbx, rsi
	test	rsi, rsi
	cmove	rbx, rsi
	mov	rsi, qword ptr [rdi + 8]
	add	rbx, rcx
	add	r14, rbx
	cmp	r14, rsi
	jbe	.LBB1_2
	mov	edx, 1
	mov	rdi, rax
	mov	rcx, r14
	call	qword ptr [rip + __rust_realloc@GOTPCREL]
	mov	qword ptr [r15], rax
	mov	qword ptr [r15 + 8], r14
.LBB1_2:
	mov	qword ptr [r15 + 16], r14
	add	rax, rbx
	pop	rbx
	.cfi_def_cfa_offset 24
	pop	r14
	.cfi_def_cfa_offset 16
	pop	r15
	.cfi_def_cfa_offset 8
	ret
.Lfunc_end1:
	.size	_ZN10allocators5stack14StackAllocator8allocate17h75e2c31672bbc85aE, .Lfunc_end1-_ZN10allocators5stack14StackAllocator8allocate17h75e2c31672bbc85aE
	.cfi_endproc

	.section	".text._ZN75_$LT$allocators..stack..StackAllocator$u20$as$u20$core..ops..drop..Drop$GT$4drop17hedc05e17ce8bbd69E","ax",@progbits
	.globl	_ZN75_$LT$allocators..stack..StackAllocator$u20$as$u20$core..ops..drop..Drop$GT$4drop17hedc05e17ce8bbd69E
	.p2align	4, 0x90
	.type	_ZN75_$LT$allocators..stack..StackAllocator$u20$as$u20$core..ops..drop..Drop$GT$4drop17hedc05e17ce8bbd69E,@function
_ZN75_$LT$allocators..stack..StackAllocator$u20$as$u20$core..ops..drop..Drop$GT$4drop17hedc05e17ce8bbd69E:
	.cfi_startproc
	mov	rax, qword ptr [rdi]
	mov	rsi, qword ptr [rdi + 8]
	mov	edx, 1
	mov	rdi, rax
	jmp	qword ptr [rip + __rust_dealloc@GOTPCREL]
.Lfunc_end2:
	.size	_ZN75_$LT$allocators..stack..StackAllocator$u20$as$u20$core..ops..drop..Drop$GT$4drop17hedc05e17ce8bbd69E, .Lfunc_end2-_ZN75_$LT$allocators..stack..StackAllocator$u20$as$u20$core..ops..drop..Drop$GT$4drop17hedc05e17ce8bbd69E
	.cfi_endproc

	.ident	"rustc version 1.84.0-nightly (917a50a03 2024-11-15)"
	.section	".note.GNU-stack","",@progbits
