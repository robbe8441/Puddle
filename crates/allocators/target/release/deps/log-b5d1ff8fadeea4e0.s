	.text
	.intel_syntax noprefix
	.file	"log.dd1c01103bb47176-cgu.0"
	.section	".text._ZN57_$LT$log..Level$u20$as$u20$core..str..traits..FromStr$GT$8from_str17h6e71fe1af30c2ec6E","ax",@progbits
	.globl	_ZN57_$LT$log..Level$u20$as$u20$core..str..traits..FromStr$GT$8from_str17h6e71fe1af30c2ec6E
	.p2align	4, 0x90
	.type	_ZN57_$LT$log..Level$u20$as$u20$core..str..traits..FromStr$GT$8from_str17h6e71fe1af30c2ec6E,@function
_ZN57_$LT$log..Level$u20$as$u20$core..str..traits..FromStr$GT$8from_str17h6e71fe1af30c2ec6E:
	.cfi_startproc
	cmp	rsi, 3
	jne	.LBB0_5
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 111
	jne	.LBB0_5
	movzx	eax, byte ptr [rdi + 1]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 102
	jne	.LBB0_5
	movzx	eax, byte ptr [rdi + 2]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 102
	je	.LBB0_4
.LBB0_5:
	cmp	rsi, 5
	jne	.LBB0_11
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 101
	jne	.LBB0_11
	movzx	eax, byte ptr [rdi + 1]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 114
	jne	.LBB0_11
	movzx	eax, byte ptr [rdi + 2]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 114
	jne	.LBB0_11
	movzx	eax, byte ptr [rdi + 3]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 111
	jne	.LBB0_11
	movzx	eax, byte ptr [rdi + 4]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	mov	eax, 1
	cmp	cl, 114
	je	.LBB0_36
.LBB0_11:
	cmp	rsi, 4
	jne	.LBB0_22
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 119
	jne	.LBB0_16
	movzx	eax, byte ptr [rdi + 1]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 97
	jne	.LBB0_16
	movzx	eax, byte ptr [rdi + 2]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 114
	jne	.LBB0_16
	movzx	eax, byte ptr [rdi + 3]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	mov	eax, 2
	cmp	cl, 110
	je	.LBB0_36
.LBB0_16:
	cmp	rsi, 4
	jne	.LBB0_22
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 105
	jne	.LBB0_22
	movzx	eax, byte ptr [rdi + 1]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 110
	jne	.LBB0_22
	movzx	eax, byte ptr [rdi + 2]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 102
	jne	.LBB0_22
	cmp	rsi, 5
	setne	dl
	movzx	eax, byte ptr [rdi + 3]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	xor	eax, eax
	cmp	cl, 111
	sete	cl
	or	dl, cl
	je	.LBB0_23
	mov	al, cl
	lea	rax, [rax + 2*rax]
	ret
.LBB0_22:
	cmp	rsi, 5
	jne	.LBB0_4
.LBB0_23:
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 100
	jne	.LBB0_30
	movzx	eax, byte ptr [rdi + 1]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 101
	jne	.LBB0_30
	movzx	eax, byte ptr [rdi + 2]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 98
	jne	.LBB0_30
	movzx	eax, byte ptr [rdi + 3]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 117
	jne	.LBB0_30
	cmp	rsi, 5
	setne	dl
	movzx	eax, byte ptr [rdi + 4]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	xor	eax, eax
	cmp	cl, 103
	sete	cl
	or	dl, cl
	je	.LBB0_31
	mov	al, cl
	shl	eax, 2
	ret
.LBB0_30:
	cmp	rsi, 5
	jne	.LBB0_4
.LBB0_31:
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 116
	jne	.LBB0_4
	movzx	eax, byte ptr [rdi + 1]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 114
	jne	.LBB0_4
	movzx	eax, byte ptr [rdi + 2]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 97
	jne	.LBB0_4
	movzx	eax, byte ptr [rdi + 3]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 99
	jne	.LBB0_4
	movzx	eax, byte ptr [rdi + 4]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	xor	eax, eax
	cmp	cl, 101
	sete	al
	lea	rax, [rax + 4*rax]
.LBB0_36:
	ret
.LBB0_4:
	xor	eax, eax
	ret
.Lfunc_end0:
	.size	_ZN57_$LT$log..Level$u20$as$u20$core..str..traits..FromStr$GT$8from_str17h6e71fe1af30c2ec6E, .Lfunc_end0-_ZN57_$LT$log..Level$u20$as$u20$core..str..traits..FromStr$GT$8from_str17h6e71fe1af30c2ec6E
	.cfi_endproc

	.section	".text._ZN49_$LT$log..Level$u20$as$u20$core..fmt..Display$GT$3fmt17he8763188174be67eE","ax",@progbits
	.globl	_ZN49_$LT$log..Level$u20$as$u20$core..fmt..Display$GT$3fmt17he8763188174be67eE
	.p2align	4, 0x90
	.type	_ZN49_$LT$log..Level$u20$as$u20$core..fmt..Display$GT$3fmt17he8763188174be67eE,@function
