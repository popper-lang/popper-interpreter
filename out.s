	.section	__TEXT,__text,regular,pure_instructions
	.build_version macos, 14, 0
	.globl	_main                           ## -- Begin function main
	.p2align	4, 0x90
_main:                                  ## @main
	.cfi_startproc
## %bb.0:                               ## %entry
	subq	$24, %rsp
	.cfi_def_cfa_offset 32
	movabsq	$8589934593, %rax               ## imm = 0x200000001
	movq	%rax, 16(%rsp)
	leaq	L_g0(%rip), %rdi
	movl	$1, %esi
	xorl	%eax, %eax
	callq	_printf
	movl	%eax, 12(%rsp)
	xorl	%eax, %eax
	addq	$24, %rsp
	retq
	.cfi_endproc
                                        ## -- End function
	.section	__TEXT,__cstring,cstring_literals
L_g0:                                   ## @g0
	.asciz	"%s"

.subsections_via_symbols
