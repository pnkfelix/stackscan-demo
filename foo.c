#include <stdio.h>
#include <stdlib.h>
#include "libunwind.h"

void foo();
void sub3() { foo(); }
void sub2() { sub3(); }
void sub1() { sub2(); }

int main(int argc, char **argv) {
    // printf("Hello World\n");
    // printf("sizeof(unw_context_t): %d\n", sizeof(unw_context_t));
    sub1();
    return 0;
}

void report_error(unw_error_t err, const char *w);

void foo() {
    unw_context_t ctxt;
    unw_getcontext(&ctxt);
    unw_cursor_t cur;
    unw_init_local(&cur, &ctxt);
#   define LEN 100
    char name[LEN] = {0};
    while (unw_step(&cur) > 0) {
	unw_word_t offset, pc;
	int ret = unw_get_reg(&cur, UNW_REG_IP, &pc);
	if (ret != UNW_ESUCCESS) { report_error(ret, "foo/unw_get_reg"); }
	if (pc == 0) {
	    break;
	}
	printf("0x%lx:", pc);

	unw_word_t offp;
	ret = unw_get_proc_name(&cur, name, LEN, &offp);
	if (ret != UNW_ESUCCESS) { report_error(ret, "foo/unw_get_proc_name"); }
	if (ret == 0) {
	    printf(" (%s+0x%lx)\n", name, offp);
	} else {
	    printf(" -- error: unable to obtain symbol name for this frame\n");
	}
    }
    // unw_proc_info_t proc_info;
    // unw_get_proc_info(&cur, &proc_info);
}

void report_error(unw_error_t err, const char *w) {
    // As noted in the unw_error_t docs, the routines return
    // *negated* values of these error codes.
    switch (-err) {
    UNW_ESUCCESS:
	printf("success condition from %s\n", w);
    UNW_EUNSPEC:
	printf("unspecified error from %s\n", w); exit(-1);
    UNW_ENOINFO:
	printf("unable to find procedure name from %s\n", w);
    UNW_ENOMEM:
	printf("name was truncated from %s\n", w);
	    
    UNW_EBADREG:
	printf("/* bad register number */ from %s\n", w); exit(-1);

    UNW_EREADONLYREG:
	printf("/* attempt to write read-only register */ from %s\n", w); exit(-1);
    UNW_ESTOPUNWIND:
	printf("/* stop unwinding */ from %s\n", w); exit(-1);
    UNW_EINVALIDIP:;
	printf("/* invalid IP */ from %s\n", w); exit(-1);
    UNW_EBADFRAME:
	printf("/* bad frame */ from %s\n", w); exit(-1);
    UNW_EINVAL:
	printf("/* unsupported operation or bad value */ from %s\n", w); exit(-1);
    UNW_EBADVERSION:
	printf("/* unwind info has unsupported version */ from %s\n", w); exit(-1);

    default:
	printf("unknown error value here %d from %s\n", err, w); exit(-1);
    }
}