_ZN49_$LT$log..Level$u20$as$u20$core..fmt..Display$GT$3fmt17he8763188174be67eE:
	.cfi_startproc
	mov	rax, rsi
	mov	rcx, qword ptr [rdi]
	shl	rcx, 4
	lea	rdx, [rip + _ZN3log15LOG_LEVEL_NAMES17hed9827fff4b6d6e3E]
	mov	rsi, qword ptr [rcx + rdx]
	mov	rdx, qword ptr [rcx + rdx + 8]
	mov	rdi, rax
	jmp	qword ptr [rip + _ZN4core3fmt9Formatter3pad17h1fc1b4f3cd0dac27E@GOTPCREL]
.Lfunc_end1:
	.size	_ZN49_$LT$log..Level$u20$as$u20$core..fmt..Display$GT$3fmt17he8763188174be67eE, .Lfunc_end1-_ZN49_$LT$log..Level$u20$as$u20$core..fmt..Display$GT$3fmt17he8763188174be67eE
	.cfi_endproc

	.section	.text._ZN3log5Level6as_str17hfe0139bef511d06aE,"ax",@progbits
	.globl	_ZN3log5Level6as_str17hfe0139bef511d06aE
	.p2align	4, 0x90
	.type	_ZN3log5Level6as_str17hfe0139bef511d06aE,@function
_ZN3log5Level6as_str17hfe0139bef511d06aE:
	.cfi_startproc
	mov	rcx, qword ptr [rdi]
	shl	rcx, 4
	lea	rdx, [rip + _ZN3log15LOG_LEVEL_NAMES17hed9827fff4b6d6e3E]
	mov	rax, qword ptr [rcx + rdx]
	mov	rdx, qword ptr [rcx + rdx + 8]
	ret
.Lfunc_end2:
	.size	_ZN3log5Level6as_str17hfe0139bef511d06aE, .Lfunc_end2-_ZN3log5Level6as_str17hfe0139bef511d06aE
	.cfi_endproc

	.section	".text._ZN63_$LT$log..LevelFilter$u20$as$u20$core..str..traits..FromStr$GT$8from_str17ha281062f9c8931c8E","ax",@progbits
	.globl	_ZN63_$LT$log..LevelFilter$u20$as$u20$core..str..traits..FromStr$GT$8from_str17ha281062f9c8931c8E
	.p2align	4, 0x90
	.type	_ZN63_$LT$log..LevelFilter$u20$as$u20$core..str..traits..FromStr$GT$8from_str17ha281062f9c8931c8E,@function
_ZN63_$LT$log..LevelFilter$u20$as$u20$core..str..traits..FromStr$GT$8from_str17ha281062f9c8931c8E:
	.cfi_startproc
	cmp	rsi, 3
	jne	.LBB3_5
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 111
	jne	.LBB3_5
	movzx	eax, byte ptr [rdi + 1]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 102
	jne	.LBB3_5
	movzx	eax, byte ptr [rdi + 2]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 102
	jne	.LBB3_5
	xor	eax, eax
	ret
.LBB3_5:
	cmp	rsi, 5
	jne	.LBB3_11
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 101
	jne	.LBB3_11
	movzx	eax, byte ptr [rdi + 1]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 114
	jne	.LBB3_11
	movzx	eax, byte ptr [rdi + 2]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 114
	jne	.LBB3_11
	movzx	eax, byte ptr [rdi + 3]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 111
	jne	.LBB3_11
	movzx	eax, byte ptr [rdi + 4]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	mov	eax, 1
	cmp	cl, 114
	je	.LBB3_35
.LBB3_11:
	cmp	rsi, 4
	jne	.LBB3_22
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 119
	jne	.LBB3_16
	movzx	eax, byte ptr [rdi + 1]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 97
	jne	.LBB3_16
	movzx	eax, byte ptr [rdi + 2]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 114
	jne	.LBB3_16
	movzx	eax, byte ptr [rdi + 3]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	mov	eax, 2
	cmp	cl, 110
	je	.LBB3_35
