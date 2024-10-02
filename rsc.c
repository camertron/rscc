#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <math.h>

void rsc_init() {
    srand(time(NULL));
}

void rsc_out(double number) {
    printf("%.2f\n", number);
}

double rsc_input() {
    double val = 0;
    int scan_result = 0;

    while (scan_result != 1) {
        printf("Input: ");
        scan_result = scanf("%lf", &val);

        if (scan_result != 1) {
            printf("Invalid entry, try again.\n");

            // clear invalid input
            while (getchar() != '\n');
        }
    }

    return val;
}

static int min = -0x10000;
static int max = 0x10000;

double rsc_rand() {
    double div = 2 + rand() % 11;
    double val = (min + rand() % (max - min + 1)) / div;
    return round(val * 100) / 100;
}
