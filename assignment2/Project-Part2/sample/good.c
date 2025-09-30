#include <stdio.h>

int *answer_to_universe;
void c_foo(int *c) {
    answer_to_universe = c;
}

void c_print_answer_to_universe() {
    printf("The answer to the universe is %d\n", *answer_to_universe);
}