.LBB3_16:
	cmp	rsi, 4
	jne	.LBB3_22
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 105
	jne	.LBB3_22
	movzx	eax, byte ptr [rdi + 1]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 110
	jne	.LBB3_22
	movzx	eax, byte ptr [rdi + 2]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 102
	jne	.LBB3_22
	cmp	rsi, 5
	setne	dl
	movzx	eax, byte ptr [rdi + 3]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	xor	eax, eax
	cmp	cl, 111
	setne	cl
	sete	r8b
	or	r8b, dl
	je	.LBB3_23
	mov	al, cl
	lea	rax, [rax + 2*rax]
	add	rax, 3
	ret
.LBB3_22:
	mov	eax, 6
	cmp	rsi, 5
	jne	.LBB3_35
.LBB3_23:
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 100
	jne	.LBB3_29
	movzx	eax, byte ptr [rdi + 1]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 101
	jne	.LBB3_29
	movzx	eax, byte ptr [rdi + 2]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 98
	jne	.LBB3_29
	movzx	eax, byte ptr [rdi + 3]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	cmp	cl, 117
	jne	.LBB3_29
	cmp	rsi, 5
	setne	dl
	movzx	eax, byte ptr [rdi + 4]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	xor	eax, eax
	cmp	cl, 103
	setne	cl
	sete	sil
	or	sil, dl
	je	.LBB3_30
	mov	al, cl
	lea	rax, [2*rax + 4]
	ret
.LBB3_29:
	mov	eax, 6
	cmp	rsi, 5
	jne	.LBB3_35
.LBB3_30:
	movzx	eax, byte ptr [rdi]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	mov	eax, 6
	cmp	cl, 116
	jne	.LBB3_35
	movzx	ecx, byte ptr [rdi + 1]
	lea	edx, [rcx - 65]
	cmp	dl, 26
	setb	dl
	shl	dl, 5
	or	dl, cl
	cmp	dl, 114
	jne	.LBB3_35
	movzx	ecx, byte ptr [rdi + 2]
	lea	edx, [rcx - 65]
	cmp	dl, 26
	setb	dl
	shl	dl, 5
	or	dl, cl
	cmp	dl, 97
	jne	.LBB3_35
	movzx	ecx, byte ptr [rdi + 3]
	lea	edx, [rcx - 65]
	cmp	dl, 26
	setb	dl
	shl	dl, 5
	or	dl, cl
	cmp	dl, 99
	jne	.LBB3_35
	movzx	eax, byte ptr [rdi + 4]
	lea	ecx, [rax - 65]
	cmp	cl, 26
	setb	cl
	shl	cl, 5
	or	cl, al
	xor	eax, eax
	cmp	cl, 101
	setne	al
	add	rax, 5
.LBB3_35:
	ret
.Lfunc_end3:
	.size	_ZN63_$LT$log..LevelFilter$u20$as$u20$core..str..traits..FromStr$GT$8from_str17ha281062f9c8931c8E, .Lfunc_end3-_ZN63_$LT$log..LevelFilter$u20$as$u20$core..str..traits..FromStr$GT$8from_str17ha281062f9c8931c8E
	.cfi_endproc

	.section	".text._ZN55_$LT$log..LevelFilter$u20$as$u20$core..fmt..Display$GT$3fmt17h37074f9b82b6cee3E","ax",@progbits
	.globl	_ZN55_$LT$log..LevelFilter$u20$as$u20$core..fmt..Display$GT$3fmt17h37074f9b82b6cee3E
	.p2align	4, 0x90
	.type	_ZN55_$LT$log..LevelFilter$u20$as$u20$core..fmt..Display$GT$3fmt17h37074f9b82b6cee3E,@function
_ZN55_$LT$log..LevelFilter$u20$as$u20$core..fmt..Display$GT$3fmt17h37074f9b82b6cee3E:
	.cfi_startproc
	mov	rax, rsi
	mov	rcx, qword ptr [rdi]
	shl	rcx, 4
	lea	rdx, [rip + _ZN3log15LOG_LEVEL_NAMES17hed9827fff4b6d6e3E]
	mov	rsi, qword ptr [rcx + rdx]
	mov	rdx, qword ptr [rcx + rdx + 8]
	mov	rdi, rax
	jmp	qword ptr [rip + _ZN4core3fmt9Formatter3pad17h1fc1b4f3cd0dac27E@GOTPCREL]
.Lfunc_end4:
	.size	_ZN55_$LT$log..LevelFilter$u20$as$u20$core..fmt..Display$GT$3fmt17h37074f9b82b6cee3E, .Lfunc_end4-_ZN55_$LT$log..LevelFilter$u20$as$u20$core..fmt..Display$GT$3fmt17h37074f9b82b6cee3E
	.cfi_endproc

	.section	.text._ZN3log11LevelFilter6as_str17h26538889c3cee36fE,"ax",@progbits
	.globl	_ZN3log11LevelFilter6as_str17h26538889c3cee36fE
	.p2align	4, 0x90
	.type	_ZN3log11LevelFilter6as_str17h26538889c3cee36fE,@function
