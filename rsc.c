#include <stdio.h>
#include <stdlib.h>
#include <time.h>

void rsc_out(int number) {
    printf("%d\n", number);
}

static int min = -0x10000;
static int max = 0x10000;

int rsc_rand() {
    srand(time(NULL));
    return min + rand() % (max - min + 1);
}