_ZN3log11LevelFilter6as_str17h26538889c3cee36fE:
	.cfi_startproc
	mov	rcx, qword ptr [rdi]
	shl	rcx, 4
	lea	rdx, [rip + _ZN3log15LOG_LEVEL_NAMES17hed9827fff4b6d6e3E]
	mov	rax, qword ptr [rcx + rdx]
	mov	rdx, qword ptr [rcx + rdx + 8]
	ret
.Lfunc_end5:
	.size	_ZN3log11LevelFilter6as_str17h26538889c3cee36fE, .Lfunc_end5-_ZN3log11LevelFilter6as_str17h26538889c3cee36fE
	.cfi_endproc

	.section	".text._ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$7enabled17h82a16e921cd0a8a1E","ax",@progbits
	.p2align	4, 0x90
	.type	_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$7enabled17h82a16e921cd0a8a1E,@function
_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$7enabled17h82a16e921cd0a8a1E:
	.cfi_startproc
	xor	eax, eax
	ret
.Lfunc_end6:
	.size	_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$7enabled17h82a16e921cd0a8a1E, .Lfunc_end6-_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$7enabled17h82a16e921cd0a8a1E
	.cfi_endproc

	.section	".text._ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$3log17hdfd55eca0fb9bc38E","ax",@progbits
	.p2align	4, 0x90
	.type	_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$3log17hdfd55eca0fb9bc38E,@function
_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$3log17hdfd55eca0fb9bc38E:
	.cfi_startproc
	ret
.Lfunc_end7:
	.size	_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$3log17hdfd55eca0fb9bc38E, .Lfunc_end7-_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$3log17hdfd55eca0fb9bc38E
	.cfi_endproc

	.section	".text._ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$5flush17h6580199184905da0E","ax",@progbits
	.p2align	4, 0x90
	.type	_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$5flush17h6580199184905da0E,@function
_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$5flush17h6580199184905da0E:
	.cfi_startproc
	ret
.Lfunc_end8:
	.size	_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$5flush17h6580199184905da0E, .Lfunc_end8-_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$5flush17h6580199184905da0E
	.cfi_endproc

	.section	.text._ZN3log10set_logger17h08be3e4e2874083bE,"ax",@progbits
	.globl	_ZN3log10set_logger17h08be3e4e2874083bE
	.p2align	4, 0x90
	.type	_ZN3log10set_logger17h08be3e4e2874083bE,@function
_ZN3log10set_logger17h08be3e4e2874083bE:
	.cfi_startproc
	mov	ecx, 1
	xor	eax, eax
	lock		cmpxchg	qword ptr [rip + _ZN3log5STATE17h1e4a57b154aa94a2E], rcx
	mov	rcx, rax
	sete	al
	jne	.LBB9_1
	mov	qword ptr [rip + _ZN3log6LOGGER17hc5b0c62ace99c604E.0], rdi
	mov	qword ptr [rip + _ZN3log6LOGGER17hc5b0c62ace99c604E.1], rsi
	mov	qword ptr [rip + _ZN3log5STATE17h1e4a57b154aa94a2E], 2
	xor	al, 1
	ret
.LBB9_1:
	cmp	rcx, 1
	jne	.LBB9_5
	.p2align	4, 0x90
	mov	rcx, qword ptr [rip + _ZN3log5STATE17h1e4a57b154aa94a2E]
	cmp	rcx, 1
	jne	.LBB9_5
.LBB9_3:
	pause
	mov	rcx, qword ptr [rip + _ZN3log5STATE17h1e4a57b154aa94a2E]
	cmp	rcx, 1
	je	.LBB9_3
.LBB9_5:
	xor	al, 1
	ret
.Lfunc_end9:
	.size	_ZN3log10set_logger17h08be3e4e2874083bE, .Lfunc_end9-_ZN3log10set_logger17h08be3e4e2874083bE
	.cfi_endproc

	.section	.text._ZN3log15set_logger_racy17h79e1546774ff1681E,"ax",@progbits
	.globl	_ZN3log15set_logger_racy17h79e1546774ff1681E
	.p2align	4, 0x90
	.type	_ZN3log15set_logger_racy17h79e1546774ff1681E,@function
_ZN3log15set_logger_racy17h79e1546774ff1681E:
	.cfi_startproc
	mov	rcx, qword ptr [rip + _ZN3log5STATE17h1e4a57b154aa94a2E]
	test	rcx, rcx
	je	.LBB10_3
	mov	al, 1
	cmp	rcx, 1
	je	.LBB10_2
	ret
.LBB10_3:
	mov	qword ptr [rip + _ZN3log6LOGGER17hc5b0c62ace99c604E.0], rdi
	mov	qword ptr [rip + _ZN3log6LOGGER17hc5b0c62ace99c604E.1], rsi
	mov	qword ptr [rip + _ZN3log5STATE17h1e4a57b154aa94a2E], 2
	xor	eax, eax
	ret
.LBB10_2:
	sub	rsp, 56
	.cfi_def_cfa_offset 64
	lea	rax, [rip + .L__unnamed_1]
	mov	qword ptr [rsp + 8], rax
	mov	qword ptr [rsp + 16], 1
	mov	rax, rsp
	mov	qword ptr [rsp + 24], rax
	xorps	xmm0, xmm0
	movups	xmmword ptr [rsp + 32], xmm0
	lea	rsi, [rip + .L__unnamed_2]
	lea	rdi, [rsp + 8]
	call	qword ptr [rip + _ZN4core9panicking9panic_fmt17hb2b4d3a454bfbc1dE@GOTPCREL]
.Lfunc_end10:
	.size	_ZN3log15set_logger_racy17h79e1546774ff1681E, .Lfunc_end10-_ZN3log15set_logger_racy17h79e1546774ff1681E
	.cfi_endproc

	.section	".text._ZN58_$LT$log..SetLoggerError$u20$as$u20$core..fmt..Display$GT$3fmt17h1372f91e8c0d23b8E","ax",@progbits
	.globl	_ZN58_$LT$log..SetLoggerError$u20$as$u20$core..fmt..Display$GT$3fmt17h1372f91e8c0d23b8E
	.p2align	4, 0x90
	.type	_ZN58_$LT$log..SetLoggerError$u20$as$u20$core..fmt..Display$GT$3fmt17h1372f91e8c0d23b8E,@function
_ZN58_$LT$log..SetLoggerError$u20$as$u20$core..fmt..Display$GT$3fmt17h1372f91e8c0d23b8E:
	.cfi_startproc
	mov	rdi, rsi
	lea	rsi, [rip + .L__unnamed_3]
	mov	edx, 74
	jmp	qword ptr [rip + _ZN4core3fmt9Formatter9write_str17hb8044f2d9bf66897E@GOTPCREL]
.Lfunc_end11:
	.size	_ZN58_$LT$log..SetLoggerError$u20$as$u20$core..fmt..Display$GT$3fmt17h1372f91e8c0d23b8E, .Lfunc_end11-_ZN58_$LT$log..SetLoggerError$u20$as$u20$core..fmt..Display$GT$3fmt17h1372f91e8c0d23b8E
	.cfi_endproc

	.section	".text._ZN59_$LT$log..ParseLevelError$u20$as$u20$core..fmt..Display$GT$3fmt17h0b2a11b9c59bad30E","ax",@progbits
	.globl	_ZN59_$LT$log..ParseLevelError$u20$as$u20$core..fmt..Display$GT$3fmt17h0b2a11b9c59bad30E
	.p2align	4, 0x90
	.type	_ZN59_$LT$log..ParseLevelError$u20$as$u20$core..fmt..Display$GT$3fmt17h0b2a11b9c59bad30E,@function
_ZN59_$LT$log..ParseLevelError$u20$as$u20$core..fmt..Display$GT$3fmt17h0b2a11b9c59bad30E:
	.cfi_startproc
	mov	rdi, rsi
	lea	rsi, [rip + .L__unnamed_4]
	mov	edx, 70
	jmp	qword ptr [rip + _ZN4core3fmt9Formatter9write_str17hb8044f2d9bf66897E@GOTPCREL]
.Lfunc_end12:
	.size	_ZN59_$LT$log..ParseLevelError$u20$as$u20$core..fmt..Display$GT$3fmt17h0b2a11b9c59bad30E, .Lfunc_end12-_ZN59_$LT$log..ParseLevelError$u20$as$u20$core..fmt..Display$GT$3fmt17h0b2a11b9c59bad30E
	.cfi_endproc

	.section	.text._ZN3log6logger17ha0016377e2e3a335E,"ax",@progbits
	.globl	_ZN3log6logger17ha0016377e2e3a335E
	.p2align	4, 0x90
	.type	_ZN3log6logger17ha0016377e2e3a335E,@function
_ZN3log6logger17ha0016377e2e3a335E:
	.cfi_startproc
	mov	rax, qword ptr [rip + _ZN3log5STATE17h1e4a57b154aa94a2E]
	cmp	rax, 2
	lea	rdx, [rip + .L__unnamed_5]
	cmove	rdx, qword ptr [rip + _ZN3log6LOGGER17hc5b0c62ace99c604E.1]
	lea	rax, [rip + _ZN3log6logger3NOP17h7812cdf87fc5b1a9E]
	cmove	rax, qword ptr [rip + _ZN3log6LOGGER17hc5b0c62ace99c604E.0]
	ret
.Lfunc_end13:
	.size	_ZN3log6logger17ha0016377e2e3a335E, .Lfunc_end13-_ZN3log6logger17ha0016377e2e3a335E
	.cfi_endproc

	.section	.text._ZN3log13__private_api8log_impl17hca9e449539be2bc1E,"ax",@progbits
	.globl	_ZN3log13__private_api8log_impl17hca9e449539be2bc1E
	.p2align	4, 0x90
	.type	_ZN3log13__private_api8log_impl17hca9e449539be2bc1E,@function
_ZN3log13__private_api8log_impl17hca9e449539be2bc1E:
	.cfi_startproc
	sub	rsp, 136
	.cfi_def_cfa_offset 144
	test	rcx, rcx
	jne	.LBB14_2
	mov	rax, qword ptr [rdx + 32]
	movups	xmm0, xmmword ptr [rdx]
	movups	xmm1, xmmword ptr [rdx + 16]
	movups	xmm2, xmmword ptr [rdi]
	movups	xmm3, xmmword ptr [rdi + 16]
	movups	xmm4, xmmword ptr [rdi + 32]
	movups	xmm5, xmmword ptr [rax]
	mov	eax, dword ptr [rax + 16]
	mov	rcx, qword ptr [rip + _ZN3log5STATE17h1e4a57b154aa94a2E]
	cmp	rcx, 2
	lea	rcx, [rip + .L__unnamed_5]
	cmove	rcx, qword ptr [rip + _ZN3log6LOGGER17hc5b0c62ace99c604E.1]
	lea	rdi, [rip + _ZN3log6logger3NOP17h7812cdf87fc5b1a9E]
	cmove	rdi, qword ptr [rip + _ZN3log6LOGGER17hc5b0c62ace99c604E.0]
	mov	qword ptr [rsp + 56], rsi
	movups	xmmword ptr [rsp + 64], xmm0
	movups	xmmword ptr [rsp + 88], xmm2
	movups	xmmword ptr [rsp + 104], xmm3
	movups	xmmword ptr [rsp + 120], xmm4
	mov	qword ptr [rsp + 8], 0
	movups	xmmword ptr [rsp + 16], xmm1
	mov	qword ptr [rsp + 32], 0
	movups	xmmword ptr [rsp + 40], xmm5
	mov	dword ptr [rsp + 80], 1
	mov	dword ptr [rsp + 84], eax
	lea	rsi, [rsp + 8]
	call	qword ptr [rcx + 32]
	add	rsp, 136
	.cfi_def_cfa_offset 8
	ret
.LBB14_2:
	.cfi_def_cfa_offset 144
	lea	rax, [rip + .L__unnamed_6]
	mov	qword ptr [rsp + 8], rax
	mov	qword ptr [rsp + 16], 1
	mov	qword ptr [rsp + 24], 8
	xorps	xmm0, xmm0
	movups	xmmword ptr [rsp + 32], xmm0
	lea	rsi, [rip + .L__unnamed_7]
	lea	rdi, [rsp + 8]
	call	qword ptr [rip + _ZN4core9panicking9panic_fmt17hb2b4d3a454bfbc1dE@GOTPCREL]
.Lfunc_end14:
	.size	_ZN3log13__private_api8log_impl17hca9e449539be2bc1E, .Lfunc_end14-_ZN3log13__private_api8log_impl17hca9e449539be2bc1E
	.cfi_endproc

	.section	.text._ZN3log13__private_api7enabled17h95578f41e9b8924eE,"ax",@progbits
	.globl	_ZN3log13__private_api7enabled17h95578f41e9b8924eE
	.p2align	4, 0x90
	.type	_ZN3log13__private_api7enabled17h95578f41e9b8924eE,@function
_ZN3log13__private_api7enabled17h95578f41e9b8924eE:
	.cfi_startproc
	sub	rsp, 24
	.cfi_def_cfa_offset 32
	mov	rax, qword ptr [rip + _ZN3log5STATE17h1e4a57b154aa94a2E]
	cmp	rax, 2
	lea	rcx, [rip + .L__unnamed_5]
	cmove	rcx, qword ptr [rip + _ZN3log6LOGGER17hc5b0c62ace99c604E.1]
	lea	rax, [rip + _ZN3log6logger3NOP17h7812cdf87fc5b1a9E]
	cmove	rax, qword ptr [rip + _ZN3log6LOGGER17hc5b0c62ace99c604E.0]
	mov	qword ptr [rsp], rdi
	mov	qword ptr [rsp + 8], rsi
	mov	qword ptr [rsp + 16], rdx
	mov	rsi, rsp
	mov	rdi, rax
	call	qword ptr [rcx + 24]
	add	rsp, 24
	.cfi_def_cfa_offset 8
	ret
.Lfunc_end15:
	.size	_ZN3log13__private_api7enabled17h95578f41e9b8924eE, .Lfunc_end15-_ZN3log13__private_api7enabled17h95578f41e9b8924eE
	.cfi_endproc

	.section	.text._ZN3log13__private_api3loc17h54585fb30fda2342E,"ax",@progbits
	.globl	_ZN3log13__private_api3loc17h54585fb30fda2342E
	.p2align	4, 0x90
	.type	_ZN3log13__private_api3loc17h54585fb30fda2342E,@function
_ZN3log13__private_api3loc17h54585fb30fda2342E:
	.cfi_startproc
	mov	rax, rdi
	ret
.Lfunc_end16:
	.size	_ZN3log13__private_api3loc17h54585fb30fda2342E, .Lfunc_end16-_ZN3log13__private_api3loc17h54585fb30fda2342E
	.cfi_endproc

	.type	.L__unnamed_5,@object
	.section	.data.rel.ro..L__unnamed_5,"aw",@progbits
	.p2align	3, 0x0
.L__unnamed_5:
	.asciz	"\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\001\000\000\000\000\000\000"
	.quad	_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$7enabled17h82a16e921cd0a8a1E
	.quad	_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$3log17hdfd55eca0fb9bc38E
	.quad	_ZN43_$LT$log..NopLogger$u20$as$u20$log..Log$GT$5flush17h6580199184905da0E
	.size	.L__unnamed_5, 48

	.type	_ZN3log6LOGGER17hc5b0c62ace99c604E.0,@object
	.section	.data._ZN3log6LOGGER17hc5b0c62ace99c604E.0,"aw",@progbits
	.p2align	3, 0x0
_ZN3log6LOGGER17hc5b0c62ace99c604E.0:
	.quad	1
	.size	_ZN3log6LOGGER17hc5b0c62ace99c604E.0, 8

	.type	_ZN3log6LOGGER17hc5b0c62ace99c604E.1,@object
	.section	.data._ZN3log6LOGGER17hc5b0c62ace99c604E.1,"aw",@progbits
	.p2align	3, 0x0
_ZN3log6LOGGER17hc5b0c62ace99c604E.1:
	.quad	.L__unnamed_5
	.size	_ZN3log6LOGGER17hc5b0c62ace99c604E.1, 8

	.type	_ZN3log5STATE17h1e4a57b154aa94a2E,@object
	.section	.bss._ZN3log5STATE17h1e4a57b154aa94a2E,"aw",@nobits
	.p2align	3, 0x0
_ZN3log5STATE17h1e4a57b154aa94a2E:
	.zero	8
	.size	_ZN3log5STATE17h1e4a57b154aa94a2E, 8

	.type	_ZN3log20MAX_LOG_LEVEL_FILTER17hc3b280bf7bb940dfE,@object
	.section	.bss._ZN3log20MAX_LOG_LEVEL_FILTER17hc3b280bf7bb940dfE,"aw",@nobits
	.globl	_ZN3log20MAX_LOG_LEVEL_FILTER17hc3b280bf7bb940dfE
	.p2align	3, 0x0
_ZN3log20MAX_LOG_LEVEL_FILTER17hc3b280bf7bb940dfE:
	.zero	8
	.size	_ZN3log20MAX_LOG_LEVEL_FILTER17hc3b280bf7bb940dfE, 8

	.type	.L__unnamed_8,@object
	.section	.rodata..L__unnamed_8,"a",@progbits
.L__unnamed_8:
	.ascii	"OFF"
	.size	.L__unnamed_8, 3

	.type	.L__unnamed_9,@object
	.section	.rodata..L__unnamed_9,"a",@progbits
.L__unnamed_9:
	.ascii	"ERROR"
	.size	.L__unnamed_9, 5

	.type	.L__unnamed_10,@object
	.section	.rodata.cst4,"aM",@progbits,4
.L__unnamed_10:
	.ascii	"WARN"
	.size	.L__unnamed_10, 4

	.type	.L__unnamed_11,@object
.L__unnamed_11:
	.ascii	"INFO"
	.size	.L__unnamed_11, 4

	.type	.L__unnamed_12,@object
	.section	.rodata..L__unnamed_12,"a",@progbits
.L__unnamed_12:
	.ascii	"DEBUG"
	.size	.L__unnamed_12, 5

	.type	.L__unnamed_13,@object
	.section	.rodata..L__unnamed_13,"a",@progbits
.L__unnamed_13:
	.ascii	"TRACE"
	.size	.L__unnamed_13, 5

	.type	_ZN3log15LOG_LEVEL_NAMES17hed9827fff4b6d6e3E,@object
	.section	.data.rel.ro._ZN3log15LOG_LEVEL_NAMES17hed9827fff4b6d6e3E,"aw",@progbits
	.p2align	3, 0x0
_ZN3log15LOG_LEVEL_NAMES17hed9827fff4b6d6e3E:
	.quad	.L__unnamed_8
	.asciz	"\003\000\000\000\000\000\000"
	.quad	.L__unnamed_9
	.asciz	"\005\000\000\000\000\000\000"
	.quad	.L__unnamed_10
	.asciz	"\004\000\000\000\000\000\000"
	.quad	.L__unnamed_11
	.asciz	"\004\000\000\000\000\000\000"
	.quad	.L__unnamed_12
	.asciz	"\005\000\000\000\000\000\000"
	.quad	.L__unnamed_13
	.asciz	"\005\000\000\000\000\000\000"
	.size	_ZN3log15LOG_LEVEL_NAMES17hed9827fff4b6d6e3E, 96

	.type	.L__unnamed_3,@object
	.section	.rodata..L__unnamed_3,"a",@progbits
.L__unnamed_3:
	.ascii	"attempted to set a logger after the logging system was already initialized"
	.size	.L__unnamed_3, 74

	.type	.L__unnamed_4,@object
	.section	.rodata..L__unnamed_4,"a",@progbits
.L__unnamed_4:
	.ascii	"attempted to convert a string that doesn't match an existing log level"
	.size	.L__unnamed_4, 70

	.type	.L__unnamed_14,@object
	.section	.rodata..L__unnamed_14,"a",@progbits
.L__unnamed_14:
	.ascii	"/home/robbe/.cargo/registry/src/index.crates.io-6f17d22bba15001f/log-0.4.22/src/lib.rs"
	.size	.L__unnamed_14, 86

	.type	.L__unnamed_15,@object
	.section	.rodata..L__unnamed_15,"a",@progbits
.L__unnamed_15:
	.ascii	"internal error: entered unreachable code: set_logger_racy must not be used with other initialization functions"
	.size	.L__unnamed_15, 110

	.type	.L__unnamed_1,@object
	.section	.data.rel.ro..L__unnamed_1,"aw",@progbits
	.p2align	3, 0x0
.L__unnamed_1:
	.quad	.L__unnamed_15
	.asciz	"n\000\000\000\000\000\000"
	.size	.L__unnamed_1, 16

	.type	.L__unnamed_2,@object
	.section	.data.rel.ro..L__unnamed_2,"aw",@progbits
	.p2align	3, 0x0
.L__unnamed_2:
	.quad	.L__unnamed_14
	.asciz	"V\000\000\000\000\000\000\000\257\005\000\000\r\000\000"
	.size	.L__unnamed_2, 24

	.type	_ZN3log6logger3NOP17h7812cdf87fc5b1a9E,@object
	.section	.rodata._ZN3log6logger3NOP17h7812cdf87fc5b1a9E,"a",@progbits
_ZN3log6logger3NOP17h7812cdf87fc5b1a9E:
	.size	_ZN3log6logger3NOP17h7812cdf87fc5b1a9E, 0

	.type	.L__unnamed_16,@object
	.section	.rodata..L__unnamed_16,"a",@progbits
.L__unnamed_16:
	.ascii	"key-value support is experimental and must be enabled using the `kv` feature"
	.size	.L__unnamed_16, 76

	.type	.L__unnamed_6,@object
	.section	.data.rel.ro..L__unnamed_6,"aw",@progbits
	.p2align	3, 0x0
.L__unnamed_6:
	.quad	.L__unnamed_16
	.asciz	"L\000\000\000\000\000\000"
	.size	.L__unnamed_6, 16

	.type	.L__unnamed_17,@object
	.section	.rodata..L__unnamed_17,"a",@progbits
.L__unnamed_17:
	.ascii	"/home/robbe/.cargo/registry/src/index.crates.io-6f17d22bba15001f/log-0.4.22/src/__private_api.rs"
	.size	.L__unnamed_17, 96

	.type	.L__unnamed_7,@object
	.section	.data.rel.ro..L__unnamed_7,"aw",@progbits
	.p2align	3, 0x0
.L__unnamed_7:
	.quad	.L__unnamed_17
	.asciz	"`\000\000\000\000\000\000\000-\000\000\000\t\000\000"
	.size	.L__unnamed_7, 24

	.ident	"rustc version 1.84.0-nightly (917a50a03 2024-11-15)"
	.section	".note.GNU-stack","",@progbits
